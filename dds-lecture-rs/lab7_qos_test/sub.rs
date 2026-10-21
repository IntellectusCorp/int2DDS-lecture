//! 실습 7 · 레시피 Subscriber (실습 1 sub + recipes.rs + 상태 콜백)
//!   --recipe hard_state|health|failover  (+ --depth N | --keep-all, --deadline-ms N, --lease-ms N)
//!   --tbf-ms N   TIME_BASED_FILTER minimum_separation (D)
//!   --rate       샘플별 출력 대신 초당 수신 수 (D)
//!   --gaps       index가 건너뛰면 표시 (KEEP_LAST(1) + 빠른 쓰기)
//!   --domain N --topic NAME
use std::sync::Arc;

use clap::Parser;
use int2dds::{
    core::error::{DdsError, DdsResult},
    domain::{domain_participant_factory::DomainParticipantFactory, qos::PARTICIPANT_QOS_DEFAULT},
    infrastructure::{
        qos_policy::TimeBasedFilterQosPolicy,
        status::{
            LivelinessChangedStatus, RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus,
            StatusMask, SubscriptionMatchedStatus,
        },
    },
    subscription::{
        data_reader::DataReader,
        data_reader_listener::DataReaderListener,
        qos::{DataReaderQos, SUBSCRIBER_QOS_DEFAULT},
        sample_info::{InstanceStateKind, SampleStateKind, ViewStateKind},
    },
    topic::qos::TOPIC_QOS_DEFAULT,
};
use lecture::{
    hello::HelloWorld,
    lab::{self, ms},
    lab_log, recipes,
    shutdown::{cleanup_participant, Shutdown},
};

#[derive(Parser)]
struct Args {
    #[arg(long, default_value_t = 0)]
    domain: i32,
    #[arg(long, default_value = "hello_world_topic")]
    topic: String,
    #[arg(long, default_value = "none")]
    recipe: String,
    #[arg(long, default_value_t = 1)]
    depth: i32,
    #[arg(long)]
    keep_all: bool,
    #[arg(long, default_value_t = 1000)]
    deadline_ms: i64,
    #[arg(long, default_value_t = 2000)]
    lease_ms: i64,
    #[arg(long, default_value_t = 0)]
    tbf_ms: i64,
    #[arg(long)]
    rate: bool,
    #[arg(long)]
    gaps: bool,
}

struct Listener;

impl DataReaderListener for Listener {
    type Foo = HelloWorld;

    fn on_subscription_matched(&self, _r: &DataReader<HelloWorld>, st: &SubscriptionMatchedStatus) {
        lab_log!(
            "SUBSCRIPTION_MATCHED: current_count={} ({:+})",
            st.current_count(),
            st.current_count_change()
        );
    }

    fn on_requested_deadline_missed(
        &self,
        _r: &DataReader<HelloWorld>,
        st: &RequestedDeadlineMissedStatus,
    ) {
        lab_log!(
            "REQUESTED_DEADLINE_MISSED: total={} instance={}",
            st.total_count(),
            lab::hex4(&st.last_instance_handle())
        );
    }

    fn on_liveliness_changed(&self, _r: &DataReader<HelloWorld>, st: &LivelinessChangedStatus) {
        lab_log!(
            "LIVELINESS_CHANGED: alive={} ({:+}) not_alive={} ({:+})",
            st.alive_count(),
            st.alive_count_change(),
            st.not_alive_count(),
            st.not_alive_count_change()
        );
    }

    fn on_requested_incompatible_qos(
        &self,
        _r: &DataReader<HelloWorld>,
        st: &RequestedIncompatibleQosStatus,
    ) {
        lab_log!("REQUESTED_INCOMPATIBLE_QOS: last_policy={}", lab::policy_name(st.last_policy_id()));
    }
}

fn main() -> DdsResult<()> {
    lab::init();
    let shutdown = Shutdown::install();
    let a = Args::parse();

    let factory = DomainParticipantFactory::get_instance();
    let dp = factory.create_participant(a.domain, PARTICIPANT_QOS_DEFAULT, None, StatusMask::NONE)?;
    let subscriber = dp.create_subscriber(SUBSCRIBER_QOS_DEFAULT, None, StatusMask::NONE)?;
    let topic = dp.create_topic::<HelloWorld>(
        &a.topic,
        "demo::HelloWorld",
        TOPIC_QOS_DEFAULT,
        None,
        StatusMask::NONE,
    )?;

    let mut qos = DataReaderQos::default();
    match a.recipe.as_str() {
        "hard_state" => recipes::hard_state_reader(&mut qos, a.depth, a.keep_all),
        "health" => recipes::health_reader(&mut qos, a.deadline_ms, a.lease_ms),
        "failover" => recipes::failover_reader(&mut qos, a.lease_ms),
        _ => {}
    }
    if a.tbf_ms > 0 {
        qos.time_based_filter = TimeBasedFilterQosPolicy { minimum_separation: ms(a.tbf_ms) };
    }

    let mask = StatusMask::SUBSCRIPTION_MATCHED
        | StatusMask::REQUESTED_DEADLINE_MISSED
        | StatusMask::LIVELINESS_CHANGED
        | StatusMask::REQUESTED_INCOMPATIBLE_QOS;
    let reader = subscriber.create_datareader::<HelloWorld>(&topic, qos, Some(Arc::new(Listener)), mask)?;
    lab_log!(
        "sub ready: recipe={} depth={} keep_all={} deadline={} lease={} tbf={}",
        a.recipe,
        a.depth,
        a.keep_all,
        a.deadline_ms,
        a.lease_ms,
        a.tbf_ms
    );

    let (mut last_index, mut per_sec) = (-1, 0);
    let mut next_rate = lab::elapsed_ms() + 1000;
    while !shutdown.is_stopped() {
        if a.rate && lab::elapsed_ms() >= next_rate {
            lab_log!("rate: {} samples/s", per_sec);
            per_sec = 0;
            next_rate += 1000;
        }
        let samples = match reader.take(
            1,
            &[SampleStateKind::ANY_SAMPLE_STATE],
            &[ViewStateKind::ANY_VIEW_STATE],
            &[InstanceStateKind::ANY_INSTANCE_STATE],
        ) {
            Err(DdsError::NoData) => {
                shutdown.wait_timeout(if a.rate { 5 } else { 50 });
                continue;
            }
            other => other?,
        };
        for s in &samples {
            let info = s.sample_info();
            if !info.valid_data {
                lab_log!("{} (valid_data=false)", lab::instance_state(info.instance_state));
                continue;
            }
            let sample = s.data()?;
            per_sec += 1;
            if a.gaps && last_index >= 0 && sample.index != last_index + 1 {
                lab_log!("GAP: expected {} got {}", last_index + 1, sample.index);
            }
            last_index = sample.index;
            if !a.rate {
                lab_log!("recv index={} from={}", sample.index, sample.message);
            }
        }
    }

    cleanup_participant(dp);
    Ok(())
}

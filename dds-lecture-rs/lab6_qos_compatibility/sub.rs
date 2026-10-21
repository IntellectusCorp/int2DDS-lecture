//! 실습 6 · 일부러 안 맞추기 (Reader, 실습 1 sub + QoS 인자)
//!   --reliability reliable|best_effort   --deadline-ms N   --ownership shared|exclusive
//!   --domain N --topic NAME
use std::sync::Arc;

use clap::Parser;
use int2dds::{
    core::error::{DdsError, DdsResult},
    domain::{domain_participant_factory::DomainParticipantFactory, qos::PARTICIPANT_QOS_DEFAULT},
    infrastructure::{
        qos_policy::{DeadlineQosPolicy, OwnershipQosPolicy, ReliabilityQosPolicy},
        status::{
            RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus, StatusMask,
            SubscriptionMatchedStatus,
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
    lab_log,
    shutdown::{cleanup_participant, Shutdown},
};

#[derive(Parser)]
struct Args {
    #[arg(long, default_value_t = 0)]
    domain: i32,
    #[arg(long, default_value = "hello_world_topic")]
    topic: String,
    #[arg(long, default_value = "reliable")]
    reliability: String,
    #[arg(long, default_value_t = 0)]
    deadline_ms: i64,
    #[arg(long, default_value = "shared")]
    ownership: String,
}

struct Listener;

impl DataReaderListener for Listener {
    type Foo = HelloWorld;

    fn on_requested_incompatible_qos(
        &self,
        _r: &DataReader<HelloWorld>,
        st: &RequestedIncompatibleQosStatus,
    ) {
        lab_log!(
            "REQUESTED_INCOMPATIBLE_QOS: last_policy={} total={}",
            lab::policy_name(st.last_policy_id()),
            st.total_count()
        );
    }

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
        lab_log!("REQUESTED_DEADLINE_MISSED: total={}", st.total_count());
    }
}

fn main() -> DdsResult<()> {
    lab::init();
    let shutdown = Shutdown::install();
    let args = Args::parse();

    let factory = DomainParticipantFactory::get_instance();
    let dp = factory.create_participant(args.domain, PARTICIPANT_QOS_DEFAULT, None, StatusMask::NONE)?;
    let subscriber = dp.create_subscriber(SUBSCRIBER_QOS_DEFAULT, None, StatusMask::NONE)?;
    let topic = dp.create_topic::<HelloWorld>(
        &args.topic,
        "demo::HelloWorld",
        TOPIC_QOS_DEFAULT,
        None,
        StatusMask::NONE,
    )?;

    let mut qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: lab::reliability_kind(&args.reliability),
            max_blocking_time: ms(0),
        },
        ownership: OwnershipQosPolicy { kind: lab::ownership_kind(&args.ownership) },
        ..Default::default()
    };
    if args.deadline_ms > 0 {
        qos.deadline = DeadlineQosPolicy { period: ms(args.deadline_ms) };
    }

    let mask = StatusMask::REQUESTED_INCOMPATIBLE_QOS
        | StatusMask::SUBSCRIPTION_MATCHED
        | StatusMask::REQUESTED_DEADLINE_MISSED;
    let reader = subscriber.create_datareader::<HelloWorld>(&topic, qos, Some(Arc::new(Listener)), mask)?;
    lab_log!(
        "sub ready: reliability={} deadline={} ms ownership={}",
        args.reliability,
        args.deadline_ms,
        args.ownership
    );

    let mut next_status = 0;
    while !shutdown.is_stopped() {
        if lab::elapsed_ms() >= next_status {
            let sm = reader.get_subscription_matched_status()?;
            let rq = reader.get_requested_incompatible_qos_status()?;
            lab_log!(
                "status | matched={} incompatible_qos total={} last={}",
                sm.current_count(),
                rq.total_count(),
                lab::policy_name(rq.last_policy_id())
            );
            next_status = lab::elapsed_ms() + 1000;
        }
        let samples = match reader.take(
            1,
            &[SampleStateKind::ANY_SAMPLE_STATE],
            &[ViewStateKind::ANY_VIEW_STATE],
            &[InstanceStateKind::ANY_INSTANCE_STATE],
        ) {
            Err(DdsError::NoData) => {
                shutdown.wait_timeout(100);
                continue;
            }
            other => other?,
        };
        for s in samples.iter().filter(|s| s.sample_info().valid_data) {
            lab_log!("recv index={}", s.data()?.index);
        }
    }

    cleanup_participant(dp);
    Ok(())
}

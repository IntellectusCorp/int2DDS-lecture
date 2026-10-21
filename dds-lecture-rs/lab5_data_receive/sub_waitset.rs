//! 실습 5 · WaitSet 버전 (실습 2 sub → ReadCondition + GuardCondition)
//!   wait()에서 블로킹, 처리 스레드와 순서는 애플리케이션이 쥔다.
//!   Ctrl+C → 감시 스레드가 GuardCondition을 켜서 루프를 깨운다.
//!   --domain N   --timeout-ms N (기본 1000; TIMEOUT 반환을 보려면 pub 없이 띄운다)
use std::sync::Arc;

use clap::Parser;
use int2dds::{
    core::error::{DdsError, DdsResult},
    domain::{domain_participant_factory::DomainParticipantFactory, qos::PARTICIPANT_QOS_DEFAULT},
    infrastructure::{
        condition::Condition,
        guard_condition::GuardCondition,
        qos_policy::{ReliabilityQosPolicy, ReliabilityQosPolicyKind},
        status::StatusMask,
        wait_set::WaitSet,
    },
    subscription::{
        qos::{DataReaderQos, SUBSCRIBER_QOS_DEFAULT},
        sample_info::{InstanceStateKind, SampleStateKind, ViewStateKind},
    },
    topic::qos::TOPIC_QOS_DEFAULT,
};
use lecture::{
    floor::TilePressure,
    lab::{self, ms},
    lab_log,
    shutdown::{cleanup_participant, Shutdown},
};

#[derive(Parser)]
struct Args {
    #[arg(long, default_value_t = 0)]
    domain: i32,
    #[arg(long, default_value_t = 1000)]
    timeout_ms: i64,
}

fn main() -> DdsResult<()> {
    lab::init();
    let shutdown = Shutdown::install();
    let args = Args::parse();

    let factory = DomainParticipantFactory::get_instance();
    let dp = factory.create_participant(args.domain, PARTICIPANT_QOS_DEFAULT, None, StatusMask::NONE)?;
    let subscriber = dp.create_subscriber(SUBSCRIBER_QOS_DEFAULT, None, StatusMask::NONE)?;
    let topic = dp.create_topic::<TilePressure>(
        "TilePressure",
        "TilePressure",
        TOPIC_QOS_DEFAULT,
        None,
        StatusMask::NONE,
    )?;
    let qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: ms(0),
        },
        ..Default::default()
    };
    let reader = subscriber.create_datareader::<TilePressure>(&topic, qos, None, StatusMask::NONE)?;

    let ws = WaitSet::new();
    let new_data = reader.create_readcondition(
        &[SampleStateKind::NOT_READ_SAMPLE_STATE],
        &[ViewStateKind::ANY_VIEW_STATE],
        &[InstanceStateKind::ANY_INSTANCE_STATE],
    )?;
    let stop = Arc::new(GuardCondition::new());
    ws.attach_condition(new_data.clone())?;
    ws.attach_condition(stop.clone() as Arc<dyn Condition + Send + Sync>)?;
    {
        let (shutdown, stop) = (shutdown.clone(), stop.clone());
        std::thread::spawn(move || {
            shutdown.wait();
            let _ = stop.set_trigger_value(true);
        });
    }
    lab_log!("sub ready (waitset): domain={} timeout={} ms", args.domain, args.timeout_ms);

    loop {
        match ws.wait(ms(args.timeout_ms)) {
            Err(DdsError::Timeout) => {
                lab_log!("wait: TIMEOUT");
                continue;
            }
            other => other?,
        };
        if stop.get_trigger_value()? {
            break;
        }

        let samples = match reader.take_w_condition(32, new_data.clone()) {
            Err(DdsError::NoData) => continue,
            other => other?,
        };
        for s in &samples {
            let info = s.sample_info();
            if info.valid_data {
                let t = s.data()?;
                lab_log!("tile({},{}) p={:5.1}", t.row, t.col, t.pressure);
            } else {
                lab_log!("{}", lab::instance_state(info.instance_state));
            }
        }
    }

    ws.detach_condition(new_data.clone())?;
    ws.detach_condition(stop as Arc<dyn Condition + Send + Sync>)?;
    reader.delete_readcondition(new_data)?;
    cleanup_participant(dp);
    Ok(())
}

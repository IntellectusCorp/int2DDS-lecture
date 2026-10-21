//! 실습 1 · Subscriber (폴링)
//!   --domain N   --topic NAME
use clap::Parser;
use int2dds::{
    core::error::{DdsError, DdsResult},
    domain::{domain_participant_factory::DomainParticipantFactory, qos::PARTICIPANT_QOS_DEFAULT},
    infrastructure::{
        qos_policy::{ReliabilityQosPolicy, ReliabilityQosPolicyKind},
        status::StatusMask,
    },
    subscription::{
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

    let qos = DataReaderQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: ms(0),
        },
        ..Default::default()
    };
    let reader = subscriber.create_datareader::<HelloWorld>(&topic, qos, None, StatusMask::NONE)?;
    lab_log!("sub ready: domain={} topic={}", args.domain, args.topic);

    while !shutdown.is_stopped() {
        let samples = match reader.take(
            1,
            &[SampleStateKind::ANY_SAMPLE_STATE],
            &[ViewStateKind::ANY_VIEW_STATE],
            &[InstanceStateKind::ANY_INSTANCE_STATE],
        ) {
            Err(DdsError::NoData) => {
                shutdown.wait_timeout(200);
                continue;
            }
            other => other?,
        };
        for s in &samples {
            if !s.sample_info().valid_data {
                continue; // 상태 통지: 데이터가 없다
            }
            let sample = s.data()?;
            lab_log!("recv index={} message={}", sample.index, sample.message);
        }
    }

    cleanup_participant(dp);
    Ok(())
}

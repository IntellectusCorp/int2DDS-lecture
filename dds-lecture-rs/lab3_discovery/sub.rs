//! 실습 3 · Discovery 관찰
//!   실습 1 sub + 1초마다 발견된 Participant 목록과 매칭된 Writer 수를 찍는다.
//!   목록은 상대만 센다(자기 자신 제외). pub 쪽은 lab1_pub 그대로 쓴다.
//!   --domain N --topic NAME
use clap::Parser;
use int2dds::{
    core::error::{DdsError, DdsResult},
    domain::{
        domain_participant::DomainParticipant,
        domain_participant_factory::DomainParticipantFactory, qos::PARTICIPANT_QOS_DEFAULT,
    },
    infrastructure::{
        qos_policy::{ReliabilityQosPolicy, ReliabilityQosPolicyKind},
        status::StatusMask,
    },
    subscription::{
        data_reader::DataReader,
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

fn report(dp: &DomainParticipant, reader: &DataReader<HelloWorld>) -> DdsResult<()> {
    let handles = dp.get_discovered_participants()?;
    let list: String = handles.iter().map(|h| lab::hex4(h) + " ").collect();
    let sm = reader.get_subscription_matched_status()?;
    lab_log!("participants={} [{}]  matched writers={}", handles.len(), list, sm.current_count());
    Ok(())
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

    let mut next_report = 0;
    while !shutdown.is_stopped() {
        if lab::elapsed_ms() >= next_report {
            report(&dp, &reader)?;
            next_report = lab::elapsed_ms() + 1000;
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
            let sample = s.data()?;
            lab_log!("recv index={} message={}", sample.index, sample.message);
        }
    }

    cleanup_participant(dp);
    Ok(())
}

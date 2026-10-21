//! 실습 4 · Partition Subscriber (실습 3 sub + Subscriber QoS의 PARTITION + 런타임 변경)
//!   --partition floor-*[,x]            시작 파티션 (없으면 기본 "")
//!   --switch-after S --switch-to LIST  S초 뒤 파티션을 바꾼다
//!   --domain N --topic NAME
use clap::Parser;
use int2dds::{
    core::error::{DdsError, DdsResult},
    domain::{
        domain_participant::DomainParticipant,
        domain_participant_factory::DomainParticipantFactory, qos::PARTICIPANT_QOS_DEFAULT,
    },
    infrastructure::{qos_policy::PartitionQosPolicy, status::StatusMask},
    subscription::{
        data_reader::DataReader,
        qos::{SubscriberQos, DATAREADER_QOS_DEFAULT},
        sample_info::{InstanceStateKind, SampleStateKind, ViewStateKind},
    },
    topic::qos::TOPIC_QOS_DEFAULT,
};
use lecture::{
    hello::HelloWorld,
    lab, lab_log,
    shutdown::{cleanup_participant, Shutdown},
};

#[derive(Parser)]
struct Args {
    #[arg(long, default_value_t = 0)]
    domain: i32,
    #[arg(long, default_value = "hello_world_topic")]
    topic: String,
    #[arg(long)]
    partition: Option<String>,
    #[arg(long, default_value_t = 0)]
    switch_after: u64,
    #[arg(long)]
    switch_to: Option<String>,
}

fn report(dp: &DomainParticipant, reader: &DataReader<HelloWorld>) -> DdsResult<()> {
    let count = dp.get_discovered_participants()?.len();
    let sm = reader.get_subscription_matched_status()?;
    lab_log!("participants={}  matched writers={}", count, sm.current_count());
    Ok(())
}

fn main() -> DdsResult<()> {
    lab::init();
    let shutdown = Shutdown::install();
    let args = Args::parse();
    let parts = lab::split_list(args.partition.as_deref());

    let factory = DomainParticipantFactory::get_instance();
    let dp = factory.create_participant(args.domain, PARTICIPANT_QOS_DEFAULT, None, StatusMask::NONE)?;
    let sq = SubscriberQos {
        partition: PartitionQosPolicy { name: parts.clone() },
        ..Default::default()
    };
    let subscriber = dp.create_subscriber(sq, None, StatusMask::NONE)?;
    let topic = dp.create_topic::<HelloWorld>(
        &args.topic,
        "demo::HelloWorld",
        TOPIC_QOS_DEFAULT,
        None,
        StatusMask::NONE,
    )?;
    let reader =
        subscriber.create_datareader::<HelloWorld>(&topic, DATAREADER_QOS_DEFAULT, None, StatusMask::NONE)?;
    lab_log!(
        "sub ready: domain={} partitions={} ({})",
        args.domain,
        parts.len(),
        args.partition.as_deref().unwrap_or("\"\"")
    );

    let mut next_report = 0;
    let mut switched = false;
    while !shutdown.is_stopped() {
        if lab::elapsed_ms() >= next_report {
            report(&dp, &reader)?;
            next_report = lab::elapsed_ms() + 1000;
        }
        if args.switch_after > 0 && !switched && lab::elapsed_ms() >= args.switch_after as u128 * 1000 {
            // Partition은 변경 가능한 QoS: Subscriber set_qos로 바꾸면 Reader가 다시 광고된다
            let mut q = subscriber.get_qos()?;
            q.partition.name = lab::split_list(args.switch_to.as_deref());
            subscriber.set_qos(q)?;
            switched = true;
            lab_log!("partition switched to ({})", args.switch_to.as_deref().unwrap_or("\"\""));
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

//! 실습 4 · Partition Publisher (실습 1 pub + Publisher QoS의 PARTITION)
//!   --partition floor-1[,floor-2]   --domain N --topic NAME --period-ms N --count N(0=무한)
use clap::Parser;
use int2dds::{
    common::instance_handle::InstanceHandle,
    core::error::DdsResult,
    domain::{domain_participant_factory::DomainParticipantFactory, qos::PARTICIPANT_QOS_DEFAULT},
    infrastructure::{qos_policy::PartitionQosPolicy, status::StatusMask},
    publication::qos::{PublisherQos, DATAWRITER_QOS_DEFAULT},
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
    #[arg(long, default_value_t = 500)]
    period_ms: u64,
    #[arg(long, default_value_t = 0)]
    count: i32,
    #[arg(long)]
    partition: Option<String>,
}

fn main() -> DdsResult<()> {
    lab::init();
    let shutdown = Shutdown::install();
    let args = Args::parse();
    let parts = lab::split_list(args.partition.as_deref());

    let factory = DomainParticipantFactory::get_instance();
    let dp = factory.create_participant(args.domain, PARTICIPANT_QOS_DEFAULT, None, StatusMask::NONE)?;

    // Partition은 Publisher/Subscriber QoS다
    let pq = PublisherQos {
        partition: PartitionQosPolicy { name: parts.clone() },
        ..Default::default()
    };
    let publisher = dp.create_publisher(pq, None, StatusMask::NONE)?;

    let topic = dp.create_topic::<HelloWorld>(
        &args.topic,
        "demo::HelloWorld",
        TOPIC_QOS_DEFAULT,
        None,
        StatusMask::NONE,
    )?;
    let writer =
        publisher.create_datawriter::<HelloWorld>(&topic, DATAWRITER_QOS_DEFAULT, None, StatusMask::NONE)?;
    lab_log!(
        "pub ready: domain={} partitions={} ({})",
        args.domain,
        parts.len(),
        args.partition.as_deref().unwrap_or("\"\"")
    );

    let mut matched = 0;
    let mut i = 1;
    while !shutdown.is_stopped() && (args.count == 0 || i <= args.count) {
        writer.write(&HelloWorld { index: i, message: "hello".into() }, InstanceHandle::NIL)?;
        lab_log!("write index={}", i);
        lab::poll_writer_matched(&writer, &mut matched)?;
        if shutdown.wait_timeout(args.period_ms) {
            break;
        }
        i += 1;
    }

    cleanup_participant(dp);
    Ok(())
}

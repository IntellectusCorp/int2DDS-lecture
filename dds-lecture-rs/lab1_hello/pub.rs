//! 실습 1 · Publisher
//!   --domain N        (변형 1: pub만 domain 1)
//!   --topic NAME      (변형 2: 토픽 이름 바꾸기)
//!   --no-wait         (변형 3: 매칭 대기 없이 바로 write)
//!   --count N --period-ms N
//!   --hold            다 쓰고도 Ctrl+C까지 살아 있기 (실습 3·7)
//!   --crash-after S   S초 뒤 정리 없이 죽기 (실습 3: 정상 종료와 비교, 상대는 약 100초 뒤 감지)
use clap::Parser;
use int2dds::{
    common::instance_handle::InstanceHandle,
    core::error::DdsResult,
    domain::{domain_participant_factory::DomainParticipantFactory, qos::PARTICIPANT_QOS_DEFAULT},
    infrastructure::{
        qos_policy::{ReliabilityQosPolicy, ReliabilityQosPolicyKind},
        status::StatusMask,
    },
    publication::qos::{DataWriterQos, PUBLISHER_QOS_DEFAULT},
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
    #[arg(long, default_value_t = 10)]
    count: i32,
    #[arg(long, default_value_t = 500)]
    period_ms: u64,
    #[arg(long)]
    no_wait: bool,
    #[arg(long)]
    hold: bool,
    #[arg(long, default_value_t = 0)]
    crash_after: u64,
}

fn main() -> DdsResult<()> {
    lab::init();
    let shutdown = Shutdown::install();
    let args = Args::parse();

    let factory = DomainParticipantFactory::get_instance();
    let dp = factory.create_participant(args.domain, PARTICIPANT_QOS_DEFAULT, None, StatusMask::NONE)?;
    let publisher = dp.create_publisher(PUBLISHER_QOS_DEFAULT, None, StatusMask::NONE)?;
    let topic = dp.create_topic::<HelloWorld>(
        &args.topic,
        "demo::HelloWorld",
        TOPIC_QOS_DEFAULT,
        None,
        StatusMask::NONE,
    )?;

    let qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: ms(100),
        },
        ..Default::default()
    };
    let writer = publisher.create_datawriter::<HelloWorld>(&topic, qos, None, StatusMask::NONE)?;
    lab_log!("pub ready: domain={} topic={}", args.domain, args.topic);

    // 매칭 대기는 실습 편의용이지 write의 필수 조건은 아니다
    let mut matched = 0;
    if !args.no_wait {
        while !shutdown.is_stopped() && !lab::poll_writer_matched(&writer, &mut matched)? {
            shutdown.wait_timeout(100);
        }
    }

    for i in 1..=args.count {
        if shutdown.is_stopped() {
            break;
        }
        let sample = HelloWorld { index: i, message: "hello".into() };
        writer.write(&sample, InstanceHandle::NIL)?;
        lab_log!("write index={}", i);

        lab::poll_writer_matched(&writer, &mut matched)?;
        lab::crash_check(args.crash_after);
        if shutdown.wait_timeout(args.period_ms) {
            break;
        }
    }

    while args.hold && !shutdown.is_stopped() {
        lab::poll_writer_matched(&writer, &mut matched)?;
        lab::crash_check(args.crash_after);
        shutdown.wait_timeout(200);
    }

    cleanup_participant(dp);
    Ok(())
}

//! 실습 2 · Active Floor Publisher
//!   타일 4개를 주기적으로 쓰다가 cycles회 뒤 (1,1)을 dispose, (1,2)를 unregister 한다.
//!   좌표는 1부터: 키가 전부 0이면 InstanceHandle이 16바이트 0(NIL)과 같아진다.
//!   --cycles N --period-ms N
//!   --autodispose 0|1   unregister 시 autodispose_unregistered_instances (기본 1 → DISPOSED)
//!   --events            StepEvent 토픽도 함께 발행 (도전)
//!   --no-wait           Reader 매칭을 기다리지 않는다 (late joiner 실험: 실습 8)
//!   --hold              Ctrl+C까지 남은 타일을 계속 쓴다
//!   --crash-after S     정리 없이 죽기 → 남은 타일이 NO_WRITERS가 되는 시점 관찰 (약 100초 뒤)
//!   --profile Lib::Prof [--profile-file F]   Writer QoS를 프로파일로 (실습 8)
use std::time::{SystemTime, UNIX_EPOCH};

use clap::Parser;
use int2dds::{
    common::instance_handle::InstanceHandle,
    core::error::DdsResult,
    domain::{domain_participant_factory::DomainParticipantFactory, qos::PARTICIPANT_QOS_DEFAULT},
    infrastructure::{
        qos_policy::{
            ReliabilityQosPolicy, ReliabilityQosPolicyKind, WriterDataLifecycleQosPolicy,
        },
        status::StatusMask,
    },
    publication::qos::{DataWriterQos, DATAWRITER_QOS_DEFAULT, PUBLISHER_QOS_DEFAULT},
    topic::qos::TOPIC_QOS_DEFAULT,
};
use lecture::{
    floor::{StepEvent, TilePressure},
    lab::{self, ms},
    lab_log,
    shutdown::{cleanup_participant, Shutdown},
};

const TILES: [(i16, i16); 4] = [(1, 1), (1, 2), (2, 1), (2, 2)];

#[derive(Parser)]
struct Args {
    #[arg(long, default_value_t = 0)]
    domain: i32,
    #[arg(long, default_value_t = 5)]
    cycles: i32,
    #[arg(long, default_value_t = 500)]
    period_ms: u64,
    #[arg(long, default_value_t = 1)]
    autodispose: u8,
    #[arg(long)]
    events: bool,
    #[arg(long)]
    no_wait: bool,
    #[arg(long)]
    hold: bool,
    #[arg(long, default_value_t = 0)]
    crash_after: u64,
    #[arg(long)]
    profile: Option<String>,
    #[arg(long, default_value = concat!(env!("CARGO_MANIFEST_DIR"), "/lab8_xml/team_qos.xml"))]
    profile_file: String,
}

fn main() -> DdsResult<()> {
    lab::init();
    let shutdown = Shutdown::install();
    let args = Args::parse();
    let autodispose = args.autodispose != 0;

    let factory = DomainParticipantFactory::get_instance();
    let dp = factory.create_participant(args.domain, PARTICIPANT_QOS_DEFAULT, None, StatusMask::NONE)?;
    let publisher = dp.create_publisher(PUBLISHER_QOS_DEFAULT, None, StatusMask::NONE)?;
    // keyed 토픽: 타입의 #[dds(key)] 필드가 Instance를 가른다
    let topic = dp.create_topic::<TilePressure>(
        "TilePressure",
        "TilePressure",
        TOPIC_QOS_DEFAULT,
        None,
        StatusMask::NONE,
    )?;

    let writer = if let Some(profile) = &args.profile {
        factory.load_profiles(&[&args.profile_file])?;
        let qos = publisher.get_datawriter_qos_from_profile(profile)?;
        lab_log!("writer qos from profile {}", profile);
        publisher.create_datawriter::<TilePressure>(&topic, qos, None, StatusMask::NONE)?
    } else {
        let qos = DataWriterQos {
            reliability: ReliabilityQosPolicy {
                kind: ReliabilityQosPolicyKind::Reliable,
                max_blocking_time: ms(100),
            },
            writer_data_lifecycle: WriterDataLifecycleQosPolicy {
                autodispose_unregistered_instances: autodispose,
            },
            ..Default::default()
        };
        publisher.create_datawriter::<TilePressure>(&topic, qos, None, StatusMask::NONE)?
    };
    let ev_writer = if args.events {
        let ev_topic = dp.create_topic::<StepEvent>(
            "StepEvent",
            "StepEvent",
            TOPIC_QOS_DEFAULT,
            None,
            StatusMask::NONE,
        )?;
        Some(publisher.create_datawriter::<StepEvent>(
            &ev_topic,
            DATAWRITER_QOS_DEFAULT,
            None,
            StatusMask::NONE,
        )?)
    } else {
        None
    };
    lab_log!("pub ready: domain={} autodispose={} events={}", args.domain, autodispose, args.events);

    let mut matched = 0;
    if !args.no_wait {
        while !shutdown.is_stopped() && !lab::poll_writer_matched(&writer, &mut matched)? {
            shutdown.wait_timeout(100);
        }
    }

    // 1. register_instance: 샘플의 키로 InstanceHandle을 받아 둔다 (와이어로는 안 나간다)
    let mut handles = Vec::new();
    for &(row, col) in &TILES {
        let h = writer.register_instance(&TilePressure { row, col, pressure: 0.0 })?;
        lab_log!("register tile({},{}) handle={}", row, col, lab::hex4(&h));
        handles.push(h);
    }

    let mut alive = [true; 4];
    let mut c = 1;
    while !shutdown.is_stopped() {
        let spike = c as usize % TILES.len(); // 이번 주기에 밟힌 타일
        for (i, &(row, col)) in TILES.iter().enumerate() {
            if !alive[i] {
                continue;
            }
            let pressure = 10.0 * i as f32 + if i == spike { 50.0 } else { 0.0 };
            // 2. write: 샘플 안의 키로 Instance를 식별한다 → Reader에서 ALIVE
            writer.write(&TilePressure { row, col, pressure }, InstanceHandle::NIL)?;
            lab_log!("write tile({},{}) p={:.1}", row, col, pressure);

            if let (Some(ev), true) = (&ev_writer, i == spike) {
                let ts_ms = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as i64;
                ev.write(&StepEvent { row, col, ts_ms }, InstanceHandle::NIL)?;
            }
        }

        if c == args.cycles {
            // 3. dispose: 등록해 둔 핸들로 지정 → NOT_ALIVE_DISPOSED
            let (row, col) = TILES[0];
            writer.dispose(&TilePressure { row, col, pressure: 0.0 }, handles[0])?;
            alive[0] = false;
            lab_log!("dispose tile(1,1)");

            // 4. unregister_instance: 샘플의 키로 지정
            //    autodispose=1 → NOT_ALIVE_DISPOSED, 0 → NOT_ALIVE_NO_WRITERS
            let (row, col) = TILES[1];
            writer.unregister_instance(&TilePressure { row, col, pressure: 0.0 }, InstanceHandle::NIL)?;
            alive[1] = false;
            lab_log!("unregister tile(1,2) (autodispose={})", autodispose);
        }
        if c >= 2 * args.cycles && !args.hold {
            break;
        }

        lab::crash_check(args.crash_after);
        if shutdown.wait_timeout(args.period_ms) {
            break;
        }
        c += 1;
    }

    cleanup_participant(dp);
    Ok(())
}

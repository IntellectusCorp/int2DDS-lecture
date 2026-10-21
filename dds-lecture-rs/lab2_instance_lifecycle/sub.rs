//! 실습 2 · Active Floor Subscriber
//!   SampleInfo의 valid_data · view_state · instance_state를 찍는다.
//!   --domain N   --events (StepEvent 토픽도 구독)
use std::collections::HashMap;

use clap::Parser;
use int2dds::{
    common::instance_handle::InstanceHandle,
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
    floor::{StepEvent, TilePressure},
    lab::{self, ms},
    lab_log,
    shutdown::{cleanup_participant, Shutdown},
};

#[derive(Parser)]
struct Args {
    #[arg(long, default_value_t = 0)]
    domain: i32,
    #[arg(long)]
    events: bool,
}

/// 상태 알림(valid_data=false)에는 데이터가 없으니 핸들→타일 대응을 미리 배워 둔다
fn tile_name(known: &HashMap<[u8; 16], (i16, i16)>, h: &InstanceHandle) -> String {
    match known.get(h.value()) {
        Some((row, col)) => format!("tile({},{})", row, col),
        None => format!("handle {}", lab::hex4(h)),
    }
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
    let reader =
        subscriber.create_datareader::<TilePressure>(&topic, qos.clone(), None, StatusMask::NONE)?;
    let ev_reader = if args.events {
        let ev_topic = dp.create_topic::<StepEvent>(
            "StepEvent",
            "StepEvent",
            TOPIC_QOS_DEFAULT,
            None,
            StatusMask::NONE,
        )?;
        Some(subscriber.create_datareader::<StepEvent>(&ev_topic, qos, None, StatusMask::NONE)?)
    } else {
        None
    };
    lab_log!("sub ready: domain={} events={}", args.domain, args.events);

    let any = (
        [SampleStateKind::ANY_SAMPLE_STATE],
        [ViewStateKind::ANY_VIEW_STATE],
        [InstanceStateKind::ANY_INSTANCE_STATE],
    );
    let mut known = HashMap::new();
    while !shutdown.is_stopped() {
        let samples = match reader.take(1, &any.0, &any.1, &any.2) {
            Err(DdsError::NoData) => {
                if let Some(ev_reader) = &ev_reader {
                    while let Ok(events) = ev_reader.take(1, &any.0, &any.1, &any.2) {
                        for s in events.iter().filter(|s| s.sample_info().valid_data) {
                            let e = s.data()?;
                            lab_log!("EVENT step on tile({},{}) ts={}", e.row, e.col, e.ts_ms);
                        }
                    }
                }
                shutdown.wait_timeout(100);
                continue;
            }
            other => other?,
        };

        for s in &samples {
            let info = s.sample_info();
            if !info.valid_data {
                // dispose · unregister · Writer 소멸 알림: 데이터를 꺼내지 않는다
                lab_log!(
                    "{} -> {}  (valid_data=false)",
                    tile_name(&known, &info.instance_handle),
                    lab::instance_state(info.instance_state)
                );
                continue;
            }
            let t = s.data()?;
            known.insert(*info.instance_handle.value(), (t.row, t.col));
            lab_log!(
                "tile({},{}) p={:5.1}  view={} inst={}",
                t.row,
                t.col,
                t.pressure,
                lab::view_state(info.view_state),
                lab::instance_state(info.instance_state)
            );
        }
    }

    cleanup_participant(dp);
    Ok(())
}

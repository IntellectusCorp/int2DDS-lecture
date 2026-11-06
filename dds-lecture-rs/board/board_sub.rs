//! 보드 실습 · Display: board.xml의 Participant 트리로 Reader를 만들고 받은 상태를 LED 명령으로 낸다.
//!   board_sub | python3 led.py   (PC에서는 python led.py --text)
//!   타일 (1,1)(1,2)(2,1)(2,2) → LED 0~3
//!   led N on 밝기   ALIVE, 밝기 = pressure/100
//!   led N off       NOT_ALIVE_DISPOSED
//!   led N blink     NOT_ALIVE_NO_WRITERS (Writer가 사라짐 · LIVELINESS 만료)
//!   led N fast      REQUESTED_DEADLINE_MISSED (갱신이 끊김)
//!   --config F (board.xml)  --participant NAME (Board::Display)  --reader NAME (sub::tile)
use std::collections::HashMap;

use clap::Parser;
use int2dds::{
    common::instance_handle::InstanceHandle,
    core::error::{DdsError, DdsResult},
    domain::domain_participant_factory::DomainParticipantFactory,
    subscription::sample_info::{InstanceStateKind, SampleStateKind, ViewStateKind},
};
use lecture::{
    lab::{self, field},
    lab_log,
    shutdown::{cleanup_participant, Shutdown},
};

#[derive(Parser)]
struct Args {
    #[arg(long, default_value = "board.xml")]
    config: String,
    #[arg(long, default_value = "Board::Display")]
    participant: String,
    #[arg(long, default_value = "sub::tile")]
    reader: String,
}

fn led_of(row: i16, col: i16) -> Option<usize> {
    ((1..=2).contains(&row) && (1..=2).contains(&col)).then(|| ((row - 1) * 2 + (col - 1)) as usize)
}

fn main() -> DdsResult<()> {
    lab::init();
    let shutdown = Shutdown::install();
    let a = Args::parse();

    let factory = DomainParticipantFactory::get_instance();
    factory.load_profiles(&[&a.config])?;
    let app = factory.create_participant_from_config(&a.participant)?;
    let reader = app
        .datareader(&a.reader)
        .ok_or_else(|| DdsError::Error(format!("reader {} not found", a.reader)))?;

    let q = reader.get_qos()?;
    lab_log!(
        "sub ready ({}): reliability={:?} durability={:?} deadline={} ownership={:?} lease={}",
        a.config,
        q.reliability.kind,
        q.durability.kind,
        lab::dur(q.deadline.period),
        q.ownership.kind,
        lab::dur(q.liveliness.lease_duration)
    );

    // dispose 알림에는 키가 없어서 InstanceHandle로 LED를 찾는다. 값 = (LED, 마지막 수신 ms)
    let mut leds: HashMap<InstanceHandle, (usize, u128)> = HashMap::new();
    let deadline_ms = (!q.deadline.period.is_infinite()).then(|| q.deadline.period.as_millis() as u128);
    let (mut matched, mut missed, mut alive, mut incompatible) = (-1, 0, -1, 0);
    while !shutdown.is_stopped() {
        let st = reader.get_subscription_matched_status()?;
        if st.current_count() != matched {
            matched = st.current_count();
            lab_log!("matched writers={}", matched);
        }
        let st = reader.get_requested_deadline_missed_status()?;
        if st.total_count() != missed {
            missed = st.total_count();
            lab_log!("REQUESTED_DEADLINE_MISSED: total={}", missed);
            // 상태는 마지막 Instance 하나만 알려 주므로, period 동안 갱신이 없던 타일을 모두 표시
            let now = lab::elapsed_ms();
            for (i, last) in leds.values() {
                if deadline_ms.is_some_and(|p| now - last >= p) {
                    println!("led {} fast", i);
                }
            }
        }
        let st = reader.get_liveliness_changed_status()?;
        if st.alive_count() != alive {
            alive = st.alive_count();
            lab_log!("LIVELINESS_CHANGED: alive={} not_alive={}", alive, st.not_alive_count());
        }
        let st = reader.get_requested_incompatible_qos_status()?;
        if st.total_count() != incompatible {
            incompatible = st.total_count();
            lab_log!("REQUESTED_INCOMPATIBLE_QOS: last_policy={}", lab::policy_name(st.last_policy_id()));
        }

        let samples = match reader.take(
            16,
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
        for s in &samples {
            let info = s.sample_info();
            if info.valid_data {
                let d = s.data()?;
                let (row, col, p): (i16, i16, f32) =
                    (field(&d, "row")?, field(&d, "col")?, field(&d, "pressure")?);
                if let Some(i) = led_of(row, col) {
                    leds.insert(info.instance_handle, (i, lab::elapsed_ms()));
                    println!("led {} on {:.2}", i, (p / 100.0).clamp(0.0, 1.0));
                }
                continue;
            }
            let Some(&(i, _)) = leds.get(&info.instance_handle) else { continue };
            lab_log!("led {}: {}", i, lab::instance_state(info.instance_state));
            if info.instance_state == InstanceStateKind::NOT_ALIVE_DISPOSED_INSTANCE_STATE {
                println!("led {} off", i);
            } else {
                println!("led {} blink", i);
            }
        }
    }

    drop(reader);
    cleanup_participant(app.participant);
    Ok(())
}

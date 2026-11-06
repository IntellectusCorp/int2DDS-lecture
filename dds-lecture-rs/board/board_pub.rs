//! 보드 실습 · Sensor: board.xml의 Participant 트리로 Writer를 만들고 타일을 쓴다.
//!   QoS · Partition · Domain은 전부 board.xml에서 바꾼다. 인자는 무엇을 쓸지만 정한다.
//!   --config F (board.xml)  --participant NAME (Board::Sensor)  --writer NAME (pub::tile)
//!   --tiles 11,12,21,22   쓸 타일 (행열 두 자리)
//!   --pressure P          LED 밝기가 되는 값 (0~100)
//!   --count N             N번 쓰고 나서 멈추고 Ctrl+C까지 남는다 (0 = 계속 쓴다)
//!   --dispose-after S     S초 뒤 타일을 dispose하고 남는다
//!   --crash-after S       S초 뒤 정리 없이 종료 (전원을 뽑은 것과 같다)
use clap::Parser;
use int2dds::{
    common::instance_handle::InstanceHandle,
    config::xml::XmlTypeRegistry,
    core::error::{DdsError, DdsResult},
    domain::domain_participant_factory::DomainParticipantFactory,
    xtypes::DynamicData,
};
use lecture::{
    lab::{self, split_list},
    lab_log,
    shutdown::{cleanup_participant, Shutdown},
};

#[derive(Parser)]
struct Args {
    #[arg(long, default_value = "board.xml")]
    config: String,
    #[arg(long, default_value = "Board::Sensor")]
    participant: String,
    #[arg(long, default_value = "pub::tile")]
    writer: String,
    #[arg(long, default_value = "11,12,21,22")]
    tiles: String,
    #[arg(long, default_value_t = 80.0)]
    pressure: f32,
    #[arg(long, default_value_t = 500)]
    period_ms: u64,
    #[arg(long, default_value_t = 0)]
    count: u32,
    #[arg(long, default_value_t = 0)]
    dispose_after: u64,
    #[arg(long, default_value_t = 0)]
    crash_after: u64,
}

fn tile(proto: &DynamicData, row: i16, col: i16, pressure: f32) -> DdsResult<DynamicData> {
    let mut d = proto.clone();
    for r in [d.set("row", row), d.set("col", col), d.set("pressure", pressure)] {
        r.map_err(|e| DdsError::Error(e.to_string()))?;
    }
    Ok(d)
}

fn main() -> DdsResult<()> {
    lab::init();
    let shutdown = Shutdown::install();
    let a = Args::parse();
    let tiles: Vec<(i16, i16)> = split_list(Some(&a.tiles))
        .iter()
        .filter_map(|t| Some((t.get(0..1)?.parse().ok()?, t.get(1..2)?.parse().ok()?)))
        .collect();

    // 1. 구성 파일 하나로 타입 · QoS · Participant 트리를 만든다
    let factory = DomainParticipantFactory::get_instance();
    factory.load_profiles(&[&a.config])?;
    let app = factory.create_participant_from_config(&a.participant)?;
    let writer = app
        .datawriter(&a.writer)
        .ok_or_else(|| DdsError::Error(format!("writer {} not found", a.writer)))?;
    // 쓸 샘플의 틀도 같은 파일의 <types>에서
    let proto = XmlTypeRegistry::from_file(&a.config)?.get("TilePressure")?.create_data();

    let q = writer.get_qos()?;
    lab_log!(
        "pub ready ({}): reliability={:?} durability={:?} deadline={} ownership={:?} strength={} lease={}",
        a.config,
        q.reliability.kind,
        q.durability.kind,
        lab::dur(q.deadline.period),
        q.ownership.kind,
        q.ownership_strength.value,
        lab::dur(q.liveliness.lease_duration)
    );

    let (mut matched, mut written, mut disposed) = (0, 0, false);
    while !shutdown.is_stopped() {
        lab::poll_writer_matched(&writer, &mut matched)?;
        if a.dispose_after > 0 && !disposed && lab::elapsed_ms() >= a.dispose_after as u128 * 1000 {
            for &(row, col) in &tiles {
                writer.dispose(&tile(&proto, row, col, 0.0)?, InstanceHandle::NIL)?;
                lab_log!("dispose tile({},{})", row, col);
            }
            disposed = true;
        }
        if !disposed && (a.count == 0 || written < a.count) {
            for &(row, col) in &tiles {
                writer.write(&tile(&proto, row, col, a.pressure)?, InstanceHandle::NIL)?;
            }
            written += 1;
            lab_log!("write #{} tiles={} p={:.0}", written, tiles.len(), a.pressure);
            if written == a.count {
                lab_log!("count reached: stop writing (Ctrl+C to exit)");
            }
        }
        lab::crash_check(a.crash_after);
        shutdown.wait_timeout(a.period_ms);
    }

    drop(writer);
    cleanup_participant(app.participant);
    Ok(())
}

//! 실습 8 · 3단계: 타입·QoS·Domain·Participant 트리를 app.xml 하나로. 코드에 남는 건 이름뿐이다.
//!   --config FILE  (기본 app.xml)  --reader NAME (기본 sub::tile)
use clap::Parser;
use int2dds::{
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
    #[arg(long, default_value = concat!(env!("CARGO_MANIFEST_DIR"), "/lab8_xml/app.xml"))]
    config: String,
    #[arg(long, default_value = "sub::tile")]
    reader: String,
}

fn main() -> DdsResult<()> {
    lab::init();
    let shutdown = Shutdown::install();
    let args = Args::parse();

    // 1. 구성 파일 로드: 타입 · QoS · Domain · Participant 선언이 한 번에
    let factory = DomainParticipantFactory::get_instance();
    factory.load_profiles(&[&args.config])?;

    // 2. 이름으로 트리 생성
    let app = factory.create_participant_from_config("Apps::Monitor")?;

    // 3. 선언한 이름으로 Reader를 꺼낸다
    let reader = app
        .datareader(&args.reader)
        .ok_or_else(|| DdsError::Error(format!("reader {} not found", args.reader)))?;
    let q = reader.get_qos()?;
    lab_log!(
        "sub ready (config): reader={} reliability={:?} durability={:?}",
        args.reader,
        q.reliability.kind,
        q.durability.kind
    );

    let mut next_status = 0;
    while !shutdown.is_stopped() {
        if lab::elapsed_ms() >= next_status {
            lab_log!("matched writers={}", reader.get_subscription_matched_status()?.current_count());
            next_status = lab::elapsed_ms() + 1000;
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
        for s in &samples {
            let info = s.sample_info();
            if !info.valid_data {
                lab_log!("{} (valid_data=false)", lab::instance_state(info.instance_state));
                continue;
            }
            let d = s.data()?;
            let (row, col, p): (i16, i16, f32) =
                (field(&d, "row")?, field(&d, "col")?, field(&d, "pressure")?);
            lab_log!("tile({},{}) p={:5.1}", row, col, p);
        }
    }

    // 4. 해제: 트리의 Participant를 꺼내 정리한다
    drop(reader);
    cleanup_participant(app.participant);
    Ok(())
}

//! 실습 8 · 1단계: 타입만 XML로. IDL·코드 생성 없이 실습 2의 lab2_pub와 통신한다.
//!   --xml FILE   (기본 tile.xml; tile_lab.xml + --type lab::TilePressure 로 이름 불일치 실험)
//!   --type NAME  --topic NAME  --domain N
use std::sync::Arc;

use clap::Parser;
use int2dds::{
    config::xml::XmlTypeRegistry,
    core::error::{DdsError, DdsResult},
    domain::{domain_participant_factory::DomainParticipantFactory, qos::PARTICIPANT_QOS_DEFAULT},
    infrastructure::status::StatusMask,
    subscription::{
        data_sample::DataSample,
        qos::{DATAREADER_QOS_DEFAULT, SUBSCRIBER_QOS_DEFAULT},
        sample_info::{InstanceStateKind, SampleStateKind, ViewStateKind},
    },
    topic::qos::TOPIC_QOS_DEFAULT,
    xtypes::DynamicData,
};
use lecture::{
    lab::{self, field},
    lab_log,
    shutdown::{cleanup_participant, Shutdown},
};

#[derive(Parser)]
struct Args {
    #[arg(long, default_value_t = 0)]
    domain: i32,
    #[arg(long, default_value = concat!(env!("CARGO_MANIFEST_DIR"), "/lab8_xml/tile.xml"))]
    xml: String,
    #[arg(long, default_value = "TilePressure")]
    r#type: String,
    #[arg(long, default_value = "TilePressure")]
    topic: String,
}

fn print_sample(s: &DataSample<DynamicData>) -> DdsResult<()> {
    let info = s.sample_info();
    if !info.valid_data {
        lab_log!("{} (valid_data=false)", lab::instance_state(info.instance_state));
        return Ok(());
    }
    let d = s.data()?;
    let (row, col, p): (i16, i16, f32) = (field(&d, "row")?, field(&d, "col")?, field(&d, "pressure")?);
    lab_log!(
        "tile({},{}) p={:5.1}  view={} inst={}",
        row,
        col,
        p,
        lab::view_state(info.view_state),
        lab::instance_state(info.instance_state)
    );
    Ok(())
}

fn main() -> DdsResult<()> {
    lab::init();
    let shutdown = Shutdown::install();
    let args = Args::parse();

    // 1. XML 로드 → 타입 지원 객체 (int2dds-idl 단계가 없다)
    let registry = XmlTypeRegistry::from_file(&args.xml)?;
    let type_support = Arc::new(registry.get(&args.r#type)?);

    let factory = DomainParticipantFactory::get_instance();
    let dp = factory.create_participant(args.domain, PARTICIPANT_QOS_DEFAULT, None, StatusMask::NONE)?;
    let subscriber = dp.create_subscriber(SUBSCRIBER_QOS_DEFAULT, None, StatusMask::NONE)?;

    // 2. _dynamic 계열로 Topic · Reader 생성
    let topic =
        dp.create_topic_dynamic(&args.topic, type_support.clone(), TOPIC_QOS_DEFAULT, None, StatusMask::NONE)?;
    let reader = subscriber.create_datareader_dynamic(
        &topic,
        type_support,
        DATAREADER_QOS_DEFAULT,
        None,
        StatusMask::NONE,
    )?;
    lab_log!("sub ready (xml type): type={} topic={} domain={}", args.r#type, args.topic, args.domain);

    // 3. 필드 이름으로 읽기
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
            print_sample(s)?;
        }
    }

    cleanup_participant(dp);
    Ok(())
}

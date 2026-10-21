//! 실습 8 · 2단계: QoS도 XML로. Team::HardState를 기본 프로파일로 지정해 기본 QoS Reader가 받게 한다.
//!   pub 쪽도 같은 프로파일이어야 한다(VOLATILE Writer는 TRANSIENT_LOCAL Reader와 매칭되지 않는다):
//!     lab2_pub --profile Team::HardState --hold   를 먼저, 이 sub를 늦게 띄운다.
//!   --profile-file FILE  --xml FILE  --domain N
use std::sync::Arc;

use clap::Parser;
use int2dds::{
    common::env::set_default_qos_profile,
    config::xml::XmlTypeRegistry,
    core::error::{DdsError, DdsResult},
    domain::{domain_participant_factory::DomainParticipantFactory, qos::PARTICIPANT_QOS_DEFAULT},
    infrastructure::status::StatusMask,
    subscription::{
        qos::{DATAREADER_QOS_DEFAULT, SUBSCRIBER_QOS_DEFAULT},
        sample_info::{InstanceStateKind, SampleStateKind, ViewStateKind},
    },
    topic::qos::TOPIC_QOS_DEFAULT,
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
    #[arg(long, default_value = concat!(env!("CARGO_MANIFEST_DIR"), "/lab8_xml/team_qos.xml"))]
    profile_file: String,
    #[arg(long, default_value = concat!(env!("CARGO_MANIFEST_DIR"), "/lab8_xml/tile.xml"))]
    xml: String,
}

fn main() -> DdsResult<()> {
    lab::init();
    let shutdown = Shutdown::install();
    let args = Args::parse();

    // 첫 Participant 생성 전에 프로파일을 로드하고 기본 프로파일을 지정한다
    let factory = DomainParticipantFactory::get_instance();
    factory.load_profiles(&[&args.profile_file])?;
    set_default_qos_profile("Team::HardState");

    let registry = XmlTypeRegistry::from_file(&args.xml)?;
    let type_support = Arc::new(registry.get("TilePressure")?);

    let dp = factory.create_participant(args.domain, PARTICIPANT_QOS_DEFAULT, None, StatusMask::NONE)?;
    let subscriber = dp.create_subscriber(SUBSCRIBER_QOS_DEFAULT, None, StatusMask::NONE)?;
    let topic =
        dp.create_topic_dynamic("TilePressure", type_support.clone(), TOPIC_QOS_DEFAULT, None, StatusMask::NONE)?;
    // *_QOS_DEFAULT → 기본 프로파일
    let reader = subscriber.create_datareader_dynamic(
        &topic,
        type_support,
        DATAREADER_QOS_DEFAULT,
        None,
        StatusMask::NONE,
    )?;
    lab_log!("sub ready (xml type + profile): domain={}", args.domain);
    let q = reader.get_qos()?;
    lab_log!(
        "effective reader qos: reliability={:?} durability={:?} history={:?}",
        q.reliability.kind,
        q.durability.kind,
        q.history.kind
    );

    while !shutdown.is_stopped() {
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
            lab_log!("tile({},{}) p={:5.1}  view={}", row, col, p, lab::view_state(info.view_state));
        }
    }

    cleanup_participant(dp);
    Ok(())
}

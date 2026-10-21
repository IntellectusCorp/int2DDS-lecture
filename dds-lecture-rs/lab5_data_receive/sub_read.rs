//! 실습 5 · read와 take, 상태 마스크 (실습 2 sub 기반, pub은 lab2_pub)
//!   1초마다 도착한 샘플에 대해 다섯 단계를 순서대로 실행한다.
//!     1 read(NOT_READ) 반복   → 도착한 샘플 전부
//!     2 read(NOT_READ) 한 번  → NO_DATA (전부 READ가 됐다)
//!     3 read(READ) 한 번      → 첫 샘플이 sample_state=READ로 다시 온다
//!     4 take(ANY) 반복        → 꺼내면서 Cache에서 제거
//!     5 read(ANY) 한 번       → NO_DATA (take 뒤엔 비어 있다)
//!   --domain N
use clap::Parser;
use int2dds::{
    core::error::{DdsError, DdsResult},
    domain::{domain_participant_factory::DomainParticipantFactory, qos::PARTICIPANT_QOS_DEFAULT},
    infrastructure::{
        qos_policy::{
            HistoryQosPolicy, HistoryQosPolicyKind, ReliabilityQosPolicy, ReliabilityQosPolicyKind,
        },
        status::StatusMask,
    },
    subscription::{
        data_reader::DataReader,
        qos::{DataReaderQos, SUBSCRIBER_QOS_DEFAULT},
        sample_info::{InstanceStateKind, SampleStateKind, ViewStateKind},
    },
    topic::qos::TOPIC_QOS_DEFAULT,
};
use lecture::{
    floor::TilePressure,
    lab::{self, ms},
    lab_log,
    shutdown::{cleanup_participant, Shutdown},
};

#[derive(Parser)]
struct Args {
    #[arg(long, default_value_t = 0)]
    domain: i32,
}

/// single: 한 번만 호출. read는 상태를 바꾸지 않으니 READ/ANY 마스크 read를 반복하면 같은 샘플만 계속 온다
fn drain(
    reader: &DataReader<TilePressure>,
    label: &str,
    take: bool,
    states: SampleStateKind,
    single: bool,
) -> DdsResult<usize> {
    let (views, insts) = ([ViewStateKind::ANY_VIEW_STATE], [InstanceStateKind::ANY_INSTANCE_STATE]);
    let mut count = 0;
    loop {
        let result = if take {
            reader.take(1, &[states], &views, &insts)
        } else {
            reader.read(1, &[states], &views, &insts)
        };
        let samples = match result {
            Err(DdsError::NoData) => break,
            other => other?,
        };
        for s in &samples {
            count += 1;
            let info = s.sample_info();
            if info.valid_data {
                let t = s.data()?;
                lab_log!(
                    "  {:<22} tile({},{}) p={:5.1}  sample_state={}",
                    label,
                    t.row,
                    t.col,
                    t.pressure,
                    lab::sample_state(info.sample_state)
                );
            } else {
                lab_log!("  {:<22} {} (valid_data=false)", label, lab::instance_state(info.instance_state));
            }
        }
        if single {
            break;
        }
    }
    if count == 0 {
        lab_log!("  {:<22} NO_DATA", label);
    }
    Ok(count)
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
        history: HistoryQosPolicy { kind: HistoryQosPolicyKind::KeepLast(8), ..Default::default() },
        ..Default::default()
    };
    let reader = subscriber.create_datareader::<TilePressure>(&topic, qos, None, StatusMask::NONE)?;
    lab_log!("sub ready: domain={}", args.domain);

    use SampleStateKind as S;
    while !shutdown.wait_timeout(1000) {
        if drain(&reader, "1 read(NOT_READ)", false, S::NOT_READ_SAMPLE_STATE, false)? == 0 {
            continue;
        }
        drain(&reader, "2 read(NOT_READ) again", false, S::NOT_READ_SAMPLE_STATE, true)?;
        drain(&reader, "3 read(READ)", false, S::READ_SAMPLE_STATE, true)?;
        drain(&reader, "4 take(ANY)", true, S::ANY_SAMPLE_STATE, false)?;
        drain(&reader, "5 read(ANY)", false, S::ANY_SAMPLE_STATE, true)?;
        lab_log!("----");
    }

    cleanup_participant(dp);
    Ok(())
}

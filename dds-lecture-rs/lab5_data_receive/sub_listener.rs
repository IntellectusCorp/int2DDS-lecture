//! 실습 5 · Listener 버전 (실습 2 sub → 콜백)
//!   콜백은 RTPS 수신 스레드에서 동기 호출된다: 안에서는 take만, 처리는 짧게.
//!   --slow-ms N   on_data_available 안에서 N ms 잠들어 수신이 밀리는 것을 본다
//!   --domain N
use std::sync::{
    atomic::{AtomicI32, Ordering},
    Arc,
};
use std::time::Duration;

use clap::Parser;
use int2dds::{
    core::error::DdsResult,
    domain::{domain_participant_factory::DomainParticipantFactory, qos::PARTICIPANT_QOS_DEFAULT},
    infrastructure::{
        qos_policy::{ReliabilityQosPolicy, ReliabilityQosPolicyKind},
        status::{StatusMask, SubscriptionMatchedStatus},
    },
    subscription::{
        data_reader::DataReader,
        data_reader_listener::DataReaderListener,
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
    #[arg(long, default_value_t = 0)]
    slow_ms: u64,
}

struct App {
    received: AtomicI32,
    slow_ms: u64,
}

impl DataReaderListener for App {
    type Foo = TilePressure;

    fn on_data_available(&self, reader: &DataReader<TilePressure>) {
        while let Ok(samples) = reader.take(
            1,
            &[SampleStateKind::ANY_SAMPLE_STATE],
            &[ViewStateKind::ANY_VIEW_STATE],
            &[InstanceStateKind::ANY_INSTANCE_STATE],
        ) {
            for s in &samples {
                let n = self.received.fetch_add(1, Ordering::Relaxed) + 1;
                match s.data() {
                    Ok(t) if s.sample_info().valid_data => lab_log!(
                        "on_data_available #{}: tile({},{}) p={:5.1}",
                        n,
                        t.row,
                        t.col,
                        t.pressure
                    ),
                    _ => lab_log!(
                        "on_data_available #{}: {}",
                        n,
                        lab::instance_state(s.sample_info().instance_state)
                    ),
                }
            }
        }
        if self.slow_ms > 0 {
            lab_log!("  callback sleeping {} ms (receive thread blocked)", self.slow_ms);
            std::thread::sleep(Duration::from_millis(self.slow_ms));
        }
    }

    fn on_subscription_matched(
        &self,
        _reader: &DataReader<TilePressure>,
        st: &SubscriptionMatchedStatus,
    ) {
        lab_log!(
            "on_subscription_matched: current_count={} ({:+}) total={}",
            st.current_count(),
            st.current_count_change(),
            st.total_count()
        );
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

    // 구현하지 않은 콜백은 trait의 빈 기본 구현이 쓰인다
    let app = Arc::new(App { received: AtomicI32::new(0), slow_ms: args.slow_ms });
    let mask = StatusMask::DATA_AVAILABLE | StatusMask::SUBSCRIPTION_MATCHED;
    let _reader = subscriber.create_datareader::<TilePressure>(&topic, qos, Some(app), mask)?;
    lab_log!("sub ready (listener): domain={} slow_ms={}", args.domain, args.slow_ms);

    shutdown.wait(); // 메인 스레드는 할 일이 없다: Ctrl+C까지 블로킹

    cleanup_participant(dp);
    Ok(())
}

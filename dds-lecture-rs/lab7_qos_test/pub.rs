//! 실습 7 · 레시피 Publisher (실습 1 pub + recipes.rs)
//!   --recipe hard_state|health|failover
//!     A hard_state: --depth N | --keep-all      + --count 10 --hold  (먼저 띄우고 sub를 늦게)
//!     B health:     --deadline-ms N --lease-ms N + --pause-after S [--assert]
//!     C failover:   --strength N --lease-ms N   + --name A|B   (같은 실행 파일 두 개)
//!   --count N(0=무한) --period-ms N --hold --crash-after S --domain N --topic NAME
use std::sync::Arc;

use clap::Parser;
use int2dds::{
    common::instance_handle::InstanceHandle,
    core::error::DdsResult,
    domain::{domain_participant_factory::DomainParticipantFactory, qos::PARTICIPANT_QOS_DEFAULT},
    infrastructure::status::{
        LivelinessLostStatus, OfferedDeadlineMissedStatus, PublicationMatchedStatus, StatusMask,
    },
    publication::{
        data_writer::DataWriter,
        data_writer_listener::DataWriterListener,
        qos::{DataWriterQos, PUBLISHER_QOS_DEFAULT},
    },
    topic::qos::TOPIC_QOS_DEFAULT,
};
use lecture::{
    hello::HelloWorld,
    lab, lab_log, recipes,
    shutdown::{cleanup_participant, Shutdown},
};

#[derive(Parser)]
struct Args {
    #[arg(long, default_value_t = 0)]
    domain: i32,
    #[arg(long, default_value = "hello_world_topic")]
    topic: String,
    #[arg(long, default_value = "none")]
    recipe: String,
    #[arg(long, default_value = "A")]
    name: String,
    #[arg(long, default_value_t = 0)]
    count: i32,
    #[arg(long, default_value_t = 500)]
    period_ms: u64,
    #[arg(long, default_value_t = 1)]
    depth: i32,
    #[arg(long)]
    keep_all: bool,
    #[arg(long, default_value_t = 500)]
    deadline_ms: i64,
    #[arg(long, default_value_t = 2000)]
    lease_ms: i64,
    #[arg(long, default_value_t = 100)]
    strength: i32,
    #[arg(long, default_value_t = 0)]
    pause_after: u64,
    #[arg(long)]
    assert: bool,
    #[arg(long)]
    hold: bool,
    #[arg(long, default_value_t = 0)]
    crash_after: u64,
}

struct Listener;

impl DataWriterListener for Listener {
    type Foo = HelloWorld;

    fn on_publication_matched(&self, _w: &DataWriter<HelloWorld>, st: &PublicationMatchedStatus) {
        lab_log!(
            "PUBLICATION_MATCHED: current_count={} ({:+})",
            st.current_count(),
            st.current_count_change()
        );
    }

    fn on_offered_deadline_missed(
        &self,
        _w: &DataWriter<HelloWorld>,
        st: &OfferedDeadlineMissedStatus,
    ) {
        lab_log!("OFFERED_DEADLINE_MISSED: total={}", st.total_count());
    }

    fn on_liveliness_lost(&self, _w: &DataWriter<HelloWorld>, st: &LivelinessLostStatus) {
        lab_log!("LIVELINESS_LOST: total={}", st.total_count());
    }
}

fn main() -> DdsResult<()> {
    lab::init();
    let shutdown = Shutdown::install();
    let a = Args::parse();

    let factory = DomainParticipantFactory::get_instance();
    let dp = factory.create_participant(a.domain, PARTICIPANT_QOS_DEFAULT, None, StatusMask::NONE)?;
    let publisher = dp.create_publisher(PUBLISHER_QOS_DEFAULT, None, StatusMask::NONE)?;
    let topic = dp.create_topic::<HelloWorld>(
        &a.topic,
        "demo::HelloWorld",
        TOPIC_QOS_DEFAULT,
        None,
        StatusMask::NONE,
    )?;

    let mut qos = DataWriterQos::default();
    match a.recipe.as_str() {
        "hard_state" => recipes::hard_state_writer(&mut qos, a.depth, a.keep_all),
        "health" => recipes::health_writer(&mut qos, a.deadline_ms, a.lease_ms),
        "failover" => recipes::failover_writer(&mut qos, a.strength, a.lease_ms),
        _ => {}
    }

    let mask = StatusMask::PUBLICATION_MATCHED
        | StatusMask::OFFERED_DEADLINE_MISSED
        | StatusMask::LIVELINESS_LOST;
    let writer = publisher.create_datawriter::<HelloWorld>(&topic, qos, Some(Arc::new(Listener)), mask)?;
    lab_log!(
        "pub {} ready: recipe={} depth={} keep_all={} deadline={} lease={} strength={}",
        a.name,
        a.recipe,
        a.depth,
        a.keep_all,
        a.deadline_ms,
        a.lease_ms,
        a.strength
    );

    let mut paused = false;
    let mut i = 1;
    while !shutdown.is_stopped() && (a.count == 0 || i <= a.count) {
        if !paused && a.pause_after > 0 && lab::elapsed_ms() >= a.pause_after as u128 * 1000 {
            paused = true;
            lab_log!(
                "paused: no more write{}",
                if a.assert { " (assert_liveliness continues)" } else { "" }
            );
        }
        if paused {
            if a.assert {
                writer.assert_liveliness()?;
            }
            lab::crash_check(a.crash_after);
            shutdown.wait_timeout(200);
            continue;
        }
        writer.write(&HelloWorld { index: i, message: a.name.clone() }, InstanceHandle::NIL)?;
        if a.period_ms >= 100 {
            lab_log!("write index={}", i);
        }
        i += 1;
        lab::crash_check(a.crash_after);
        if shutdown.wait_timeout(a.period_ms) {
            break;
        }
    }
    while a.hold && !shutdown.is_stopped() {
        lab::crash_check(a.crash_after);
        shutdown.wait_timeout(200);
    }

    cleanup_participant(dp);
    Ok(())
}

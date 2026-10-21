//! 실습 6 · 일부러 안 맞추기 (Writer, 실습 1 pub + QoS 인자)
//!   --reliability reliable|best_effort   --deadline-ms N   --ownership shared|exclusive [--strength N]
//!   --fix-deadline-after S --fix-deadline-ms N   런타임 set_qos로 DEADLINE 수정 (변경 가능 정책)
//!   --try-immutable                              런타임에 RELIABILITY 변경 시도 → ImmutablePolicy
//!   --period-ms N --count N(0=무한) --domain N --topic NAME
use std::sync::Arc;

use clap::Parser;
use int2dds::{
    common::instance_handle::InstanceHandle,
    core::error::DdsResult,
    domain::{domain_participant_factory::DomainParticipantFactory, qos::PARTICIPANT_QOS_DEFAULT},
    infrastructure::{
        qos_policy::{
            DeadlineQosPolicy, OwnershipQosPolicy, OwnershipStrengthQosPolicy, ReliabilityQosPolicy,
            ReliabilityQosPolicyKind,
        },
        status::{
            OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus, PublicationMatchedStatus,
            StatusMask,
        },
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
    lab::{self, ms},
    lab_log,
    shutdown::{cleanup_participant, Shutdown},
};

#[derive(Parser)]
struct Args {
    #[arg(long, default_value_t = 0)]
    domain: i32,
    #[arg(long, default_value = "hello_world_topic")]
    topic: String,
    #[arg(long, default_value_t = 500)]
    period_ms: u64,
    #[arg(long, default_value_t = 0)]
    count: i32,
    #[arg(long, default_value = "reliable")]
    reliability: String,
    #[arg(long, default_value_t = 0)]
    deadline_ms: i64,
    #[arg(long, default_value = "shared")]
    ownership: String,
    #[arg(long, default_value_t = 0)]
    strength: i32,
    #[arg(long, default_value_t = 0)]
    fix_deadline_after: u64,
    #[arg(long, default_value_t = 0)]
    fix_deadline_ms: i64,
    #[arg(long)]
    try_immutable: bool,
}

struct Listener;

impl DataWriterListener for Listener {
    type Foo = HelloWorld;

    fn on_offered_incompatible_qos(
        &self,
        _w: &DataWriter<HelloWorld>,
        st: &OfferedIncompatibleQosStatus,
    ) {
        lab_log!(
            "OFFERED_INCOMPATIBLE_QOS: last_policy={} total={}",
            lab::policy_name(st.last_policy_id()),
            st.total_count()
        );
    }

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
}

fn main() -> DdsResult<()> {
    lab::init();
    let shutdown = Shutdown::install();
    let args = Args::parse();

    let factory = DomainParticipantFactory::get_instance();
    let dp = factory.create_participant(args.domain, PARTICIPANT_QOS_DEFAULT, None, StatusMask::NONE)?;
    let publisher = dp.create_publisher(PUBLISHER_QOS_DEFAULT, None, StatusMask::NONE)?;
    let topic = dp.create_topic::<HelloWorld>(
        &args.topic,
        "demo::HelloWorld",
        TOPIC_QOS_DEFAULT,
        None,
        StatusMask::NONE,
    )?;

    let mut qos = DataWriterQos {
        reliability: ReliabilityQosPolicy {
            kind: lab::reliability_kind(&args.reliability),
            max_blocking_time: ms(100),
        },
        ownership: OwnershipQosPolicy { kind: lab::ownership_kind(&args.ownership) },
        ownership_strength: OwnershipStrengthQosPolicy { value: args.strength },
        ..Default::default()
    };
    if args.deadline_ms > 0 {
        qos.deadline = DeadlineQosPolicy { period: ms(args.deadline_ms) };
    }

    let mask = StatusMask::OFFERED_INCOMPATIBLE_QOS
        | StatusMask::PUBLICATION_MATCHED
        | StatusMask::OFFERED_DEADLINE_MISSED;
    let writer = publisher.create_datawriter::<HelloWorld>(&topic, qos, Some(Arc::new(Listener)), mask)?;
    lab_log!(
        "pub ready: reliability={} deadline={} ms ownership={} strength={}",
        args.reliability,
        args.deadline_ms,
        args.ownership,
        args.strength
    );

    let mut next_status = 0;
    let (mut fixed, mut tried) = (false, false);
    let mut i = 1;
    while !shutdown.is_stopped() && (args.count == 0 || i <= args.count) {
        writer.write(&HelloWorld { index: i, message: "hello".into() }, InstanceHandle::NIL)?;

        if lab::elapsed_ms() >= next_status {
            let pm = writer.get_publication_matched_status()?;
            let oq = writer.get_offered_incompatible_qos_status()?;
            lab_log!(
                "write index={} | matched={} incompatible_qos total={} last={}",
                i,
                pm.current_count(),
                oq.total_count(),
                lab::policy_name(oq.last_policy_id())
            );
            next_status = lab::elapsed_ms() + 1000;
        }

        if args.fix_deadline_after > 0
            && !fixed
            && lab::elapsed_ms() >= args.fix_deadline_after as u128 * 1000
        {
            let mut q = writer.get_qos()?;
            q.deadline.period = ms(args.fix_deadline_ms);
            lab_log!("set_qos(deadline={} ms) -> {:?}", args.fix_deadline_ms, writer.set_qos(q));
            fixed = true;
        }
        if args.try_immutable && !tried && lab::elapsed_ms() >= 3000 {
            let mut q = writer.get_qos()?;
            q.reliability.kind = match q.reliability.kind {
                ReliabilityQosPolicyKind::Reliable => ReliabilityQosPolicyKind::BestEffort,
                ReliabilityQosPolicyKind::BestEffort => ReliabilityQosPolicyKind::Reliable,
            };
            lab_log!("set_qos(reliability change) -> {:?}", writer.set_qos(q));
            tried = true;
        }
        if shutdown.wait_timeout(args.period_ms) {
            break;
        }
        i += 1;
    }

    cleanup_participant(dp);
    Ok(())
}

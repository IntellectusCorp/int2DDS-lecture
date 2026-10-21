//! 실습 공통 헬퍼: 시간, 상태 이름, 매칭 확인
use std::sync::OnceLock;
use std::time::Instant;

use int2dds::{
    common::instance_handle::InstanceHandle,
    core::{
        error::{DdsError, DdsResult},
        time::Duration,
    },
    infrastructure::qos_policy::{
        DurabilityQosPolicyKind, LivelinessQosPolicyKind, OwnershipQosPolicyKind, QosPolicyId,
        ReliabilityQosPolicyKind,
    },
    publication::data_writer::DataWriter,
    subscription::sample_info::{InstanceStateKind, SampleStateKind, ViewStateKind},
    xtypes::{DynamicData, FromDynamicValue},
};

static T0: OnceLock<Instant> = OnceLock::new();

/// 시작 시각 기록 (Ctrl+C 처리는 shutdown)
pub fn init() {
    T0.get_or_init(Instant::now);
}

pub fn elapsed_ms() -> u128 {
    T0.get_or_init(Instant::now).elapsed().as_millis()
}

/// 경과 시간(ms)을 앞에 붙여 출력
#[macro_export]
macro_rules! lab_log {
    ($($arg:tt)*) => {
        println!("[{:6} ms] {}", $crate::lab::elapsed_ms(), format_args!($($arg)*))
    };
}

/// ms → QoS Duration
pub fn ms(x: i64) -> Duration {
    Duration::from_millis(x)
}

pub fn instance_state(s: InstanceStateKind) -> &'static str {
    if s == InstanceStateKind::ALIVE_INSTANCE_STATE {
        "ALIVE"
    } else if s == InstanceStateKind::NOT_ALIVE_DISPOSED_INSTANCE_STATE {
        "NOT_ALIVE_DISPOSED"
    } else if s == InstanceStateKind::NOT_ALIVE_NO_WRITERS_INSTANCE_STATE {
        "NOT_ALIVE_NO_WRITERS"
    } else {
        "?"
    }
}

pub fn view_state(s: ViewStateKind) -> &'static str {
    if s == ViewStateKind::NEW_VIEW_STATE {
        "NEW"
    } else {
        "NOT_NEW"
    }
}

pub fn sample_state(s: SampleStateKind) -> &'static str {
    if s == SampleStateKind::READ_SAMPLE_STATE {
        "READ"
    } else {
        "NOT_READ"
    }
}

pub fn policy_name(id: QosPolicyId) -> &'static str {
    match id {
        QosPolicyId::Durability => "DURABILITY",
        QosPolicyId::Deadline => "DEADLINE",
        QosPolicyId::Ownership => "OWNERSHIP",
        QosPolicyId::Liveliness => "LIVELINESS",
        QosPolicyId::Partition => "PARTITION",
        QosPolicyId::Reliability => "RELIABILITY",
        QosPolicyId::DestinationOrder => "DESTINATION_ORDER",
        QosPolicyId::History => "HISTORY",
        QosPolicyId::DataRepresentation => "DATA_REPRESENTATION",
        QosPolicyId::Invalid => "(none)",
        _ => "?",
    }
}

/// InstanceHandle 앞 4바이트를 hex로
pub fn hex4(h: &InstanceHandle) -> String {
    let v = h.value();
    format!("{:02x}{:02x}{:02x}{:02x}", v[0], v[1], v[2], v[3])
}

pub fn reliability_kind(s: &str) -> ReliabilityQosPolicyKind {
    if s == "best_effort" {
        ReliabilityQosPolicyKind::BestEffort
    } else {
        ReliabilityQosPolicyKind::Reliable
    }
}

pub fn durability_kind(s: &str) -> DurabilityQosPolicyKind {
    if s == "transient_local" {
        DurabilityQosPolicyKind::TransientLocal
    } else {
        DurabilityQosPolicyKind::Volatile
    }
}

pub fn ownership_kind(s: &str) -> OwnershipQosPolicyKind {
    if s == "exclusive" {
        OwnershipQosPolicyKind::Exclusive
    } else {
        OwnershipQosPolicyKind::Shared
    }
}

pub fn liveliness_kind(s: &str) -> LivelinessQosPolicyKind {
    match s {
        "manual_by_topic" => LivelinessQosPolicyKind::ManualByTopic,
        "manual_by_participant" => LivelinessQosPolicyKind::ManualByParticipant,
        _ => LivelinessQosPolicyKind::Automatic,
    }
}

/// publication_matched.current_count가 바뀌면 기록. 매칭된 Reader가 있으면 true
pub fn poll_writer_matched<T: Clone + 'static>(writer: &DataWriter<T>, last: &mut i32) -> DdsResult<bool> {
    let st = writer.get_publication_matched_status()?;
    if st.current_count() != *last {
        crate::lab_log!(
            "publication_matched: current_count {} -> {} (total {})",
            *last,
            st.current_count(),
            st.total_count()
        );
        *last = st.current_count();
    }
    Ok(st.current_count() > 0)
}

/// --crash-after N초: 정리 없이 즉시 종료 → SPDP dispose가 나가지 않는다
pub fn crash_check(crash_after_s: u64) {
    if crash_after_s > 0 && elapsed_ms() >= crash_after_s as u128 * 1000 {
        crate::lab_log!("simulated crash: exit without cleanup");
        std::process::exit(3);
    }
}

/// DynamicData 필드를 이름으로 읽는다. 없는 이름이면 에러
pub fn field<T: FromDynamicValue>(d: &DynamicData, name: &str) -> DdsResult<T> {
    d.get(name).map_err(|e| DdsError::Error(e.to_string()))
}

/// "a,b,c" → 목록
pub fn split_list(csv: Option<&str>) -> Vec<String> {
    csv.map(|s| s.split(',').filter(|p| !p.is_empty()).map(String::from).collect())
        .unwrap_or_default()
}

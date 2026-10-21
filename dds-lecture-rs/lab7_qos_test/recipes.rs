//! 실습 7 · 관용구 3종을 QoS 설정 함수로
use int2dds::{
    infrastructure::qos_policy::{
        DeadlineQosPolicy, DurabilityQosPolicy, DurabilityQosPolicyKind, HistoryQosPolicy,
        HistoryQosPolicyKind, LivelinessQosPolicy, LivelinessQosPolicyKind, OwnershipQosPolicy,
        OwnershipQosPolicyKind, OwnershipStrengthQosPolicy, ReliabilityQosPolicy,
        ReliabilityQosPolicyKind,
    },
    publication::qos::DataWriterQos,
    subscription::qos::DataReaderQos,
};
use crate::lab::ms;

fn history(depth: i32, keep_all: bool) -> HistoryQosPolicy {
    let kind = if keep_all { HistoryQosPolicyKind::KeepAll } else { HistoryQosPolicyKind::KeepLast(depth) };
    HistoryQosPolicy { kind, ..Default::default() }
}

/// 동작 1 · Late Joiner가 최신 상태를 받는다: RELIABLE + TRANSIENT_LOCAL + HISTORY
/// depth=1이면 인스턴스별 최신 1개만 남는다. 키 없는 토픽에서 index=1부터 보려면 depth≥count 또는 KEEP_ALL
pub fn hard_state_writer(q: &mut DataWriterQos, depth: i32, keep_all: bool) {
    q.reliability = ReliabilityQosPolicy { kind: ReliabilityQosPolicyKind::Reliable, max_blocking_time: ms(100) };
    q.durability = DurabilityQosPolicy { kind: DurabilityQosPolicyKind::TransientLocal };
    q.history = history(depth, keep_all);
}

pub fn hard_state_reader(q: &mut DataReaderQos, depth: i32, keep_all: bool) {
    q.reliability = ReliabilityQosPolicy { kind: ReliabilityQosPolicyKind::Reliable, max_blocking_time: ms(0) };
    q.durability = DurabilityQosPolicy { kind: DurabilityQosPolicyKind::TransientLocal };
    // Reader Cache도 같은 깊이여야 밀려온 과거 샘플을 take 전에 잃지 않는다
    q.history = history(depth, keep_all);
}

/// 동작 2 · 헬스 모니터링: DEADLINE(갱신 주기) + LIVELINESS MANUAL_BY_TOPIC(Writer 생존)
pub fn health_writer(q: &mut DataWriterQos, deadline_ms: i64, lease_ms: i64) {
    q.deadline = DeadlineQosPolicy { period: ms(deadline_ms) };
    q.liveliness = LivelinessQosPolicy {
        kind: LivelinessQosPolicyKind::ManualByTopic,
        lease_duration: ms(lease_ms),
    };
}

pub fn health_reader(q: &mut DataReaderQos, deadline_ms: i64, lease_ms: i64) {
    q.deadline = DeadlineQosPolicy { period: ms(deadline_ms) };
    q.liveliness = LivelinessQosPolicy {
        kind: LivelinessQosPolicyKind::ManualByTopic,
        lease_duration: ms(lease_ms),
    };
}

/// 동작 3 · Failover: EXCLUSIVE + strength + AUTOMATIC lease (프로세스 장애 감지)
pub fn failover_writer(q: &mut DataWriterQos, strength: i32, lease_ms: i64) {
    q.ownership = OwnershipQosPolicy { kind: OwnershipQosPolicyKind::Exclusive };
    q.ownership_strength = OwnershipStrengthQosPolicy { value: strength };
    q.liveliness = LivelinessQosPolicy {
        kind: LivelinessQosPolicyKind::Automatic,
        lease_duration: ms(lease_ms),
    };
}

pub fn failover_reader(q: &mut DataReaderQos, lease_ms: i64) {
    q.ownership = OwnershipQosPolicy { kind: OwnershipQosPolicyKind::Exclusive };
    q.liveliness = LivelinessQosPolicy {
        kind: LivelinessQosPolicyKind::Automatic,
        lease_duration: ms(lease_ms),
    };
}

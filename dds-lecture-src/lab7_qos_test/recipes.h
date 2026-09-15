/* 실습 7 · E: 관용구 3종을 QoS 설정 함수로 (동작 1·2·3 = 55·57·59장) */
#ifndef RECIPES_H
#define RECIPES_H
#include "lab.h"

/* 동작 1 · Late Joiner가 최신 상태를 받는다: RELIABLE + TRANSIENT_LOCAL + HISTORY
 * depth=1이면 인스턴스별 최신 1개만 남는다. 키 없는 토픽에서 index=1부터 보려면 depth≥count 또는 KEEP_ALL */
static inline void recipe_hard_state_writer(Int2DdsDataWriterQos *q, int depth, bool keep_all) {
    CHECK(int2dds_datawriter_qos_set_reliability(q, INT2DDS_QOS_RELIABILITY_RELIABLE, MS(100)));
    CHECK(int2dds_datawriter_qos_set_durability(q, INT2DDS_QOS_DURABILITY_TRANSIENT_LOCAL));
    CHECK(int2dds_datawriter_qos_set_history(q, keep_all ? INT2DDS_QOS_HISTORY_KEEP_ALL : INT2DDS_QOS_HISTORY_KEEP_LAST, depth));
}
static inline void recipe_hard_state_reader(Int2DdsDataReaderQos *q, int depth, bool keep_all) {
    CHECK(int2dds_datareader_qos_set_reliability(q, INT2DDS_QOS_RELIABILITY_RELIABLE, 0));
    CHECK(int2dds_datareader_qos_set_durability(q, INT2DDS_QOS_DURABILITY_TRANSIENT_LOCAL));
    /* Reader Cache도 같은 깊이여야 밀려온 과거 샘플을 take 전에 잃지 않는다 */
    CHECK(int2dds_datareader_qos_set_history(q, keep_all ? INT2DDS_QOS_HISTORY_KEEP_ALL : INT2DDS_QOS_HISTORY_KEEP_LAST, depth));
}

/* 동작 2 · 헬스 모니터링: DEADLINE(갱신 주기) + LIVELINESS MANUAL_BY_TOPIC(Writer 생존) */
static inline void recipe_health_writer(Int2DdsDataWriterQos *q, int deadline_ms, int lease_ms) {
    CHECK(int2dds_datawriter_qos_set_deadline(q, MS(deadline_ms)));
    CHECK(int2dds_datawriter_qos_set_liveliness(q, INT2DDS_QOS_LIVELINESS_MANUAL_BY_TOPIC, MS(lease_ms)));
}
static inline void recipe_health_reader(Int2DdsDataReaderQos *q, int deadline_ms, int lease_ms) {
    CHECK(int2dds_datareader_qos_set_deadline(q, MS(deadline_ms)));
    CHECK(int2dds_datareader_qos_set_liveliness(q, INT2DDS_QOS_LIVELINESS_MANUAL_BY_TOPIC, MS(lease_ms)));
}

/* 동작 3 · Failover: EXCLUSIVE + strength + AUTOMATIC lease (프로세스 장애 감지) */
static inline void recipe_failover_writer(Int2DdsDataWriterQos *q, int strength, int lease_ms) {
    CHECK(int2dds_datawriter_qos_set_ownership(q, INT2DDS_QOS_OWNERSHIP_EXCLUSIVE));
    CHECK(int2dds_datawriter_qos_set_ownership_strength(q, strength));
    CHECK(int2dds_datawriter_qos_set_liveliness(q, INT2DDS_QOS_LIVELINESS_AUTOMATIC, MS(lease_ms)));
}
static inline void recipe_failover_reader(Int2DdsDataReaderQos *q, int lease_ms) {
    CHECK(int2dds_datareader_qos_set_ownership(q, INT2DDS_QOS_OWNERSHIP_EXCLUSIVE));
    CHECK(int2dds_datareader_qos_set_liveliness(q, INT2DDS_QOS_LIVELINESS_AUTOMATIC, MS(lease_ms)));
}

#endif

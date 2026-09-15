/* 실습 6 · 일부러 안 맞추기 (Writer, 실습 1 pub + QoS 인자)
 *   --reliability reliable|best_effort   --deadline-ms N   --ownership shared|exclusive [--strength N]
 *   --fix-deadline-after S --fix-deadline-ms N   런타임 set_qos로 DEADLINE 수정 (변경 가능 정책)
 *   --try-immutable                              런타임에 RELIABILITY 변경 시도 → IMMUTABLE_POLICY
 *   --period-ms N --count N(0=무한) --domain N --topic NAME */
#include "lab.h"
#include "shutdown.h"
#include "hello.h"

static void on_bad_qos(Int2DdsDataWriter *w, const Int2DdsOfferedIncompatibleQosStatus *st,
                       Int2DdsUserContext ctx) {
    (void)w;
    (void)ctx;
    lab_log("OFFERED_INCOMPATIBLE_QOS: last_policy=%s total=%d", lab_policy_name(st->last_policy_id),
            st->total_count);
}
static void on_matched(Int2DdsDataWriter *w, const Int2DdsPublicationMatchedStatus *st,
                       Int2DdsUserContext ctx) {
    (void)w;
    (void)ctx;
    lab_log("PUBLICATION_MATCHED: current_count=%d (%+d)", st->current_count, st->current_count_change);
}
static void on_deadline(Int2DdsDataWriter *w, const Int2DdsOfferedDeadlineMissedStatus *st,
                        Int2DdsUserContext ctx) {
    (void)w;
    (void)ctx;
    lab_log("OFFERED_DEADLINE_MISSED: total=%d", st->total_count);
}

int main(int argc, char **argv) {
    lab_init();
    shutdown_install();
    int domain = arg_int(argc, argv, "--domain", 0);
    const char *topic_name = arg_str(argc, argv, "--topic", "hello_world_topic");
    int period_ms = arg_int(argc, argv, "--period-ms", 500);
    int count = arg_int(argc, argv, "--count", 0);
    const char *reliability = arg_str(argc, argv, "--reliability", "reliable");
    int deadline_ms = arg_int(argc, argv, "--deadline-ms", 0);
    const char *ownership = arg_str(argc, argv, "--ownership", "shared");
    int strength = arg_int(argc, argv, "--strength", 0);
    int fix_after = arg_int(argc, argv, "--fix-deadline-after", 0);
    int fix_ms = arg_int(argc, argv, "--fix-deadline-ms", 0);
    bool try_immutable = arg_flag(argc, argv, "--try-immutable");

    Int2DdsParticipantFactory *factory;
    Int2DdsParticipant *dp;
    Int2DdsPublisher *pub;
    Int2DdsTopic *topic;
    Int2DdsDataWriterQos *qos;
    Int2DdsDataWriter *writer;

    CHECK(int2dds_domain_participant_factory_get_instance(&factory));
    CHECK(int2dds_create_participant(factory, domain, NULL, &dp));
    CHECK(int2dds_create_publisher(dp, NULL, &pub));
    CHECK(HelloWorld_create_topic(dp, topic_name, NULL, &topic));

    CHECK(int2dds_datawriter_qos_create_default(&qos));
    CHECK(int2dds_datawriter_qos_set_reliability(qos, lab_reliability_kind(reliability), MS(100)));
    if (deadline_ms > 0) CHECK(int2dds_datawriter_qos_set_deadline(qos, MS(deadline_ms)));
    CHECK(int2dds_datawriter_qos_set_ownership(qos, lab_ownership_kind(ownership)));
    if (strength) CHECK(int2dds_datawriter_qos_set_ownership_strength(qos, strength));

    Int2DdsDataWriterListener l = {0};
    l.on_offered_incompatible_qos = on_bad_qos;
    l.on_publication_matched = on_matched;
    l.on_offered_deadline_missed = on_deadline;
    uint32_t mask = INT2DDS_STATUS_OFFERED_INCOMPATIBLE_QOS | INT2DDS_STATUS_PUBLICATION_MATCHED |
                    INT2DDS_STATUS_OFFERED_DEADLINE_MISSED;
    CHECK(int2dds_create_datawriter(pub, topic, qos, &l, mask, &writer));
    int2dds_datawriter_qos_destroy(qos);
    lab_log("pub ready: reliability=%s deadline=%d ms ownership=%s strength=%d", reliability, deadline_ms,
            ownership, strength);

    uint8_t buf[4096];
    HelloWorld sample = {0};
    int64_t next_status = 0;
    bool fixed = false, tried = false;
    for (int i = 1; !shutdown_is_stopped() && (count == 0 || i <= count); i++) {
        sample.index = i;
        snprintf(sample.message, sizeof sample.message, "hello");
        size_t n = HelloWorld_serialize_for(writer, &sample, buf, sizeof buf);
        CHECK(int2dds_datawriter_write_serialized(writer, buf, n));

        if (now_ms() >= next_status) {
            Int2DdsPublicationMatchedStatus pm;
            Int2DdsOfferedIncompatibleQosStatus oq;
            CHECK(int2dds_datawriter_get_publication_matched_status(writer, &pm));
            CHECK(int2dds_datawriter_get_offered_incompatible_qos_status(writer, &oq));
            lab_log("write index=%d | matched=%d incompatible_qos total=%d last=%s", i, pm.current_count,
                    oq.total_count, lab_policy_name(oq.last_policy_id));
            next_status = now_ms() + 1000;
        }

        if (fix_after > 0 && !fixed && now_ms() - g_t0 >= (int64_t)fix_after * 1000) {
            Int2DdsDataWriterQos *q;
            CHECK(int2dds_datawriter_get_qos(writer, &q));
            CHECK(int2dds_datawriter_qos_set_deadline(q, MS(fix_ms)));
            Int2DdsRet r = int2dds_datawriter_set_qos(writer, q);
            int2dds_datawriter_qos_destroy(q);
            lab_log("set_qos(deadline=%d ms) -> %s", fix_ms, lab_ret_name(r));
            fixed = true;
        }
        if (try_immutable && !tried && now_ms() - g_t0 >= 3000) {
            Int2DdsDataWriterQos *q;
            CHECK(int2dds_datawriter_get_qos(writer, &q));
            int32_t other = lab_reliability_kind(reliability) == INT2DDS_QOS_RELIABILITY_RELIABLE
                                ? INT2DDS_QOS_RELIABILITY_BEST_EFFORT
                                : INT2DDS_QOS_RELIABILITY_RELIABLE;
            CHECK(int2dds_datawriter_qos_set_reliability(q, other, MS(100)));
            Int2DdsRet r = int2dds_datawriter_set_qos(writer, q);
            int2dds_datawriter_qos_destroy(q);
            lab_log("set_qos(reliability change) -> %s (%d)", lab_ret_name(r), (int)r);
            tried = true;
        }
        if (shutdown_wait_timeout(period_ms)) break;
    }

    cleanup_participant(dp);
    return 0;
}

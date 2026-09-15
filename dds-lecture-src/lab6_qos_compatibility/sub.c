/* 실습 6 · 일부러 안 맞추기 (Reader, 실습 1 sub + QoS 인자)
 *   --reliability reliable|best_effort   --deadline-ms N   --ownership shared|exclusive
 *   --domain N --topic NAME */
#include "lab.h"
#include "shutdown.h"
#include "hello.h"

static void on_bad_qos(Int2DdsDataReader *r, const Int2DdsRequestedIncompatibleQosStatus *st,
                       Int2DdsUserContext ctx) {
    (void)r;
    (void)ctx;
    lab_log("REQUESTED_INCOMPATIBLE_QOS: last_policy=%s total=%d", lab_policy_name(st->last_policy_id),
            st->total_count);
}
static void on_matched(Int2DdsDataReader *r, const Int2DdsSubscriptionMatchedStatus *st,
                       Int2DdsUserContext ctx) {
    (void)r;
    (void)ctx;
    lab_log("SUBSCRIPTION_MATCHED: current_count=%d (%+d)", st->current_count, st->current_count_change);
}
static void on_deadline(Int2DdsDataReader *r, const Int2DdsRequestedDeadlineMissedStatus *st,
                        Int2DdsUserContext ctx) {
    (void)r;
    (void)ctx;
    lab_log("REQUESTED_DEADLINE_MISSED: total=%d", st->total_count);
}

int main(int argc, char **argv) {
    lab_init();
    shutdown_install();
    int domain = arg_int(argc, argv, "--domain", 0);
    const char *topic_name = arg_str(argc, argv, "--topic", "hello_world_topic");
    const char *reliability = arg_str(argc, argv, "--reliability", "reliable");
    int deadline_ms = arg_int(argc, argv, "--deadline-ms", 0);
    const char *ownership = arg_str(argc, argv, "--ownership", "shared");

    Int2DdsParticipantFactory *factory;
    Int2DdsParticipant *dp;
    Int2DdsSubscriber *sub;
    Int2DdsTopic *topic;
    Int2DdsDataReaderQos *qos;
    Int2DdsDataReader *reader;

    CHECK(int2dds_domain_participant_factory_get_instance(&factory));
    CHECK(int2dds_create_participant(factory, domain, NULL, &dp));
    CHECK(int2dds_create_subscriber(dp, NULL, &sub));
    CHECK(HelloWorld_create_topic(dp, topic_name, NULL, &topic));

    CHECK(int2dds_datareader_qos_create_default(&qos));
    CHECK(int2dds_datareader_qos_set_reliability(qos, lab_reliability_kind(reliability), 0));
    if (deadline_ms > 0) CHECK(int2dds_datareader_qos_set_deadline(qos, MS(deadline_ms)));
    CHECK(int2dds_datareader_qos_set_ownership(qos, lab_ownership_kind(ownership)));

    Int2DdsDataReaderListener l = {0};
    l.on_requested_incompatible_qos = on_bad_qos;
    l.on_subscription_matched = on_matched;
    l.on_requested_deadline_missed = on_deadline;
    uint32_t mask = INT2DDS_STATUS_REQUESTED_INCOMPATIBLE_QOS | INT2DDS_STATUS_SUBSCRIPTION_MATCHED |
                    INT2DDS_STATUS_REQUESTED_DEADLINE_MISSED;
    CHECK(int2dds_create_datareader(sub, topic, qos, &l, mask, &reader));
    int2dds_datareader_qos_destroy(qos);
    lab_log("sub ready: reliability=%s deadline=%d ms ownership=%s", reliability, deadline_ms, ownership);

    uint8_t buf[4096];
    uintptr_t size;
    bool valid_data;
    int64_t next_status = 0;
    while (!shutdown_is_stopped()) {
        if (now_ms() >= next_status) {
            Int2DdsSubscriptionMatchedStatus sm;
            Int2DdsRequestedIncompatibleQosStatus rq;
            CHECK(int2dds_datareader_get_subscription_matched_status(reader, &sm));
            CHECK(int2dds_datareader_get_requested_incompatible_qos_status(reader, &rq));
            lab_log("status | matched=%d incompatible_qos total=%d last=%s", sm.current_count, rq.total_count,
                    lab_policy_name(rq.last_policy_id));
            next_status = now_ms() + 1000;
        }
        Int2DdsRet ret = int2dds_datareader_take_serialized(reader, buf, sizeof buf, &size, &valid_data);
        if (ret == INT2DDS_RET_NO_DATA) {
            shutdown_wait_timeout(100);
            continue;
        }
        if (ret != INT2DDS_RET_OK) break;
        HelloWorld sample = {0};
        if (valid_data && HelloWorld_deserialize_cdr(buf, size, &sample))
            lab_log("recv index=%d", sample.index);
    }

    cleanup_participant(dp);
    return 0;
}

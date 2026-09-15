/* 실습 7 · 레시피 Subscriber (실습 1 sub + recipes.h + 상태 콜백)
 *   --recipe hard_state|health|failover  (+ --depth N | --keep-all, --deadline-ms N, --lease-ms N)
 *   --tbf-ms N   TIME_BASED_FILTER minimum_separation (D)
 *   --rate       샘플별 출력 대신 초당 수신 수 (D)
 *   --gaps       index가 건너뛰면 표시 (KEEP_LAST(1) + 빠른 쓰기)
 *   --domain N --topic NAME */
#include "recipes.h"
#include "shutdown.h"
#include "hello.h"

static void on_matched(Int2DdsDataReader *r, const Int2DdsSubscriptionMatchedStatus *st, Int2DdsUserContext ctx) {
    (void)r;
    (void)ctx;
    lab_log("SUBSCRIPTION_MATCHED: current_count=%d (%+d)", st->current_count, st->current_count_change);
}
static void on_deadline(Int2DdsDataReader *r, const Int2DdsRequestedDeadlineMissedStatus *st, Int2DdsUserContext ctx) {
    (void)r;
    (void)ctx;
    char hx[9];
    lab_hex4(st->last_instance_handle, hx);
    lab_log("REQUESTED_DEADLINE_MISSED: total=%d instance=%s", st->total_count, hx);
}
static void on_liveliness(Int2DdsDataReader *r, const Int2DdsLivelinessChangedStatus *st, Int2DdsUserContext ctx) {
    (void)r;
    (void)ctx;
    lab_log("LIVELINESS_CHANGED: alive=%d (%+d) not_alive=%d (%+d)", st->alive_count, st->alive_count_change,
            st->not_alive_count, st->not_alive_count_change);
}
static void on_bad_qos(Int2DdsDataReader *r, const Int2DdsRequestedIncompatibleQosStatus *st, Int2DdsUserContext ctx) {
    (void)r;
    (void)ctx;
    lab_log("REQUESTED_INCOMPATIBLE_QOS: last_policy=%s", lab_policy_name(st->last_policy_id));
}

int main(int argc, char **argv) {
    lab_init();
    shutdown_install();
    int domain = arg_int(argc, argv, "--domain", 0);
    const char *topic_name = arg_str(argc, argv, "--topic", "hello_world_topic");
    const char *recipe = arg_str(argc, argv, "--recipe", "none");
    int depth = arg_int(argc, argv, "--depth", 1);
    bool keep_all = arg_flag(argc, argv, "--keep-all");
    int deadline_ms = arg_int(argc, argv, "--deadline-ms", 1000);
    int lease_ms = arg_int(argc, argv, "--lease-ms", 2000);
    int tbf_ms = arg_int(argc, argv, "--tbf-ms", 0);
    bool rate = arg_flag(argc, argv, "--rate");
    bool gaps = arg_flag(argc, argv, "--gaps");

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
    if (strcmp(recipe, "hard_state") == 0) recipe_hard_state_reader(qos, depth, keep_all);
    else if (strcmp(recipe, "health") == 0) recipe_health_reader(qos, deadline_ms, lease_ms);
    else if (strcmp(recipe, "failover") == 0) recipe_failover_reader(qos, lease_ms);
    if (tbf_ms > 0) CHECK(int2dds_datareader_qos_set_time_based_filter(qos, MS(tbf_ms)));

    Int2DdsDataReaderListener l = {0};
    l.on_subscription_matched = on_matched;
    l.on_requested_deadline_missed = on_deadline;
    l.on_liveliness_changed = on_liveliness;
    l.on_requested_incompatible_qos = on_bad_qos;
    uint32_t mask = INT2DDS_STATUS_SUBSCRIPTION_MATCHED | INT2DDS_STATUS_REQUESTED_DEADLINE_MISSED |
                    INT2DDS_STATUS_LIVELINESS_CHANGED | INT2DDS_STATUS_REQUESTED_INCOMPATIBLE_QOS;
    CHECK(int2dds_create_datareader(sub, topic, qos, &l, mask, &reader));
    int2dds_datareader_qos_destroy(qos);
    lab_log("sub ready: recipe=%s depth=%d keep_all=%d deadline=%d lease=%d tbf=%d", recipe, depth, keep_all,
            deadline_ms, lease_ms, tbf_ms);

    uint8_t buf[4096];
    uintptr_t size;
    Int2DdsSampleInfo info;
    int last_index = -1, per_sec = 0;
    int64_t next_rate = now_ms() + 1000;
    while (!shutdown_is_stopped()) {
        if (rate && now_ms() >= next_rate) {
            lab_log("rate: %d samples/s", per_sec);
            per_sec = 0;
            next_rate += 1000;
        }
        Int2DdsRet ret = int2dds_datareader_take_serialized_w_info(reader, buf, sizeof buf, &size, &info);
        if (ret == INT2DDS_RET_NO_DATA) {
            shutdown_wait_timeout(rate ? 5 : 50);
            continue;
        }
        if (ret != INT2DDS_RET_OK) break;
        if (!info.valid_data) {
            lab_log("%s (valid_data=false)", lab_instance_state(info.instance_state));
            continue;
        }
        HelloWorld sample = {0};
        if (!HelloWorld_deserialize_cdr(buf, size, &sample)) continue;
        per_sec++;
        if (gaps && last_index >= 0 && sample.index != last_index + 1)
            lab_log("GAP: expected %d got %d", last_index + 1, sample.index);
        last_index = sample.index;
        if (!rate) lab_log("recv index=%d from=%s", sample.index, sample.message);
    }

    cleanup_participant(dp);
    return 0;
}

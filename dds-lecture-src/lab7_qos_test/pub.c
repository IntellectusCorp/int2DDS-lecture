/* 실습 7 · 레시피 Publisher (실습 1 pub + recipes.h)
 *   --recipe hard_state|health|failover
 *     A hard_state: --depth N | --keep-all      + --count 10 --hold  (먼저 띄우고 sub를 늦게)
 *     B health:     --deadline-ms N --lease-ms N + --pause-after S [--assert]
 *     C failover:   --strength N --lease-ms N   + --name A|B   (같은 실행 파일 두 개)
 *   --count N(0=무한) --period-ms N --hold --crash-after S --domain N --topic NAME */
#include "recipes.h"
#include "shutdown.h"
#include "hello.h"

static void on_matched(Int2DdsDataWriter *w, const Int2DdsPublicationMatchedStatus *st, Int2DdsUserContext ctx) {
    (void)w;
    (void)ctx;
    lab_log("PUBLICATION_MATCHED: current_count=%d (%+d)", st->current_count, st->current_count_change);
}
static void on_deadline(Int2DdsDataWriter *w, const Int2DdsOfferedDeadlineMissedStatus *st, Int2DdsUserContext ctx) {
    (void)w;
    (void)ctx;
    lab_log("OFFERED_DEADLINE_MISSED: total=%d", st->total_count);
}
static void on_lost(Int2DdsDataWriter *w, const Int2DdsLivelinessLostStatus *st, Int2DdsUserContext ctx) {
    (void)w;
    (void)ctx;
    lab_log("LIVELINESS_LOST: total=%d", st->total_count);
}

int main(int argc, char **argv) {
    lab_init();
    shutdown_install();
    int domain = arg_int(argc, argv, "--domain", 0);
    const char *topic_name = arg_str(argc, argv, "--topic", "hello_world_topic");
    const char *recipe = arg_str(argc, argv, "--recipe", "none");
    const char *name = arg_str(argc, argv, "--name", "A");
    int count = arg_int(argc, argv, "--count", 0);
    int period_ms = arg_int(argc, argv, "--period-ms", 500);
    int depth = arg_int(argc, argv, "--depth", 1);
    bool keep_all = arg_flag(argc, argv, "--keep-all");
    int deadline_ms = arg_int(argc, argv, "--deadline-ms", 500);
    int lease_ms = arg_int(argc, argv, "--lease-ms", 2000);
    int strength = arg_int(argc, argv, "--strength", 100);
    int pause_after = arg_int(argc, argv, "--pause-after", 0);
    bool assert_lv = arg_flag(argc, argv, "--assert");
    bool hold = arg_flag(argc, argv, "--hold");
    int crash_after = arg_int(argc, argv, "--crash-after", 0);

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
    if (strcmp(recipe, "hard_state") == 0) recipe_hard_state_writer(qos, depth, keep_all);
    else if (strcmp(recipe, "health") == 0) recipe_health_writer(qos, deadline_ms, lease_ms);
    else if (strcmp(recipe, "failover") == 0) recipe_failover_writer(qos, strength, lease_ms);

    Int2DdsDataWriterListener l = {0};
    l.on_publication_matched = on_matched;
    l.on_offered_deadline_missed = on_deadline;
    l.on_liveliness_lost = on_lost;
    uint32_t mask = INT2DDS_STATUS_PUBLICATION_MATCHED | INT2DDS_STATUS_OFFERED_DEADLINE_MISSED |
                    INT2DDS_STATUS_LIVELINESS_LOST;
    CHECK(int2dds_create_datawriter(pub, topic, qos, &l, mask, &writer));
    int2dds_datawriter_qos_destroy(qos);
    lab_log("pub %s ready: recipe=%s depth=%d keep_all=%d deadline=%d lease=%d strength=%d", name, recipe,
            depth, keep_all, deadline_ms, lease_ms, strength);

    uint8_t buf[4096];
    HelloWorld sample = {0};
    bool paused = false;
    for (int i = 1; !shutdown_is_stopped() && (count == 0 || i <= count);) {
        if (!paused && pause_after > 0 && now_ms() - g_t0 >= (int64_t)pause_after * 1000) {
            paused = true;
            lab_log("paused: no more write%s", assert_lv ? " (assert_liveliness continues)" : "");
        }
        if (paused) {
            if (assert_lv) CHECK(int2dds_datawriter_assert_liveliness(writer));
            lab_crash_check(crash_after);
            shutdown_wait_timeout(200);
            continue;
        }
        sample.index = i;
        snprintf(sample.message, sizeof sample.message, "%s", name);
        size_t n = HelloWorld_serialize_for(writer, &sample, buf, sizeof buf);
        CHECK(int2dds_datawriter_write_serialized(writer, buf, n));
        if (period_ms >= 100) lab_log("write index=%d", i);
        i++;
        lab_crash_check(crash_after);
        if (shutdown_wait_timeout(period_ms)) break;
    }
    while (hold && !shutdown_is_stopped()) {
        lab_crash_check(crash_after);
        shutdown_wait_timeout(200);
    }

    cleanup_participant(dp);
    return 0;
}

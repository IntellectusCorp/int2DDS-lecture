/* 실습 5 · Listener 버전 (실습 2 sub → 콜백)
 *   콜백은 RTPS 수신 스레드에서 동기 호출된다: 안에서는 take만, 처리는 짧게.
 *   --slow-ms N   on_data_available 안에서 N ms 잠들어 수신이 밀리는 것을 본다
 *   --domain N */
#include "lab.h"
#include "shutdown.h"
#include "floor.h"

struct app {
    int received;
    int slow_ms;
};

static void on_data(Int2DdsDataReader *reader, Int2DdsUserContext ctx) {
    struct app *a = ctx;
    uint8_t buf[4096];
    uintptr_t n;
    Int2DdsSampleInfo info;
    TilePressure t;
    while (int2dds_datareader_take_serialized_w_info(reader, buf, sizeof buf, &n, &info) == INT2DDS_RET_OK) {
        a->received++;
        if (info.valid_data && TilePressure_deserialize_cdr(buf, n, &t))
            lab_log("on_data_available #%d: tile(%d,%d) p=%5.1f", a->received, t.row, t.col, t.pressure);
        else
            lab_log("on_data_available #%d: %s", a->received, lab_instance_state(info.instance_state));
    }
    if (a->slow_ms > 0) {
        lab_log("  callback sleeping %d ms (receive thread blocked)", a->slow_ms);
        sleep_ms(a->slow_ms);
    }
}

static void on_matched(Int2DdsDataReader *reader, const Int2DdsSubscriptionMatchedStatus *st,
                       Int2DdsUserContext ctx) {
    (void)reader;
    (void)ctx;
    lab_log("on_subscription_matched: current_count=%d (%+d) total=%d", st->current_count,
            st->current_count_change, st->total_count);
}

int main(int argc, char **argv) {
    lab_init();
    shutdown_install();
    int domain = arg_int(argc, argv, "--domain", 0);
    struct app app = {0, arg_int(argc, argv, "--slow-ms", 0)};

    Int2DdsParticipantFactory *factory;
    Int2DdsParticipant *dp;
    Int2DdsSubscriber *sub;
    Int2DdsTopic *topic;
    Int2DdsDataReaderQos *qos;
    Int2DdsDataReader *reader;

    CHECK(int2dds_domain_participant_factory_get_instance(&factory));
    CHECK(int2dds_create_participant(factory, domain, NULL, &dp));
    CHECK(int2dds_create_subscriber(dp, NULL, &sub));
    CHECK(TilePressure_create_topic(dp, "TilePressure", NULL, &topic));
    CHECK(int2dds_datareader_qos_create_default(&qos));
    CHECK(int2dds_datareader_qos_set_reliability(qos, INT2DDS_QOS_RELIABILITY_RELIABLE, 0));

    Int2DdsDataReaderListener l = {0}; /* 안 쓰는 콜백은 NULL */
    l.on_data_available = on_data;
    l.on_subscription_matched = on_matched;
    l.user_context = &app;
    uint32_t mask = INT2DDS_STATUS_DATA_AVAILABLE | INT2DDS_STATUS_SUBSCRIPTION_MATCHED;
    CHECK(int2dds_create_datareader(sub, topic, qos, &l, mask, &reader));
    int2dds_datareader_qos_destroy(qos);
    lab_log("sub ready (listener): domain=%d slow_ms=%d", domain, app.slow_ms);

    shutdown_wait(); /* 메인 스레드는 할 일이 없다: Ctrl+C까지 블로킹 */

    cleanup_participant(dp);
    return 0;
}

/* 실습 2 · Active Floor Subscriber
 *   SampleInfo의 valid_data · view_state · instance_state를 찍는다.
 *   --domain N   --events (StepEvent 토픽도 구독) */
#include "lab.h"
#include "shutdown.h"
#include "floor.h"

/* 상태 알림(valid_data=false)에는 데이터가 없으니 핸들→타일 대응을 미리 배워 둔다 */
static struct { uint8_t h[16]; int16_t row, col; } known[64];
static int nknown = 0;

static void remember(const uint8_t h[16], int16_t row, int16_t col) {
    for (int i = 0; i < nknown; i++)
        if (memcmp(known[i].h, h, 16) == 0) return;
    if (nknown < 64) {
        memcpy(known[nknown].h, h, 16);
        known[nknown].row = row;
        known[nknown].col = col;
        nknown++;
    }
}
static const char *tile_name(const uint8_t h[16]) {
    static char name[32];
    for (int i = 0; i < nknown; i++)
        if (memcmp(known[i].h, h, 16) == 0) {
            snprintf(name, sizeof name, "tile(%d,%d)", known[i].row, known[i].col);
            return name;
        }
    char hx[9];
    lab_hex4(h, hx);
    snprintf(name, sizeof name, "handle %s", hx);
    return name;
}

int main(int argc, char **argv) {
    lab_init();
    shutdown_install();
    int domain = arg_int(argc, argv, "--domain", 0);
    bool events = arg_flag(argc, argv, "--events");

    Int2DdsParticipantFactory *factory;
    Int2DdsParticipant *dp;
    Int2DdsSubscriber *sub;
    Int2DdsTopic *topic, *ev_topic = NULL;
    Int2DdsDataReaderQos *qos;
    Int2DdsDataReader *reader, *ev_reader = NULL;

    CHECK(int2dds_domain_participant_factory_get_instance(&factory));
    CHECK(int2dds_create_participant(factory, domain, NULL, &dp));
    CHECK(int2dds_create_subscriber(dp, NULL, &sub));
    CHECK(TilePressure_create_topic(dp, "TilePressure", NULL, &topic));
    CHECK(int2dds_datareader_qos_create_default(&qos));
    CHECK(int2dds_datareader_qos_set_reliability(qos, INT2DDS_QOS_RELIABILITY_RELIABLE, 0));
    CHECK(int2dds_create_datareader(sub, topic, qos, NULL, 0, &reader));
    if (events) {
        CHECK(StepEvent_create_topic(dp, "StepEvent", NULL, &ev_topic));
        CHECK(int2dds_create_datareader(sub, ev_topic, qos, NULL, 0, &ev_reader));
    }
    int2dds_datareader_qos_destroy(qos);
    lab_log("sub ready: domain=%d events=%d", domain, events);

    uint8_t buf[4096];
    uintptr_t n;
    Int2DdsSampleInfo info;
    while (!shutdown_is_stopped()) {
        Int2DdsRet ret = int2dds_datareader_take_serialized_w_info(reader, buf, sizeof buf, &n, &info);
        if (ret == INT2DDS_RET_NO_DATA) {
            StepEvent e;
            while (ev_reader && int2dds_datareader_take_serialized_w_info(ev_reader, buf, sizeof buf, &n, &info) == INT2DDS_RET_OK)
                if (info.valid_data && StepEvent_deserialize_cdr(buf, n, &e))
                    lab_log("EVENT step on tile(%d,%d) ts=%lld", e.row, e.col, (long long)e.ts_ms);
            shutdown_wait_timeout(100);
            continue;
        }
        if (ret != INT2DDS_RET_OK) {
            lab_log("take failed: %s", lab_ret_name(ret));
            break;
        }

        if (!info.valid_data) { /* dispose · unregister · Writer 소멸 알림: buf를 디코딩하지 않는다 */
            lab_log("%s -> %s  (valid_data=false)", tile_name(info.instance_handle),
                    lab_instance_state(info.instance_state));
            continue;
        }

        TilePressure t;
        if (TilePressure_deserialize_cdr(buf, n, &t)) {
            remember(info.instance_handle, t.row, t.col);
            lab_log("tile(%d,%d) p=%5.1f  view=%s inst=%s", t.row, t.col, t.pressure,
                    lab_view_state(info.view_state), lab_instance_state(info.instance_state));
        }
    }

    cleanup_participant(dp);
    return 0;
}

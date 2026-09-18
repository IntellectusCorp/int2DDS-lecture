/* 실습 2 · Active Floor Publisher
 *   타일 4개를 주기적으로 쓰다가 cycles회 뒤 (1,1)을 dispose, (1,2)를 unregister 한다.
 *   좌표는 1부터: 키가 전부 0이면 InstanceHandle이 16바이트 0이 되어 FFI가 NIL로 취급한다.
 *   --cycles N --period-ms N
 *   --autodispose 0|1   unregister 시 autodispose_unregistered_instances (기본 1 → DISPOSED)
 *   --events            StepEvent 토픽도 함께 발행 (도전)
 *   --no-wait           Reader 매칭을 기다리지 않는다 (late joiner 실험: 실습 8)
 *   --hold              Ctrl+C까지 남은 타일을 계속 쓴다
 *   --crash-after S     정리 없이 죽기 → 남은 타일이 NO_WRITERS가 되는 시점 관찰 (Participant lease, 약 100초 뒤)
 *   --profile Lib::Prof [--profile-file F]   Writer QoS를 프로파일로 (실습 8) */
#include "lab.h"
#include "shutdown.h"
#include "floor.h"

#define TILES 4
static const int16_t ROWS[TILES] = {1, 1, 2, 2};
static const int16_t COLS[TILES] = {1, 2, 1, 2};

int main(int argc, char **argv) {
    lab_init();
    shutdown_install();
    int domain = arg_int(argc, argv, "--domain", 0);
    int cycles = arg_int(argc, argv, "--cycles", 5);
    int period_ms = arg_int(argc, argv, "--period-ms", 500);
    bool autodispose = arg_int(argc, argv, "--autodispose", 1) != 0;
    bool events = arg_flag(argc, argv, "--events");
    bool no_wait = arg_flag(argc, argv, "--no-wait");
    bool hold = arg_flag(argc, argv, "--hold");
    int crash_after = arg_int(argc, argv, "--crash-after", 0);
    const char *profile = arg_str(argc, argv, "--profile", NULL);
    const char *profile_file = arg_str(argc, argv, "--profile-file", LAB_XML_DIR "/team_qos.xml");

    Int2DdsParticipantFactory *factory;
    Int2DdsParticipant *dp;
    Int2DdsPublisher *pub;
    Int2DdsTopic *topic, *ev_topic = NULL;
    Int2DdsDataWriter *writer, *ev_writer = NULL;

    CHECK(int2dds_domain_participant_factory_get_instance(&factory));
    CHECK(int2dds_create_participant(factory, domain, NULL, &dp));
    CHECK(int2dds_create_publisher(dp, NULL, &pub));
    /* keyed 토픽: _create_topic이 TypeInfo(키 포함)를 함께 등록해야 Instance API를 쓸 수 있다 */
    CHECK(TilePressure_create_topic(dp, "TilePressure", NULL, &topic));

    if (profile) {
        CHECK(int2dds_load_profiles(&profile_file, 1));
        CHECK(int2dds_create_datawriter_with_profile(pub, topic, profile, NULL, 0, &writer));
        lab_log("writer qos from profile %s", profile);
    } else {
        Int2DdsDataWriterQos *qos;
        CHECK(int2dds_datawriter_qos_create_default(&qos));
        CHECK(int2dds_datawriter_qos_set_reliability(qos, INT2DDS_QOS_RELIABILITY_RELIABLE, MS(100)));
        CHECK(int2dds_datawriter_qos_set_writer_data_lifecycle(qos, autodispose));
        CHECK(int2dds_create_datawriter(pub, topic, qos, NULL, 0, &writer));
        int2dds_datawriter_qos_destroy(qos);
    }
    if (events) {
        CHECK(StepEvent_create_topic(dp, "StepEvent", NULL, &ev_topic));
        CHECK(int2dds_create_datawriter(pub, ev_topic, NULL, NULL, 0, &ev_writer));
    }
    lab_log("pub ready: domain=%d autodispose=%d events=%d", domain, autodispose, events);

    int32_t matched = 0;
    if (!no_wait)
        while (!shutdown_is_stopped() && !lab_poll_writer_matched(writer, &matched)) shutdown_wait_timeout(100);

    /* 1. register_instance: 직렬화한 샘플 전체를 넘기고 InstanceHandle을 받아 둔다 (와이어로는 안 나간다) */
    uint8_t buf[256];
    uint8_t handles[TILES][16];
    TilePressure t = {0};
    for (int i = 0; i < TILES; i++) {
        t.row = ROWS[i];
        t.col = COLS[i];
        size_t n = TilePressure_serialize_for(writer, &t, buf, sizeof buf);
        CHECK(int2dds_datawriter_register_instance(writer, buf, n, &handles[i]));
        char hx[9];
        lab_hex4(handles[i], hx);
        lab_log("register tile(%d,%d) handle=%s", t.row, t.col, hx);
    }

    bool alive[TILES] = {true, true, true, true};
    for (int c = 1; !shutdown_is_stopped(); c++) {
        int spike = c % TILES; /* 이번 주기에 밟힌 타일 */
        for (int i = 0; i < TILES; i++) {
            if (!alive[i]) continue;
            t.row = ROWS[i];
            t.col = COLS[i];
            t.pressure = 10.0f * i + (i == spike ? 50.0f : 0.0f);
            /* 2. write: 샘플 안의 키로 Instance를 식별한다 → Reader에서 ALIVE */
            size_t n = TilePressure_serialize_for(writer, &t, buf, sizeof buf);
            CHECK(int2dds_datawriter_write_serialized(writer, buf, n));
            lab_log("write tile(%d,%d) p=%.1f", t.row, t.col, t.pressure);

            if (events && i == spike) {
                StepEvent e = {t.row, t.col, now_ms()};
                n = StepEvent_serialize_for(ev_writer, &e, buf, sizeof buf);
                CHECK(int2dds_datawriter_write_serialized(ev_writer, buf, n));
            }
        }

        if (c == cycles) {
            /* 3. dispose: 등록해 둔 핸들로 지정 (key=NULL) → NOT_ALIVE_DISPOSED */
            CHECK(int2dds_datawriter_dispose(writer, NULL, 0, &handles[0]));
            alive[0] = false;
            lab_log("dispose tile(1,1)");

            /* 4. unregister_instance: 직렬화한 샘플의 키로 지정
             *    autodispose=1 → NOT_ALIVE_DISPOSED, 0 → NOT_ALIVE_NO_WRITERS */
            t.row = ROWS[1];
            t.col = COLS[1];
            size_t n = TilePressure_serialize_for(writer, &t, buf, sizeof buf);
            CHECK(int2dds_datawriter_unregister_instance(writer, buf, n, NULL));
            alive[1] = false;
            lab_log("unregister tile(1,2) (autodispose=%d)", autodispose);
        }
        if (c >= 2 * cycles && !hold) break;

        lab_crash_check(crash_after);
        if (shutdown_wait_timeout(period_ms)) break;
    }

    cleanup_participant(dp);
    return 0;
}

/* 실습 8 · 3단계: 타입·QoS·Domain·Participant 트리를 app.xml 하나로. 코드에 남는 건 이름뿐이다.
 *   --config FILE  (기본 app.xml)  --reader NAME (기본 sub::tile) */
#include "lab.h"
#include "shutdown.h"

int main(int argc, char **argv) {
    lab_init();
    shutdown_install();
    const char *config = arg_str(argc, argv, "--config", LAB_XML_DIR "/app.xml");
    const char *reader_name = arg_str(argc, argv, "--reader", "sub::tile");

    /* 1. 구성 파일 로드: 타입 · QoS · Domain · Participant 선언이 한 번에 */
    CHECK(int2dds_load_profiles(&config, 1));

    /* 2. 이름으로 트리 생성 */
    Int2DdsParticipantFactory *factory;
    Int2DdsConfiguredParticipant *app;
    CHECK(int2dds_domain_participant_factory_get_instance(&factory));
    CHECK(int2dds_create_participant_from_config(factory, "Apps::Monitor", &app));

    /* 3. 선언한 이름으로 Reader를 꺼낸다 (없는 이름 → DYNAMIC_FIELD_NOT_FOUND) */
    Int2DdsDynamicDataReader *reader;
    CHECK(int2dds_configured_participant_get_datareader(app, reader_name, &reader));

    Int2DdsDataReaderQos *q;
    int32_t rel, dur;
    int64_t mbt;
    CHECK(int2dds_dynamic_reader_get_qos(reader, &q));
    CHECK(int2dds_datareader_qos_get_reliability(q, &rel, &mbt));
    CHECK(int2dds_datareader_qos_get_durability(q, &dur));
    int2dds_datareader_qos_destroy(q);
    lab_log("sub ready (config): reader=%s reliability=%s durability=%s", reader_name,
            rel == INT2DDS_QOS_RELIABILITY_RELIABLE ? "RELIABLE" : "BEST_EFFORT",
            dur == INT2DDS_QOS_DURABILITY_TRANSIENT_LOCAL ? "TRANSIENT_LOCAL" : "VOLATILE");

    int64_t next_status = 0;
    while (!shutdown_is_stopped()) {
        if (now_ms() >= next_status) {
            int32_t matched;
            CHECK(int2dds_dynamic_reader_subscription_matched_count(reader, &matched));
            lab_log("matched writers=%d", matched);
            next_status = now_ms() + 1000;
        }
        Int2DdsDynamicData *sample = NULL;
        Int2DdsSampleInfo info;
        Int2DdsRet ret = int2dds_dynamic_reader_take(reader, &sample, &info);
        if (ret == INT2DDS_RET_NO_DATA) {
            shutdown_wait_timeout(100);
            continue;
        }
        if (ret != INT2DDS_RET_OK) break;
        if (info.valid_data) {
            int16_t row, col;
            float p;
            CHECK(int2dds_dynamic_data_get_i16(sample, "row", &row));
            CHECK(int2dds_dynamic_data_get_i16(sample, "col", &col));
            CHECK(int2dds_dynamic_data_get_f32(sample, "pressure", &p));
            lab_log("tile(%d,%d) p=%5.1f", row, col, p);
        } else {
            lab_log("%s (valid_data=false)", lab_instance_state(info.instance_state));
        }
        if (sample) int2dds_dynamic_data_destroy(sample);
    }

    /* 4. 해제: 꺼낸 핸들 → 트리 */
    lab_log("cleanup");
    int2dds_dynamic_reader_destroy(reader);
    int2dds_configured_participant_destroy(app);
    return 0;
}

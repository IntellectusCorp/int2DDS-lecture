/* 실습 8 · 2단계: QoS도 XML로. Team::HardState를 기본 프로파일로 물려 NULL QoS Reader가 받게 한다.
 *   dynamic 계열에는 _with_profile이 없다 → 기본 프로파일 경로(DDS_DEFAULT_QOS_PROFILE)를 쓴다.
 *   pub 쪽도 같은 프로파일이어야 한다(VOLATILE Writer는 TRANSIENT_LOCAL Reader와 매칭되지 않는다):
 *     lab2_pub --profile Team::HardState --hold   를 먼저, 이 sub를 늦게 띄운다.
 *   --profile-file FILE  --xml FILE  --domain N */
#include "lab.h"
#include "shutdown.h"

static void print_reader_qos(Int2DdsDynamicDataReader *reader) {
    Int2DdsDataReaderQos *q;
    int32_t rel, dur, hk, depth;
    int64_t mbt;
    CHECK(int2dds_dynamic_reader_get_qos(reader, &q));
    CHECK(int2dds_datareader_qos_get_reliability(q, &rel, &mbt));
    CHECK(int2dds_datareader_qos_get_durability(q, &dur));
    CHECK(int2dds_datareader_qos_get_history(q, &hk, &depth));
    int2dds_datareader_qos_destroy(q);
    lab_log("effective reader qos: reliability=%s durability=%s history=%s(%d)",
            rel == INT2DDS_QOS_RELIABILITY_RELIABLE ? "RELIABLE" : "BEST_EFFORT",
            dur == INT2DDS_QOS_DURABILITY_TRANSIENT_LOCAL ? "TRANSIENT_LOCAL" : "VOLATILE",
            hk == INT2DDS_QOS_HISTORY_KEEP_ALL ? "KEEP_ALL" : "KEEP_LAST", depth);
}

int main(int argc, char **argv) {
    lab_init();
    shutdown_install();
    int domain = arg_int(argc, argv, "--domain", 0);
    const char *profile_file = arg_str(argc, argv, "--profile-file", LAB_XML_DIR "/team_qos.xml");
    const char *xml = arg_str(argc, argv, "--xml", LAB_XML_DIR "/tile.xml");

    /* 첫 Participant 생성 전에 프로파일을 로드하고 기본 프로파일을 지정한다 */
    CHECK(int2dds_load_profiles(&profile_file, 1));
    CHECK(int2dds_env_set_default_qos_profile("Team::HardState"));

    Int2DdsXmlTypeRegistry *reg;
    Int2DdsDynamicTypeSupport *type;
    CHECK(int2dds_xml_type_registry_from_file(xml, &reg));
    CHECK(int2dds_xml_type_registry_get_type_support(reg, "TilePressure", &type));

    Int2DdsParticipantFactory *factory;
    Int2DdsParticipant *dp;
    Int2DdsSubscriber *sub;
    Int2DdsTopic *topic;
    Int2DdsDynamicDataReader *reader;
    CHECK(int2dds_domain_participant_factory_get_instance(&factory));
    CHECK(int2dds_create_participant(factory, domain, NULL, &dp));
    CHECK(int2dds_create_subscriber(dp, NULL, &sub));
    CHECK(int2dds_create_topic_dynamic(dp, "TilePressure", type, NULL, &topic));
    CHECK(int2dds_create_datareader_dynamic(sub, topic, type, NULL, &reader)); /* NULL QoS → 기본 프로파일 */
    lab_log("sub ready (xml type + profile): domain=%d", domain);
    print_reader_qos(reader);

    while (!shutdown_is_stopped()) {
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
            lab_log("tile(%d,%d) p=%5.1f  view=%s", row, col, p, lab_view_state(info.view_state));
        } else {
            lab_log("%s (valid_data=false)", lab_instance_state(info.instance_state));
        }
        if (sample) int2dds_dynamic_data_destroy(sample);
    }

    int2dds_dynamic_reader_destroy(reader);
    cleanup_participant(dp);
    int2dds_dynamic_type_support_destroy(type);
    int2dds_xml_type_registry_destroy(reg);
    return 0;
}

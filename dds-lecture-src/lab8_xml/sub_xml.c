/* 실습 8 · 1단계: 타입만 XML로. IDL·코드 생성 없이 실습 2의 lab2_pub와 통신한다.
 *   --xml FILE   (기본 tile.xml; tile_lab.xml + --type lab::TilePressure 로 이름 불일치 실험)
 *   --type NAME  --topic NAME  --domain N */
#include "lab.h"
#include "shutdown.h"

static void print_sample(Int2DdsDynamicData *sample, const Int2DdsSampleInfo *info) {
    if (!info->valid_data) {
        lab_log("%s (valid_data=false)", lab_instance_state(info->instance_state));
        return;
    }
    int16_t row, col;
    float p;
    CHECK(int2dds_dynamic_data_get_i16(sample, "row", &row));
    CHECK(int2dds_dynamic_data_get_i16(sample, "col", &col));
    CHECK(int2dds_dynamic_data_get_f32(sample, "pressure", &p));
    lab_log("tile(%d,%d) p=%5.1f  view=%s inst=%s", row, col, p, lab_view_state(info->view_state),
            lab_instance_state(info->instance_state));
}

int main(int argc, char **argv) {
    lab_init();
    shutdown_install();
    int domain = arg_int(argc, argv, "--domain", 0);
    const char *xml = arg_str(argc, argv, "--xml", LAB_XML_DIR "/tile.xml");
    const char *type_name = arg_str(argc, argv, "--type", "TilePressure");
    const char *topic_name = arg_str(argc, argv, "--topic", "TilePressure");

    /* 1. XML 로드 → 타입 지원 객체 (int2dds-idl 단계가 없다) */
    Int2DdsXmlTypeRegistry *reg;
    Int2DdsDynamicTypeSupport *type;
    CHECK(int2dds_xml_type_registry_from_file(xml, &reg));
    CHECK(int2dds_xml_type_registry_get_type_support(reg, type_name, &type));

    Int2DdsParticipantFactory *factory;
    Int2DdsParticipant *dp;
    Int2DdsSubscriber *sub;
    Int2DdsTopic *topic;
    Int2DdsDynamicDataReader *reader;
    CHECK(int2dds_domain_participant_factory_get_instance(&factory));
    CHECK(int2dds_create_participant(factory, domain, NULL, &dp));
    CHECK(int2dds_create_subscriber(dp, NULL, &sub));

    /* 2. _dynamic 계열로 Topic · Reader 생성 */
    CHECK(int2dds_create_topic_dynamic(dp, topic_name, type, NULL, &topic));
    CHECK(int2dds_create_datareader_dynamic(sub, topic, type, NULL, &reader));
    lab_log("sub ready (xml type): type=%s topic=%s domain=%d", type_name, topic_name, domain);

    /* 3. 필드 이름으로 읽기 */
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
        if (ret != INT2DDS_RET_OK) {
            lab_log("take failed: %s", lab_ret_name(ret));
            break;
        }
        print_sample(sample, &info);
        if (sample) int2dds_dynamic_data_destroy(sample);
    }

    /* 해제 순서: DynamicData → Reader → Participant 트리 → TypeSupport → Registry */
    int2dds_dynamic_reader_destroy(reader);
    cleanup_participant(dp);
    int2dds_dynamic_type_support_destroy(type);
    int2dds_xml_type_registry_destroy(reg);
    return 0;
}

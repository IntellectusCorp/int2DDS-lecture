/* 실습 5 · read와 take, 상태 마스크 (실습 2 sub 기반, pub은 lab2_pub)
 *   1초마다 도착한 샘플에 대해 다섯 단계를 순서대로 실행한다.
 *     1 read_serialized_w_info 반복  → 도착한 샘플 전부 (기본 마스크는 NOT_READ만)
 *     2 read_serialized_w_info 한 번 → NO_DATA (전부 READ가 됐다)
 *     3 read_w_states(READ) 한 번    → 첫 샘플이 sample_state=READ로 다시 온다
 *     4 take_w_states(ANY) 반복      → 꺼내면서 Cache에서 제거
 *     5 read_w_states(ANY) 한 번     → NO_DATA (take 뒤엔 비어 있다)
 *   --domain N */
#include "lab.h"
#include "shutdown.h"
#include "floor.h"

/* single: 한 번만 호출. read는 상태를 바꾸지 않으니 READ/ANY 마스크 read를 반복하면 같은 샘플만 계속 온다 */
static int drain(Int2DdsDataReader *reader, const char *label, bool take, bool w_states, uint32_t smask,
                 bool single) {
    uint8_t buf[4096];
    uintptr_t n;
    Int2DdsSampleInfo info;
    TilePressure t;
    int count = 0;
    do {
        Int2DdsRet ret;
        if (!w_states)
            ret = int2dds_datareader_read_serialized_w_info(reader, buf, sizeof buf, &n, &info);
        else if (take)
            ret = int2dds_datareader_take_serialized_w_states(reader, buf, sizeof buf, &n, &info, smask,
                                                              INT2DDS_VIEW_STATE_ANY, INT2DDS_INSTANCE_STATE_ANY);
        else
            ret = int2dds_datareader_read_serialized_w_states(reader, buf, sizeof buf, &n, &info, smask,
                                                              INT2DDS_VIEW_STATE_ANY, INT2DDS_INSTANCE_STATE_ANY);
        if (ret == INT2DDS_RET_NO_DATA) break;
        if (ret != INT2DDS_RET_OK) {
            lab_log("  %-26s %s", label, lab_ret_name(ret));
            break;
        }
        count++;
        if (info.valid_data && TilePressure_deserialize_cdr(buf, n, &t))
            lab_log("  %-26s tile(%d,%d) p=%5.1f  sample_state=%s", label, t.row, t.col, t.pressure,
                    lab_sample_state(info.sample_state));
        else
            lab_log("  %-26s %s (valid_data=false)", label, lab_instance_state(info.instance_state));
    } while (!single);
    if (count == 0) lab_log("  %-26s NO_DATA", label);
    return count;
}

int main(int argc, char **argv) {
    lab_init();
    shutdown_install();
    int domain = arg_int(argc, argv, "--domain", 0);

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
    CHECK(int2dds_datareader_qos_set_history(qos, INT2DDS_QOS_HISTORY_KEEP_LAST, 8));
    CHECK(int2dds_create_datareader(sub, topic, qos, NULL, 0, &reader));
    int2dds_datareader_qos_destroy(qos);
    lab_log("sub ready: domain=%d", domain);

    while (!shutdown_is_stopped()) {
        if (shutdown_wait_timeout(1000)) break;
        if (drain(reader, "1 read_w_info", false, false, 0, false) == 0) continue;
        drain(reader, "2 read_w_info again", false, false, 0, true);
        drain(reader, "3 read_w_states(READ)", false, true, INT2DDS_SAMPLE_STATE_READ, true);
        drain(reader, "4 take_w_states(ANY)", true, true, INT2DDS_SAMPLE_STATE_ANY, false);
        drain(reader, "5 read_w_states(ANY)", false, true, INT2DDS_SAMPLE_STATE_ANY, true);
        lab_log("----");
    }

    cleanup_participant(dp);
    return 0;
}

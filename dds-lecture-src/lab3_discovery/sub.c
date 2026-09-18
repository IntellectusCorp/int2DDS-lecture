/* 실습 3 · Discovery 관찰
 *   실습 1 sub + 1초마다 발견된 Participant 목록과 매칭된 Writer 수를 찍는다.
 *   목록은 상대만 센다(자기 자신 제외). pub 쪽은 lab1_pub 그대로 쓴다.
 *   --domain N --topic NAME */
#include "lab.h"
#include "shutdown.h"
#include "hello.h"

static void report(Int2DdsParticipant *dp, Int2DdsDataReader *reader) {
    uint8_t handles[16][16];
    uintptr_t count = 0;
    char list[16 * 10 + 1] = "";
    CHECK(int2dds_participant_get_discovered_participants(dp, handles, 16, &count));
    /* count는 전체 수라 16을 넘을 수 있다: 배열에 받은 16개까지만 찍는다 */
    for (uintptr_t i = 0; i < count && i < 16; i++) {
        char hx[9];
        lab_hex4(handles[i], hx);
        strcat(list, hx);
        strcat(list, " ");
    }
    Int2DdsSubscriptionMatchedStatus sm;
    CHECK(int2dds_datareader_get_subscription_matched_status(reader, &sm));
    lab_log("participants=%d [%s]  matched writers=%d", (int)count, list, sm.current_count);
}

int main(int argc, char **argv) {
    lab_init();
    shutdown_install();
    int domain = arg_int(argc, argv, "--domain", 0);
    const char *topic_name = arg_str(argc, argv, "--topic", "hello_world_topic");

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
    CHECK(int2dds_datareader_qos_set_reliability(qos, INT2DDS_QOS_RELIABILITY_RELIABLE, 0));
    CHECK(int2dds_create_datareader(sub, topic, qos, NULL, 0, &reader));
    int2dds_datareader_qos_destroy(qos);
    lab_log("sub ready: domain=%d topic=%s", domain, topic_name);

    uint8_t buf[4096];
    uintptr_t size;
    bool valid_data;
    int64_t next_report = 0;
    while (!shutdown_is_stopped()) {
        if (now_ms() >= next_report) {
            report(dp, reader);
            next_report = now_ms() + 1000;
        }
        Int2DdsRet ret = int2dds_datareader_take_serialized(reader, buf, sizeof buf, &size, &valid_data);
        if (ret == INT2DDS_RET_NO_DATA) {
            shutdown_wait_timeout(100);
            continue;
        }
        if (ret != INT2DDS_RET_OK) break;
        HelloWorld sample = {0};
        if (valid_data && HelloWorld_deserialize_cdr(buf, size, &sample))
            lab_log("recv index=%d message=%s", sample.index, sample.message);
    }

    cleanup_participant(dp);
    return 0;
}

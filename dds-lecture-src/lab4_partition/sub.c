/* 실습 4 · Partition Subscriber (실습 3 sub + Subscriber QoS의 PARTITION + 런타임 변경)
 *   --partition floor-*[,x]            시작 파티션 (없으면 기본 "")
 *   --switch-after S --switch-to LIST  S초 뒤 파티션을 바꾼다 (34장 런타임 변경 코드)
 *   --domain N --topic NAME */
#include "lab.h"
#include "shutdown.h"
#include "hello.h"

static void report(Int2DdsParticipant *dp, Int2DdsDataReader *reader) {
    uint8_t handles[16][16];
    uintptr_t count = 0;
    Int2DdsSubscriptionMatchedStatus sm;
    CHECK(int2dds_participant_get_discovered_participants(dp, handles, 16, &count));
    CHECK(int2dds_datareader_get_subscription_matched_status(reader, &sm));
    lab_log("participants=%d  matched writers=%d", (int)count, sm.current_count);
}

int main(int argc, char **argv) {
    lab_init();
    shutdown_install();
    int domain = arg_int(argc, argv, "--domain", 0);
    const char *topic_name = arg_str(argc, argv, "--topic", "hello_world_topic");
    int switch_after = arg_int(argc, argv, "--switch-after", 0);
    char storage[256], storage2[256];
    const char *parts[8], *parts2[8];
    int nparts = arg_list(arg_str(argc, argv, "--partition", NULL), storage, sizeof storage, parts, 8);
    int nparts2 = arg_list(arg_str(argc, argv, "--switch-to", NULL), storage2, sizeof storage2, parts2, 8);

    Int2DdsParticipantFactory *factory;
    Int2DdsParticipant *dp;
    Int2DdsSubscriberQos *sq;
    Int2DdsSubscriber *sub;
    Int2DdsTopic *topic;
    Int2DdsDataReader *reader;

    CHECK(int2dds_domain_participant_factory_get_instance(&factory));
    CHECK(int2dds_create_participant(factory, domain, NULL, &dp));
    CHECK(int2dds_subscriber_qos_create_default(&sq));
    CHECK(int2dds_subscriber_qos_set_partition(sq, parts, nparts));
    CHECK(int2dds_create_subscriber(dp, sq, &sub));
    int2dds_subscriber_qos_destroy(sq);
    CHECK(HelloWorld_create_topic(dp, topic_name, NULL, &topic));
    CHECK(int2dds_create_datareader(sub, topic, NULL, NULL, 0, &reader));
    lab_log("sub ready: domain=%d partitions=%d (%s)", domain, nparts,
            nparts ? arg_str(argc, argv, "--partition", "") : "\"\"");

    uint8_t buf[4096];
    uintptr_t size;
    bool valid_data;
    int64_t next_report = 0;
    bool switched = false;
    while (!shutdown_is_stopped()) {
        if (now_ms() >= next_report) {
            report(dp, reader);
            next_report = now_ms() + 1000;
        }
        if (switch_after > 0 && !switched && now_ms() - g_t0 >= (int64_t)switch_after * 1000) {
            /* Partition은 변경 가능한 QoS: Subscriber set_qos 뒤 Reader set_qos로 다시 광고한다 */
            Int2DdsSubscriberQos *q;
            Int2DdsDataReaderQos *rq;
            CHECK(int2dds_subscriber_get_qos(sub, &q));
            CHECK(int2dds_subscriber_qos_set_partition(q, parts2, nparts2));
            CHECK(int2dds_subscriber_set_qos(sub, q));
            int2dds_subscriber_qos_destroy(q);
            CHECK(int2dds_datareader_get_qos(reader, &rq));
            CHECK(int2dds_datareader_set_qos(reader, rq));
            int2dds_datareader_qos_destroy(rq);
            switched = true;
            lab_log("partition switched to (%s)", arg_str(argc, argv, "--switch-to", "\"\""));
        }
        Int2DdsRet ret = int2dds_datareader_take_serialized(reader, buf, sizeof buf, &size, &valid_data);
        if (ret == INT2DDS_RET_NO_DATA) {
            shutdown_wait_timeout(100);
            continue;
        }
        if (ret != INT2DDS_RET_OK) break;
        HelloWorld sample = {0};
        if (valid_data && HelloWorld_deserialize_cdr(buf, size, &sample))
            lab_log("recv index=%d", sample.index);
    }

    cleanup_participant(dp);
    return 0;
}

/* 실습 1 · Subscriber (폴링)
 *   --domain N   --topic NAME */
#include "lab.h"
#include "shutdown.h"
#include "hello.h"

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
    while (!shutdown_is_stopped()) {
        Int2DdsRet ret = int2dds_datareader_take_serialized(reader, buf, sizeof buf, &size, &valid_data);
        if (ret == INT2DDS_RET_NO_DATA) {
            shutdown_wait_timeout(200);
            continue;
        }
        if (ret != INT2DDS_RET_OK) {
            lab_log("take failed: %s", lab_ret_name(ret));
            break;
        }
        if (!valid_data) continue; /* 상태 통지: buf가 비어 있다 */

        HelloWorld sample = {0};
        if (HelloWorld_deserialize_cdr(buf, size, &sample))
            lab_log("recv index=%d message=%s", sample.index, sample.message);
    }

    cleanup_participant(dp);
    return 0;
}

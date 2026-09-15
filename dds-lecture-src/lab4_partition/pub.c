/* 실습 4 · Partition Publisher (실습 1 pub + Publisher QoS의 PARTITION)
 *   --partition floor-1[,floor-2]   --domain N --topic NAME --period-ms N --count N(0=무한) */
#include "lab.h"
#include "shutdown.h"
#include "hello.h"

int main(int argc, char **argv) {
    lab_init();
    shutdown_install();
    int domain = arg_int(argc, argv, "--domain", 0);
    const char *topic_name = arg_str(argc, argv, "--topic", "hello_world_topic");
    int period_ms = arg_int(argc, argv, "--period-ms", 500);
    int count = arg_int(argc, argv, "--count", 0);
    char storage[256];
    const char *parts[8];
    int nparts = arg_list(arg_str(argc, argv, "--partition", NULL), storage, sizeof storage, parts, 8);

    Int2DdsParticipantFactory *factory;
    Int2DdsParticipant *dp;
    Int2DdsPublisherQos *pq;
    Int2DdsPublisher *pub;
    Int2DdsTopic *topic;
    Int2DdsDataWriter *writer;

    CHECK(int2dds_domain_participant_factory_get_instance(&factory));
    CHECK(int2dds_create_participant(factory, domain, NULL, &dp));

    /* Partition은 Publisher/Subscriber QoS다 */
    CHECK(int2dds_publisher_qos_create_default(&pq));
    CHECK(int2dds_publisher_qos_set_partition(pq, parts, nparts));
    CHECK(int2dds_create_publisher(dp, pq, &pub));
    int2dds_publisher_qos_destroy(pq);

    CHECK(HelloWorld_create_topic(dp, topic_name, NULL, &topic));
    CHECK(int2dds_create_datawriter(pub, topic, NULL, NULL, 0, &writer));
    lab_log("pub ready: domain=%d partitions=%d (%s)", domain, nparts,
            nparts ? arg_str(argc, argv, "--partition", "") : "\"\"");

    int32_t matched = 0;
    uint8_t buf[4096];
    HelloWorld sample = {0};
    for (int i = 1; !shutdown_is_stopped() && (count == 0 || i <= count); i++) {
        sample.index = i;
        snprintf(sample.message, sizeof sample.message, "hello");
        size_t n = HelloWorld_serialize_for(writer, &sample, buf, sizeof buf);
        CHECK(int2dds_datawriter_write_serialized(writer, buf, n));
        lab_log("write index=%d", i);
        lab_poll_writer_matched(writer, &matched);
        if (shutdown_wait_timeout(period_ms)) break;
    }

    cleanup_participant(dp);
    return 0;
}

/* 실습 1 · Publisher
 *   --domain N        (변형 1: pub만 domain 1)
 *   --topic NAME      (변형 2: 토픽 이름 바꾸기)
 *   --no-wait         (변형 3: 매칭 대기 없이 바로 write)
 *   --count N --period-ms N
 *   --hold            다 쓰고도 Ctrl+C까지 살아 있기 (실습 3·7)
 *   --crash-after S   S초 뒤 정리 없이 죽기 (실습 3: 정상 종료와 비교) */
#include "lab.h"
#include "shutdown.h"
#include "hello.h"

int main(int argc, char **argv) {
    lab_init();
    shutdown_install();
    int domain = arg_int(argc, argv, "--domain", 0);
    const char *topic_name = arg_str(argc, argv, "--topic", "hello_world_topic");
    int count = arg_int(argc, argv, "--count", 10);
    int period_ms = arg_int(argc, argv, "--period-ms", 500);
    bool no_wait = arg_flag(argc, argv, "--no-wait");
    bool hold = arg_flag(argc, argv, "--hold");
    int crash_after = arg_int(argc, argv, "--crash-after", 0);

    Int2DdsParticipantFactory *factory;
    Int2DdsParticipant *dp;
    Int2DdsPublisher *pub;
    Int2DdsTopic *topic;
    Int2DdsDataWriterQos *qos;
    Int2DdsDataWriter *writer;

    CHECK(int2dds_domain_participant_factory_get_instance(&factory));
    CHECK(int2dds_create_participant(factory, domain, NULL, &dp));
    CHECK(int2dds_create_publisher(dp, NULL, &pub));
    CHECK(HelloWorld_create_topic(dp, topic_name, NULL, &topic));

    CHECK(int2dds_datawriter_qos_create_default(&qos));
    CHECK(int2dds_datawriter_qos_set_reliability(qos, INT2DDS_QOS_RELIABILITY_RELIABLE, MS(100)));
    CHECK(int2dds_create_datawriter(pub, topic, qos, NULL, 0, &writer));
    int2dds_datawriter_qos_destroy(qos);
    lab_log("pub ready: domain=%d topic=%s", domain, topic_name);

    /* 매칭 대기는 실습 편의용이지 write의 필수 조건은 아니다 */
    int32_t matched = 0;
    if (!no_wait)
        while (!shutdown_is_stopped() && !lab_poll_writer_matched(writer, &matched)) shutdown_wait_timeout(100);

    uint8_t buf[4096];
    HelloWorld sample = {0};
    for (int i = 1; i <= count && !shutdown_is_stopped(); i++) {
        sample.index = i;
        snprintf(sample.message, sizeof sample.message, "hello");

        size_t n = HelloWorld_serialize_for(writer, &sample, buf, sizeof buf);
        if (n == 0) {
            fprintf(stderr, "serialize failed\n");
            break;
        }
        CHECK(int2dds_datawriter_write_serialized(writer, buf, n));
        lab_log("write index=%d", i);

        lab_poll_writer_matched(writer, &matched);
        lab_crash_check(crash_after);
        if (shutdown_wait_timeout(period_ms)) break;
    }

    while (hold && !shutdown_is_stopped()) {
        lab_poll_writer_matched(writer, &matched);
        lab_crash_check(crash_after);
        shutdown_wait_timeout(200);
    }

    cleanup_participant(dp);
    return 0;
}

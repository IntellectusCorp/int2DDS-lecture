/* 실습 5 · WaitSet 버전 (실습 2 sub → ReadCondition + GuardCondition)
 *   wait()에서 블로킹, 처리 스레드와 순서는 애플리케이션이 쥔다.
 *   Ctrl+C → 감시 스레드가 GuardCondition을 켜서 루프를 깨운다.
 *   --domain N   --timeout-ms N (기본 1000; TIMEOUT 반환을 보려면 pub 없이 띄운다) */
#include "lab.h"
#include "shutdown.h"
#include "floor.h"

static void stop_watcher(void *arg) {
    Int2DdsGuardCondition *stop = arg;
    shutdown_wait();
    int2dds_guardcondition_set_trigger_value(stop, true);
}

int main(int argc, char **argv) {
    lab_init();
    shutdown_install();
    int domain = arg_int(argc, argv, "--domain", 0);
    int timeout_ms = arg_int(argc, argv, "--timeout-ms", 1000);

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
    CHECK(int2dds_create_datareader(sub, topic, qos, NULL, 0, &reader));
    int2dds_datareader_qos_destroy(qos);

    Int2DdsWaitSet *ws;
    Int2DdsReadCondition *new_data;
    Int2DdsGuardCondition *stop;
    CHECK(int2dds_waitset_new(&ws));
    CHECK(int2dds_datareader_create_readcondition(reader, INT2DDS_SAMPLE_STATE_NOT_READ,
                                                  INT2DDS_VIEW_STATE_ANY, INT2DDS_INSTANCE_STATE_ANY, &new_data));
    CHECK(int2dds_guardcondition_new(&stop));
    CHECK(int2dds_waitset_attach_readcondition(ws, new_data));
    CHECK(int2dds_waitset_attach_guardcondition(ws, stop));
    lab_thread_start(stop_watcher, stop);
    lab_log("sub ready (waitset): domain=%d timeout=%d ms", domain, timeout_ms);

    for (;;) {
        Int2DdsRet ret = int2dds_waitset_wait_ex(ws, timeout_ms, NULL);
        if (ret == INT2DDS_RET_TIMEOUT) {
            lab_log("wait: TIMEOUT (%d)", (int)ret);
            continue;
        }
        if (ret != INT2DDS_RET_OK) {
            lab_log("wait failed: %s", lab_ret_name(ret));
            break;
        }
        bool quit;
        CHECK(int2dds_guardcondition_get_trigger_value(stop, &quit));
        if (quit) break;

        Int2DdsSampleSeq *seq;
        CHECK(int2dds_datareader_take_serialized_batch_w_readcondition(reader, new_data, 32, &seq));
        for (uintptr_t i = 0; i < int2dds_sample_seq_length(seq); i++) {
            uint8_t buf[4096];
            uintptr_t n;
            Int2DdsSampleInfo info;
            TilePressure t;
            CHECK(int2dds_sample_seq_get_data(seq, i, buf, sizeof buf, &n));
            CHECK(int2dds_sample_seq_get_info(seq, i, &info));
            if (info.valid_data && TilePressure_deserialize_cdr(buf, n, &t))
                lab_log("tile(%d,%d) p=%5.1f", t.row, t.col, t.pressure);
            else
                lab_log("%s", lab_instance_state(info.instance_state));
        }
        int2dds_sample_seq_delete(seq);
    }

    int2dds_waitset_detach_readcondition(ws, new_data);
    int2dds_waitset_detach_guardcondition(ws, stop);
    int2dds_readcondition_delete(new_data);
    int2dds_guardcondition_delete(stop);
    int2dds_waitset_delete(ws);
    cleanup_participant(dp);
    return 0;
}

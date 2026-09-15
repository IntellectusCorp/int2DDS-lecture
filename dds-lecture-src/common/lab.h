/* 실습 공통 헬퍼: 시간, 인자, 반환값 검사, 상태 이름 */
#ifndef LAB_H
#define LAB_H

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include <process.h>
#include <windows.h>
static inline void sleep_ms(unsigned ms) { Sleep(ms); }
static inline int64_t now_ms(void) { return (int64_t)GetTickCount64(); }
static inline void lab_thread_start(void (*fn)(void *), void *arg) { _beginthread(fn, 0, arg); }

#define INT2DDS_CDR_STATIC
#include "int2dds-ffi.h"

#define MS(x) ((int64_t)(x) * 1000000LL) /* ms → ns (QoS Duration 인자) */

static int64_t g_t0 = 0;

/* stdout 무버퍼, 시작 시각 기록 (Ctrl+C 처리는 shutdown.h) */
static inline void lab_init(void) {
    setvbuf(stdout, NULL, _IONBF, 0);
    g_t0 = now_ms();
}

/* 경과 시간(ms)을 앞에 붙여 출력 */
static inline void lab_log(const char *fmt, ...) {
    va_list ap;
    printf("[%6lld ms] ", (long long)(now_ms() - g_t0));
    va_start(ap, fmt);
    vprintf(fmt, ap);
    va_end(ap);
    printf("\n");
}

static inline const char *lab_ret_name(Int2DdsRet r) {
    switch (r) {
    case INT2DDS_RET_OK: return "OK";
    case INT2DDS_RET_ERROR: return "ERROR";
    case INT2DDS_RET_TIMEOUT: return "TIMEOUT";
    case INT2DDS_RET_UNSUPPORTED: return "UNSUPPORTED";
    case INT2DDS_RET_INVALID_ARGUMENT: return "INVALID_ARGUMENT";
    case INT2DDS_RET_ALREADY_DELETED: return "ALREADY_DELETED";
    case INT2DDS_RET_NOT_ENABLED: return "NOT_ENABLED";
    case INT2DDS_RET_IMMUTABLE_POLICY: return "IMMUTABLE_POLICY";
    case INT2DDS_RET_INCONSISTENT_POLICY: return "INCONSISTENT_POLICY";
    case INT2DDS_RET_PRECONDITION_NOT_MET: return "PRECONDITION_NOT_MET";
    case INT2DDS_RET_OUT_OF_RESOURCES: return "OUT_OF_RESOURCES";
    case INT2DDS_RET_ILLEGAL_OPERATION: return "ILLEGAL_OPERATION";
    case INT2DDS_RET_NO_DATA: return "NO_DATA";
    case INT2DDS_RET_NULL_POINTER: return "NULL_POINTER";
    case INT2DDS_RET_BUFFER_TOO_SMALL: return "BUFFER_TOO_SMALL";
    case INT2DDS_RET_DYNAMIC_FIELD_NOT_FOUND: return "DYNAMIC_FIELD_NOT_FOUND";
    case INT2DDS_RET_DYNAMIC_TYPE_MISMATCH: return "DYNAMIC_TYPE_MISMATCH";
    default: return "?";
    }
}

/* 실패하면 호출문과 코드를 찍고 종료한다 */
#define CHECK(call)                                                                        \
    do {                                                                                   \
        Int2DdsRet r_ = (call);                                                            \
        if (r_ != INT2DDS_RET_OK) {                                                        \
            fprintf(stderr, "%s\n  -> %s (%d)\n", #call, lab_ret_name(r_), (int)r_);       \
            exit(1);                                                                       \
        }                                                                                  \
    } while (0)

/* 인자: --name value 또는 --flag */
static inline const char *arg_str(int argc, char **argv, const char *name, const char *def) {
    for (int i = 1; i + 1 < argc; i++)
        if (strcmp(argv[i], name) == 0) return argv[i + 1];
    return def;
}
static inline int arg_int(int argc, char **argv, const char *name, int def) {
    const char *v = arg_str(argc, argv, name, NULL);
    return v ? atoi(v) : def;
}
static inline bool arg_flag(int argc, char **argv, const char *name) {
    for (int i = 1; i < argc; i++)
        if (strcmp(argv[i], name) == 0) return true;
    return false;
}
/* "a,b,c" → 배열. 반환값은 항목 수 */
static inline int arg_list(const char *csv, char *storage, size_t cap, const char **out, int max) {
    int n = 0;
    if (!csv) return 0;
    strncpy(storage, csv, cap - 1);
    storage[cap - 1] = '\0';
    for (char *p = strtok(storage, ","); p && n < max; p = strtok(NULL, ",")) out[n++] = p;
    return n;
}

static inline const char *lab_instance_state(uint32_t s) {
    switch (s) {
    case INT2DDS_INSTANCE_STATE_ALIVE: return "ALIVE";
    case INT2DDS_INSTANCE_STATE_NOT_ALIVE_DISPOSED: return "NOT_ALIVE_DISPOSED";
    case INT2DDS_INSTANCE_STATE_NOT_ALIVE_NO_WRITERS: return "NOT_ALIVE_NO_WRITERS";
    default: return "?";
    }
}
static inline const char *lab_view_state(uint32_t s) {
    return s == INT2DDS_VIEW_STATE_NEW ? "NEW" : "NOT_NEW";
}
static inline const char *lab_sample_state(uint32_t s) {
    return s == INT2DDS_SAMPLE_STATE_READ ? "READ" : "NOT_READ";
}
static inline const char *lab_policy_name(int id) {
    switch (id) {
    case Durability: return "DURABILITY";
    case Deadline: return "DEADLINE";
    case Ownership: return "OWNERSHIP";
    case Liveliness: return "LIVELINESS";
    case Partition: return "PARTITION";
    case Reliability: return "RELIABILITY";
    case DestinationOrder: return "DESTINATION_ORDER";
    case History: return "HISTORY";
    case DataRepresentation: return "DATA_REPRESENTATION";
    case Invalid: return "(none)";
    default: return "?";
    }
}
/* InstanceHandle 앞 4바이트를 hex로 */
static inline void lab_hex4(const uint8_t h[16], char out[9]) {
    snprintf(out, 9, "%02x%02x%02x%02x", h[0], h[1], h[2], h[3]);
}

static inline int32_t lab_reliability_kind(const char *s) {
    return strcmp(s, "best_effort") == 0 ? INT2DDS_QOS_RELIABILITY_BEST_EFFORT
                                         : INT2DDS_QOS_RELIABILITY_RELIABLE;
}
static inline int32_t lab_durability_kind(const char *s) {
    return strcmp(s, "transient_local") == 0 ? INT2DDS_QOS_DURABILITY_TRANSIENT_LOCAL
                                             : INT2DDS_QOS_DURABILITY_VOLATILE;
}
static inline int32_t lab_ownership_kind(const char *s) {
    return strcmp(s, "exclusive") == 0 ? INT2DDS_QOS_OWNERSHIP_EXCLUSIVE
                                       : INT2DDS_QOS_OWNERSHIP_SHARED;
}
static inline int32_t lab_liveliness_kind(const char *s) {
    if (strcmp(s, "manual_by_topic") == 0) return INT2DDS_QOS_LIVELINESS_MANUAL_BY_TOPIC;
    if (strcmp(s, "manual_by_participant") == 0) return INT2DDS_QOS_LIVELINESS_MANUAL_BY_PARTICIPANT;
    return INT2DDS_QOS_LIVELINESS_AUTOMATIC;
}

/* publication_matched.current_count가 바뀌면 기록. 매칭된 Reader가 있으면 true */
static inline bool lab_poll_writer_matched(Int2DdsDataWriter *w, int32_t *last) {
    Int2DdsPublicationMatchedStatus st;
    CHECK(int2dds_datawriter_get_publication_matched_status(w, &st));
    if (st.current_count != *last) {
        lab_log("publication_matched: current_count %d -> %d (total %d)", *last,
                st.current_count, st.total_count);
        *last = st.current_count;
    }
    return st.current_count > 0;
}

/* --crash-after N초: 정리 없이 즉시 종료 → SPDP dispose가 나가지 않는다 */
static inline void lab_crash_check(int crash_after_s) {
    if (crash_after_s > 0 && now_ms() - g_t0 >= (int64_t)crash_after_s * 1000) {
        lab_log("simulated crash: _exit without cleanup");
        _exit(3);
    }
}

#endif /* LAB_H */

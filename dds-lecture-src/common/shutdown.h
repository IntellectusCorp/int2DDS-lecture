/* 실습 공통 종료 처리 (int2DDS dds/examples/common/shutdown.rs와 같은 구조)
 *   main 첫머리에서 shutdown_install(), 루프 조건은 shutdown_is_stopped(),
 *   쉬는 자리는 shutdown_wait_timeout(ms) (Ctrl+C면 바로 깬다), 할 일 없는 메인 스레드는 shutdown_wait(),
 *   마지막에 cleanup_participant(dp) */
#ifndef SHUTDOWN_H
#define SHUTDOWN_H

#include "lab.h"

static HANDLE g_shutdown = NULL; /* 수동 리셋 이벤트: 켜지면 stop (Rust의 Mutex<bool> + Condvar) */

static inline void shutdown_stop(void) { SetEvent(g_shutdown); }

static inline bool shutdown_is_stopped(void) { return WaitForSingleObject(g_shutdown, 0) == WAIT_OBJECT_0; }

/* Ctrl+C · Ctrl+Break → stop. 두 번째부터는 기본 처리(즉시 종료)로 넘겨 정리가 멈춰도 빠져나갈 수 있다 */
static BOOL WINAPI shutdown_on_ctrl(DWORD type) {
    if (type != CTRL_C_EVENT && type != CTRL_BREAK_EVENT) return FALSE;
    if (shutdown_is_stopped()) return FALSE;
    shutdown_stop();
    return TRUE;
}

static inline void shutdown_install(void) {
    g_shutdown = CreateEventA(NULL, TRUE, FALSE, NULL);
    SetConsoleCtrlHandler(shutdown_on_ctrl, TRUE);
}

/* 최대 ms 동안 기다린다. stop이 오면 바로 깨어나 true */
static inline bool shutdown_wait_timeout(unsigned ms) { return WaitForSingleObject(g_shutdown, ms) == WAIT_OBJECT_0; }

/* stop까지 블로킹 */
static inline void shutdown_wait(void) { WaitForSingleObject(g_shutdown, INFINITE); }

/* 생성의 역순 정리: 하위 Entity 전부 → Participant. 실패해도 알리고 계속한다 */
static inline void cleanup_participant(Int2DdsParticipant *dp) {
    lab_log("cleanup");
    Int2DdsRet r = int2dds_participant_delete_contained_entities(dp);
    if (r != INT2DDS_RET_OK) fprintf(stderr, "delete_contained_entities failed: %s\n", lab_ret_name(r));
    r = int2dds_delete_participant(dp);
    if (r != INT2DDS_RET_OK) fprintf(stderr, "delete_participant failed: %s\n", lab_ret_name(r));
}

#endif /* SHUTDOWN_H */

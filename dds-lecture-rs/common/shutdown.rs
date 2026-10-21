//! 실습 공통 종료 처리 (int2DDS dds/examples/common/shutdown.rs와 같은 구조)
//!   main 첫머리에서 Shutdown::install(), 루프 조건은 is_stopped(),
//!   쉬는 자리는 wait_timeout(ms) (Ctrl+C면 바로 깬다), 할 일 없는 메인 스레드는 wait(),
//!   마지막에 cleanup_participant(dp)
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

use int2dds::domain::{
    domain_participant::DomainParticipant, domain_participant_factory::DomainParticipantFactory,
};

pub struct Shutdown {
    flag: Mutex<bool>,
    cv: Condvar,
}

impl Shutdown {
    /// Ctrl+C → stop. 두 번째부터는 즉시 종료해서 정리가 멈춰도 빠져나갈 수 있다
    pub fn install() -> Arc<Self> {
        let me = Arc::new(Self { flag: Mutex::new(false), cv: Condvar::new() });
        let me_clone = Arc::clone(&me);
        let _ = ctrlc::set_handler(move || {
            if me_clone.is_stopped() {
                std::process::exit(130);
            }
            me_clone.stop();
        });
        me
    }

    pub fn stop(&self) {
        *self.flag.lock().unwrap() = true;
        self.cv.notify_all();
    }

    pub fn is_stopped(&self) -> bool {
        *self.flag.lock().unwrap()
    }

    /// 최대 ms 동안 기다린다. stop이 오면 바로 깨어나 true
    pub fn wait_timeout(&self, ms: u64) -> bool {
        let guard = self.flag.lock().unwrap();
        let (guard, _) = self
            .cv
            .wait_timeout_while(guard, Duration::from_millis(ms), |stopped| !*stopped)
            .unwrap();
        *guard
    }

    /// stop까지 블로킹
    pub fn wait(&self) {
        let guard = self.flag.lock().unwrap();
        let _guard = self.cv.wait_while(guard, |stopped| !*stopped).unwrap();
    }
}

/// 생성의 역순 정리: 하위 Entity 전부 → Participant. 실패해도 알리고 계속한다
pub fn cleanup_participant(dp: DomainParticipant) {
    crate::lab_log!("cleanup");
    if let Err(e) = dp.delete_contained_entities() {
        eprintln!("delete_contained_entities failed: {:?}", e);
    }
    if let Err(e) = DomainParticipantFactory::get_instance().delete_participant(dp) {
        eprintln!("delete_participant failed: {:?}", e);
    }
}

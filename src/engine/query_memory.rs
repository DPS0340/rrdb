//! 쿼리 수준 메모리 예산 추적기 (#265).
//!
//! PostgreSQL의 `work_mem`/memory context에 대응하는 개념으로, 한 쿼리가
//! 소비할 수 있는 메모리 상한을 넘어서면 **쿼리를 강제로 죽입니다** (에러 반환).
//! 디스크 spill(정렬/해시를 임시 파일로 내리는 것)이 없는 rrdb에서는
//! 예산 초과 시 에러를 반환하는 것이 OOM killer의 실용적 형태입니다.
//!
//! 정확한 바이트 계산은 불가능하므로(포인터 크기, Vec capacity, clone 중복),
//! 여기서는 **근사 추정**을 사용합니다:
//! - `TableDataFieldType::estimated_bytes()` — 값의 대략적 메모리 크기
//! - 파일 로드 시 `file_size` — 세그먼트 파일이 메모리에 통째로 올라가는 크기
//! - Vec append 시 예약된 capacity만큼 누적
//!
//! 이것은 하드 리미트가 아니라 **소프트 가드레일**입니다. 예산을 훨씬 넘는
//! 쿼리를 조기에 거부해서 서버 프로세스가 OOM killer에 걸리기 전에 보호하는
//! 것이 목적입니다. (PostgreSQL도 work_mem을 "정확한 메모리 측정"이 아닌
//! "작업 단위별 예산"으로 사용합니다.)

use std::ptr::NonNull;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::errors;
use crate::errors::execute_error::ExecuteError;

/// `tokio::task_local!`에 저장하기 위한 Copy 핸들 (#265).
///
/// `tokio::task_local!`은 `Copy` 타입만 지원하므로
/// `Option<Arc<QueryMemoryTracker>>`를 직접 저장할 수 없습니다.
/// `NonNull`은 `Copy`이므로 이를 감싸서 저장합니다.
///
/// # 안전성
/// 수명은 task-local scope로 제한됩니다. scope 내부에서
/// `Arc` 원본(`_keep_alive`)을 보유하므로 핸들이 가리키는 값은
/// scope 동안 항상 살아 있습니다. `query_memory()`에서
/// `Arc::increment_strong_count` 후 `Arc::from_raw`로 재구성하므로
/// refcount 관리가 정확합니다.
#[derive(Copy, Clone)]
pub(crate) struct QueryMemoryTrackerRef(pub(crate) Option<NonNull<QueryMemoryTracker>>);

// # Safety
// - `QueryMemoryTracker`는 `AtomicU64` + `u64`만 가지므로 `Send + Sync`입니다.
// - 포인터가 가리키는 값은 task-local scope 동안 `_keep_alive`(Arc)가
//   refcount를 보유하므로, task가 다른 스레드로 이동해도 유효합니다.
// - task-local은 task 내에서만 접근하므로 포인터가 다른 스레드에서
//   역참조되는 일은 없습니다 (`query_memory()`는 같은 task에서만 호출).
unsafe impl Send for QueryMemoryTrackerRef {}
unsafe impl Sync for QueryMemoryTrackerRef {}

/// 쿼리 메모리 예산 추적기.
///
/// 한 쿼리의 실행 동안 `reserve()`를 누적하고, 설정된 상한을 넘으면
/// `MemoryLimitExceeded` 에러를 반환합니다.
#[derive(Debug)]
pub struct QueryMemoryTracker {
    /// 현재까지 누적된 추정 메모리 (bytes).
    used: AtomicU64,
    /// 허용 상한 (bytes). `0`이면 비활성(추적만 하고 거부 안 함).
    limit_bytes: u64,
}

impl QueryMemoryTracker {
    /// 새 추적기를 생성합니다.
    pub fn new(limit_bytes: u64) -> Self {
        Self {
            used: AtomicU64::new(0),
            limit_bytes,
        }
    }

    /// 현재 누적 추정 메모리를 반환합니다.
    pub fn used_bytes(&self) -> u64 {
        self.used.load(Ordering::Relaxed)
    }

    /// 설정된 상한을 반환합니다.
    pub fn limit_bytes(&self) -> u64 {
        self.limit_bytes
    }

    /// 예산이 비활성(limit == 0)인지 여부.
    pub fn is_enabled(&self) -> bool {
        self.limit_bytes != 0
    }

    /// `amount` 바이트만큼 예산을 소비합니다. 상한을 초과하면 에러를 반환합니다.
    ///
    /// `is_enabled()`가 false면(limit == 0) 누적만 하고 절대 거부하지 않습니다.
    /// 누적 자체는 항상 하므로, `used_bytes()`는 비활성 상태에서도 관찰 가능합니다.
    pub fn reserve(&self, amount: u64) -> errors::Result<()> {
        if amount == 0 {
            return Ok(());
        }

        let new_used = self.used.fetch_add(amount, Ordering::Relaxed) + amount;

        if self.is_enabled() && new_used > self.limit_bytes {
            Err(ExecuteError::wrap(format!(
                "query memory limit exceeded: used ~{} bytes, limit {} bytes",
                new_used, self.limit_bytes
            )))
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_tracker_never_rejects() {
        let tracker = QueryMemoryTracker::new(0);
        // limit 0 = 비활성. 거대한 reserve도 에러 없이 통과.
        for _ in 0..1000 {
            assert!(tracker.reserve(1024 * 1024).is_ok());
        }
        assert_eq!(tracker.used_bytes(), 1000 * 1024 * 1024);
    }

    #[test]
    fn reserve_accumulates_used_bytes() {
        let tracker = QueryMemoryTracker::new(100);
        tracker.reserve(30).unwrap();
        tracker.reserve(20).unwrap();
        assert_eq!(tracker.used_bytes(), 50);
    }

    #[test]
    fn reserve_zero_is_noop() {
        let tracker = QueryMemoryTracker::new(100);
        tracker.reserve(0).unwrap();
        assert_eq!(tracker.used_bytes(), 0);
    }

    #[test]
    fn exceeding_limit_rejects_and_keeps_accumulated() {
        let tracker = QueryMemoryTracker::new(100);
        tracker.reserve(60).unwrap();
        let error = tracker.reserve(50).unwrap_err();

        let message = error.to_string();
        assert!(
            message.contains("query memory limit exceeded"),
            "unexpected error: {message}"
        );

        // 실패한 reserve도 누적값은 유지됩니다 (이미 원자적으로 더해졌으므로).
        // 이후 성공 reserve도 누적 기준은 계속 커집니다.
        assert_eq!(tracker.used_bytes(), 110);
    }

    #[test]
    fn exact_limit_is_allowed() {
        let tracker = QueryMemoryTracker::new(100);
        tracker.reserve(100).unwrap();
        assert_eq!(tracker.used_bytes(), 100);
    }

    #[test]
    fn boundary_over_limit_rejects() {
        let tracker = QueryMemoryTracker::new(100);
        tracker.reserve(100).unwrap();
        assert!(tracker.reserve(1).is_err());
    }

    #[test]
    fn enabled_tracker_reports_limit() {
        let tracker = QueryMemoryTracker::new(42);
        assert!(tracker.is_enabled());
        assert_eq!(tracker.limit_bytes(), 42);
    }

    #[test]
    fn disabled_tracker_reports_not_enabled() {
        let tracker = QueryMemoryTracker::new(0);
        assert!(!tracker.is_enabled());
        assert_eq!(tracker.limit_bytes(), 0);
    }
}

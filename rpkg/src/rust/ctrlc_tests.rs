//! Cooperative interrupt fixtures; enabled only with the ctrlc feature.

use miniextendr_api::prelude::*;
use std::sync::atomic::{AtomicI32, Ordering};
use std::time::{Duration, Instant};

static DROPS: AtomicI32 = AtomicI32::new(0);
struct DropCounter;
impl Drop for DropCounter {
    fn drop(&mut self) {
        DROPS.fetch_add(1, Ordering::SeqCst);
    }
}

fn wait_for_interrupt(seconds: i32) -> i32 {
    let _owned = DropCounter;
    let duration = Duration::from_secs(u64::try_from(seconds).expect("nonnegative seconds"));
    let deadline = Instant::now() + duration;
    miniextendr_api::r_println!("CTRLC_READY");
    while Instant::now() < deadline {
        check_interrupt();
        std::thread::sleep(Duration::from_millis(5));
    }
    42
}

/// Wait at cooperative checkpoints while owning a drop-counted resource.
/// @param seconds Maximum wait in seconds.
/// @noRd
#[miniextendr(noexport)]
pub fn ctrlc_wait(seconds: i32) -> i32 {
    wait_for_interrupt(seconds)
}

/// Worker variant of the cooperative interrupt fixture.
/// @param seconds Maximum wait in seconds.
#[cfg(feature = "worker-thread")]
/// @noRd
#[miniextendr(noexport, worker)]
pub fn ctrlc_wait_worker(seconds: i32) -> i32 {
    wait_for_interrupt(seconds)
}

/// Number of dropped interrupt fixture resources.
/// @noRd
#[miniextendr(noexport)]
pub fn ctrlc_drop_count() -> i32 {
    DROPS.load(Ordering::SeqCst)
}

/// Report whether R kept ownership of SIGINT.
/// @noRd
#[miniextendr(noexport)]
pub fn ctrlc_handler_installed() -> bool {
    miniextendr_api::ctrlc::handler_installed()
}

/// Exercise intentional interrupt transport without sending a signal.
/// @noRd
#[miniextendr(noexport)]
pub fn ctrlc_interrupt() {
    let _owned = DropCounter;
    std::panic::resume_unwind(Box::new(miniextendr_api::condition::RCondition::Interrupt));
}

/// Exercise interrupt transport after its trait-ABI tagged-value round trip.
/// @noRd
#[miniextendr(noexport)]
pub fn gc_stress_ctrlc_roundtrip() -> bool {
    let value = miniextendr_api::unwind_protect::with_r_unwind_protect_shim(|| {
        std::panic::resume_unwind(Box::new(miniextendr_api::condition::RCondition::Interrupt))
    });
    let _root = unsafe { OwnedProtect::new(value) };
    matches!(
        unsafe { miniextendr_api::condition::RCondition::from_tagged_sexp(value) },
        Some(miniextendr_api::condition::RCondition::Interrupt)
    )
}

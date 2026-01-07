#![allow(dead_code)]

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::{OnceLock, mpsc};

use miniextendr_api::thread::RThreadBuilder;

type Job = Box<dyn FnOnce() + Send + 'static>;

static R_THREAD: OnceLock<mpsc::Sender<Job>> = OnceLock::new();

fn initialize_r() {
    unsafe {
        let engine = miniextendr_engine::REngine::build()
            .with_args(&["R", "--quiet", "--vanilla"])
            .init()
            .expect("Failed to initialize R");
        // Initialize in same order as rpkg/src/entrypoint.c.in
        miniextendr_api::backtrace::miniextendr_panic_hook();
        miniextendr_api::worker::miniextendr_worker_init();
        disable_r_stack_checking();
        assert!(
            miniextendr_engine::r_initialized_sentinel(),
            "Rf_initialize_R did not set C stack sentinels"
        );
        std::mem::forget(engine);
    }
}

fn disable_r_stack_checking() {
    // R stack checks assume the main thread stack. Our tests run R on a
    // dedicated thread, so disable R's own stack checks to avoid false
    // overflow errors in CI (the OS stack limit still applies).
    #[cfg(feature = "nonapi")]
    {
        miniextendr_api::thread::disable_stack_checking_permanently();
    }

    #[cfg(not(feature = "nonapi"))]
    unsafe {
        extern "C" {
            static mut R_CStackLimit: usize;
        }
        R_CStackLimit = usize::MAX;
    }
}

pub fn with_r_thread<T, F>(f: F) -> T
where
    T: Send + 'static,
    F: FnOnce() -> T + Send + 'static,
{
    let sender = R_THREAD.get_or_init(|| {
        let (tx, rx) = mpsc::channel::<Job>();
        RThreadBuilder::new()
            .name("r-test-main".to_string())
            .stack_size(16 * 1024 * 1024)
            .spawn(move || {
                initialize_r();
                for job in rx {
                    job();
                }
            })
            .expect("Failed to spawn R test thread");
        tx
    });

    let (result_tx, result_rx) = mpsc::sync_channel(0);
    let job: Job = Box::new(move || {
        let result = catch_unwind(AssertUnwindSafe(f));
        let _ = result_tx.send(result);
    });

    sender.send(job).expect("R test thread stopped");
    match result_rx
        .recv()
        .expect("R test thread dropped the response")
    {
        Ok(value) => value,
        Err(panic) => std::panic::resume_unwind(panic),
    }
}

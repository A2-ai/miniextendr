//! Test fixtures for log crate → R console routing.

use miniextendr_api::miniextendr;

#[miniextendr(noexport)]
pub fn test_log_info(msg: &str) {
    log::info!("{msg}");
}

#[miniextendr(noexport)]
pub fn test_log_warn(msg: &str) {
    log::warn!("{msg}");
}

#[miniextendr(noexport)]
pub fn test_log_error(msg: &str) {
    log::error!("{msg}");
}

#[miniextendr(noexport)]
pub fn test_log_set_level(level: &str) {
    miniextendr_api::optionals::log_impl::set_log_level(level);
}

#[miniextendr(noexport)]
pub fn test_log_debug(msg: &str) {
    log::debug!("{msg}");
}

/// Log `msg` from a spawned thread. The record prints when this call returns to R.
#[miniextendr(noexport)]
pub fn test_log_info_from_thread(msg: &str) {
    std::thread::scope(|s| {
        s.spawn(|| log::info!("{msg}"));
    });
}

/// Log `n` numbered records (`flood 0`, `flood 1`, ...) from a spawned thread.
/// Past the log channel's capacity of 1024, the rest are dropped.
#[miniextendr(noexport)]
pub fn test_log_flood_from_thread(n: i32) {
    std::thread::scope(|s| {
        s.spawn(|| {
            for i in 0..n {
                log::info!("flood {i}");
            }
        });
    });
}

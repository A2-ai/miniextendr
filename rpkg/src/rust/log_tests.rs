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

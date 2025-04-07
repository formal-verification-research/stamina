use std::process::exit;

use creusot_contracts::trusted;

#[trusted]
pub(crate) fn message(s: &str) {
	eprintln!("[MESSAGE] {}", s);
}
#[trusted]
pub(crate) fn warning(s: &str) {
	eprintln!("[WARNING] {}", s);
}
#[trusted]
pub(crate) fn error(s: &str) {
	eprintln!("[ERROR] {}", s);
}
#[trusted]
pub(crate) fn error_and_exit(s: &str) {
	error(s);
	exit(1);
}
#[trusted]
pub(crate) fn debug_message(s: &str) {
	if cfg!(debug_assertions) {
		eprintln!("[DEBUG MESSAGE] {}", s);
	}
}

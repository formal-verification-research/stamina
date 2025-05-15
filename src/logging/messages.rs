use std::process::exit;

use metaverify::trusted;

#[trusted]
pub fn message(s: &str) {
	eprintln!("[MESSAGE] {}", s);
}
#[trusted]
pub fn warning(s: &str) {
	eprintln!("[WARNING] {}", s);
}
#[trusted]
pub fn error(s: &str) {
	eprintln!("[ERROR] {}", s);
}
#[trusted]
pub fn error_and_exit(s: &str) {
	error(s);
	exit(1);
}
#[trusted]
pub fn debug_message(s: &str) {
	if cfg!(debug_assertions) {
		eprintln!("[DEBUG MESSAGE] {}", s);
	}
}

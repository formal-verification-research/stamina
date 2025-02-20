use std::process::exit;


pub(crate) fn message(s: &str) {
	eprintln!("[MESSAGE] {}", s);
}

pub(crate) fn warning(s: &str) {
	eprintln!("[WARNING] {}", s);
}

pub(crate) fn error(s: &str) {
	eprintln!("[ERROR] {}", s);
}

pub(crate) fn error_and_exit(s: &str) {
	error(s);
	exit(1);
}

pub(crate) fn debug_message(s: &str) {
	if cfg!(debug_assertions) {
		eprintln!("{}", s);
	}
}

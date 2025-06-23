use std::process::exit;

macro_rules! message {
    ($($arg:tt)*) => {
        eprintln!("[MESSAGE] {}", format!($($arg)*));
    };
}

macro_rules! warning {
    ($($arg:tt)*) => {
        eprintln!("[WARNING] {}", format!($($arg)*));
    };
}

macro_rules! error {
    ($($arg:tt)*) => {
        eprintln!("[ERROR] {}", format!($($arg)*));
    };
}

macro_rules! error_and_exit {
    ($($arg:tt)*) => {
        error!($($arg)*);
        exit(1);
    };
}

macro_rules! debug_message {
    ($($arg:tt)*) => {
        if cfg!(debug_assertions) {
            eprintln!("[DEBUG MESSAGE] {}", format!($($arg)*));
        }
    };
}


#[macro_export]
macro_rules! message {
    ($($arg:tt)*) => {
        eprintln!("{}[MESSAGE]{} {}", $crate::logging::colors::COLOR_MESSAGE, $crate::logging::colors::COLOR_RESET, format!($($arg)*));
    };
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        eprintln!("{}[INFORMATION]{} {}", $crate::logging::colors::COLOR_INFO, $crate::logging::colors::COLOR_RESET, format!($($arg)*));
    };
}

#[macro_export]
macro_rules! warning {
    ($($arg:tt)*) => {
        eprintln!("{}[WARNING]{} {}", $crate::logging::colors::COLOR_WARNING, $crate::logging::colors::COLOR_RESET, format!($($arg)*));
    };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        eprintln!("{}[ERROR]{} {}", $crate::logging::colors::COLOR_ERROR, $crate::logging::colors::COLOR_RESET, format!($($arg)*));
    };
}

#[macro_export]
macro_rules! error_and_exit {
    ($($arg:tt)*) => {
        $crate::error!($($arg)*);
        $crate::exit(1);
    };
}

#[macro_export]
macro_rules! debug_message {
    ($($arg:tt)*) => {
        if cfg!(debug_assertions) {
            eprintln!("{}[DEBUG]{}   {}", $crate::logging::colors::COLOR_DEBUG, $crate::logging::colors::COLOR_RESET, format!($($arg)*));
        }
    };
}

pub use debug_message;
pub use error;
pub use message;
pub use warning;

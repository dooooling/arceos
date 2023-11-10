use crate::dprint::LogLevel::{DEBUG, DEV, INFO};

#[derive(PartialOrd, PartialEq)]
pub enum LogLevel {
    INFO,
    DEV,
    DEBUG,
}

impl Into<LogLevel> for &str {
    fn into(self) -> LogLevel {
        match self.to_lowercase().trim() {
            "dev" => DEV,
            "debug" => DEBUG,
            _ => INFO,
        }
    }
}
pub fn show(level: LogLevel) -> bool {
    let debug = if cfg!(debug_assertions) { true } else { false };
    debug && level <= option_env!("AX_LOG").unwrap().into()
}

#[macro_export]
macro_rules! pinfo {
    () => { $crate::print!("\n") };
    ($($arg:tt)*) => {
        if $crate::dprint::show($crate::dprint::LogLevel::INFO){
            $crate::io::__print_impl(format_args!("\x1b[32m[ INFO] {}\x1b[0m\n", format_args!($($arg)*)));
        }
    }
}

#[macro_export]
macro_rules! pdev {
    () => {
        $crate::print!("\n")
    };
    ($($arg:tt)*) => {
        if $crate::dprint::show($crate::dprint::LogLevel::DEV){
            $crate::io::__print_impl(format_args!("\x1b[36m[  DEV] {}\x1b[0m\n", format_args!($($arg)*)));
        }
    };
}

#[macro_export]
macro_rules! pdebug {
    () => {
        $crate::print!("\n")
    };
    ($($arg:tt)*) => {
        if $crate::dprint::show($crate::dprint::LogLevel::DEBUG){
           $crate::io::__print_impl(format_args!("\x1b[34m[DEBUG] {}\x1b[0m\n", format_args!($($arg)*)));
        }
    };
}

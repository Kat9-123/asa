
enum Type {
    INFO,
    WARN,
    ERROR,
}

#[macro_export]
macro_rules! asm_error {
    ($($arg:tt)*) => {
        log::error!($($arg)*);
        panic!();
    };
}
#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        log::warn!($($arg)*);
    };
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        log::info!($($arg)*);
    };
}


pub(crate) use asm_error;

use crate::tokens::Info;
fn send() {

}

pub fn asm_err(msg: &String, info: &Info) {
    log::error!("Line {}: {}", info.line_number, msg);
    panic!();
}
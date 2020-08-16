use {
    core::fmt,
    log::{self, Level, LevelFilter, Log, Metadata, Record},
};

use crate::sbi::*;
use core::fmt::Write;

pub fn init(level: &str) {
    static LOGGER: SimpleLogger = SimpleLogger;
    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(match level {
        "error" => LevelFilter::Error,
        "warn" => LevelFilter::Warn,
        "info" => LevelFilter::Info,
        "debug" => LevelFilter::Debug,
        "trace" => LevelFilter::Trace,
        _ => LevelFilter::Off,
    });
}

/* #[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ({
        $crate::logging::print(format_args!($($arg)*));
    });
}

#[macro_export]
macro_rules! println {
    ($fmt:expr) => (print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
} */

/// Add escape sequence to print with color in Linux console
macro_rules! with_color {
    ($args: ident, $color_code: ident) => {{
        format_args!("\u{1B}[{}m{}\u{1B}[0m", $color_code as u8, $args)
    }};
}

fn print_in_color(args: fmt::Arguments, color_code: u8) {
    crate::kernel_hal_bare::arch::putfmt(with_color!(args, color_code));
}

/* #[allow(dead_code)]
pub fn print(args: fmt::Arguments) {
    crate::kernel_hal_bare::arch::putfmt(args);
} */

struct SimpleLogger;

impl Log for SimpleLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }
    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let (tid, pid) = crate::kernel_hal_bare::Thread::get_tid();
        print_in_color(
            format_args!(
                "[{:?} {:>5} {} {}:{}] {}\n",
                crate::kernel_hal_bare::timer_now(),
                record.level(),
                crate::kernel_hal_bare::apic_local_id(),
                pid,
                tid,
                record.args()
            ),
            level_to_color_code(record.level()),
        );
    }
    fn flush(&self) {}
}

fn level_to_color_code(level: Level) -> u8 {
    match level {
        Level::Error => 31, // Red
        Level::Warn => 93,  // BrightYellow
        Level::Info => 34,  // Blue
        Level::Debug => 32, // Green
        Level::Trace => 90, // BrightBlack
    }
}

/// 一个 [Zero-Sized Type]，实现 [`core::fmt::Write`] trait 来进行格式化输出
/// 
/// ZST 只可能有一个值（即为空），因此它本身就是一个单件
struct Stdout;

impl Write for Stdout {
    /// 打印一个字符串
    /// 
    /// 对于每一个字符调用 [`console_putchar`]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            console_putchar(c as usize);
        }
        Ok(())
    }
}

/// 打印由 [`core::format_args!`] 格式化后的数据
/// 
/// [`print!`] 和 [`println!`] 宏都将展开成此函数
/// 
/// [`core::format_args!`]: https://doc.rust-lang.org/nightly/core/macro.format_args.html
pub fn print(args: fmt::Arguments) {
    Stdout.write_fmt(args).unwrap();
}

/// 实现类似于标准库中的 `print!` 宏
/// 
/// 使用实现了 [`core::fmt::Write`] trait 的 [`console::Stdout`]
#[macro_export]
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::logging::print(format_args!($fmt $(, $($arg)+)?));
    }
}

/// 实现类似于标准库中的 `println!` 宏
/// 
/// 使用实现了 [`core::fmt::Write`] trait 的 [`console::Stdout`]
#[macro_export]
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::logging::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    }
}
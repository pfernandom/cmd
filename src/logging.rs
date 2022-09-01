use std::io::Write;
use termcolor::{ Color, ColorChoice, ColorSpec, StandardStream, WriteColor };

pub fn print_text(color: Color, text: &str) {
    format!("{}", "yes");
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    stdout.set_color(ColorSpec::new().set_fg(Some(color))).unwrap();
    writeln!(&mut stdout, "{}", text).unwrap();
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Black))).unwrap();
}

#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        let st = format!($($arg)*);
        $crate::logging::print_text(termcolor::Color::Rgb(128,128,128), &st);
    };
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        let st = format!($($arg)*);
        $crate::logging::print_text(termcolor::Color::Green, &st);
    };
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        let st = format!($($arg)*);
        $crate::logging::print_text(termcolor::Color::Yellow, &st);
    };
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        let st = format!($($arg)*);
        $crate::logging::print_text(termcolor::Color::Red, &st);
    };
}
use std::io::Write;
use termcolor::{ Color, ColorChoice, ColorSpec, StandardStream, WriteColor };

pub fn print_text(color: Color, text: &str) {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    stdout.set_color(ColorSpec::new().set_fg(Some(color))).unwrap();
    writeln!(&mut stdout, "\n{}", text).unwrap();
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Black))).unwrap();
    stdout.flush().unwrap();
}

#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        if log::log_enabled!(log::Level::Debug) {
            let st = format!($($arg)*);
            println!("{} {}",file!(), line!());
            $crate::logging::print_text(termcolor::Color::Rgb(128,128,128), &format!("{}:({}):{}", file!(), line!(), &st));
        }
    };
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        if log::log_enabled!(log::Level::Info) {
            let st = format!($($arg)*);
            $crate::logging::print_text(termcolor::Color::Green, &st);
        }
    };
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        if log::log_enabled!(log::Level::Info) {
            let st = format!($($arg)*);
            $crate::logging::print_text(termcolor::Color::Yellow, &st);
        }
    };
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        let st = format!($($arg)*);
        $crate::logging::print_text(termcolor::Color::Red, &st);
    };
}
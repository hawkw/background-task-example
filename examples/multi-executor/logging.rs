#![deny(warnings)]
#![deny(missing_docs)]

//! A modified version of the `pretty_env_logger` crate that prints
//! the current thread name.

use std::fmt::{self, Write};
use std::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};

use ansi_term::{Color, Style};
use env_logger::Builder;
use log::{self, Level};

struct ColorLevel(Level);

impl fmt::Display for ColorLevel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.0 {
            Level::Trace => Color::Purple.paint("TRACE"),
            Level::Debug => Color::Blue.paint("DEBUG"),
            Level::Info => Color::Green.paint("INFO "),
            Level::Warn => Color::Yellow.paint("WARN "),
            Level::Error => Color::Red.paint("ERROR")
        }.fmt(f)
    }
}

static MAX_MODULE_WIDTH: AtomicUsize = ATOMIC_USIZE_INIT;

/// Returns a `env_logger::Builder` for further customization.
///
/// This method will return a colored and formatted) `env_logger::Builder`
/// for further customization. Tefer to env_logger::Build crate documentation
/// for further details and usage.
///
/// This should be called early in the execution of a Rust program, and the
/// global logger may only be initialized once. Future initialization attempts
/// will return an error.
///
/// # Errors
///
/// This function fails to set the global logger if one has already been set.
pub fn formatted_builder() -> Result<Builder, log::SetLoggerError> {
    let mut builder = Builder::new();

    builder.format(|f, record| {
        use std::io::Write;
        use std::thread;
        let mut info = String::new();
        if let Some(module_path) = record.module_path() {
            write!(&mut info, "{}", Style::new().bold().paint(module_path)).unwrap();
        }
        if let Some(thread_name) = thread::current().name() {
            write!(&mut info, " (thread: {})", thread_name).unwrap();
        }
        let mut max_width = MAX_MODULE_WIDTH.load(Ordering::Relaxed);
        if max_width < info.len() {
            MAX_MODULE_WIDTH.store(info.len(), Ordering::Relaxed);
            max_width = info.len();
        }
        writeln!(f, " {} {: <width$} > {args}",
                ColorLevel(record.level()),
                info, width=max_width,
                args=record.args())
    });

    Ok(builder)
}

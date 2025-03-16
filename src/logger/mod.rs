use slog::{Drain, Logger};
use slog_async::Async;
use slog_scope::GlobalLoggerGuard;
use slog_term::{FullFormat, PlainDecorator};
use std::env;
use std::fs::OpenOptions;
use std::path::PathBuf;

#[allow(dead_code)]
pub enum LoggerMode {
    Console,
    File,
    Both,
}

pub fn init_logger(mode: LoggerMode) -> GlobalLoggerGuard {
    let drain: Box<dyn Drain<Ok = (), Err = slog::Never> + Send> = match mode {
        LoggerMode::Console => {
            let decorator = PlainDecorator::new(std::io::stdout());
            Box::new(FullFormat::new(decorator).build().fuse())
        }
        LoggerMode::File => {
            let mut log_path: PathBuf = env::temp_dir();
            log_path.push("netsync.log");
            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&log_path)
                .expect("Failed to open logging file in temp folder");
            let decorator = PlainDecorator::new(file);
            Box::new(FullFormat::new(decorator).build().fuse())
        }
        LoggerMode::Both => {
            // Drain for stdout.
            let decorator_stdout = PlainDecorator::new(std::io::stdout());
            let drain_stdout = FullFormat::new(decorator_stdout).build().fuse();
            // Drain for file.
            let mut log_path: PathBuf = env::temp_dir();
            log_path.push("netsync.log");
            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&log_path)
                .expect("Failed to open logging file in temp folder");
            let decorator_file = PlainDecorator::new(file);
            let drain_file = FullFormat::new(decorator_file).build().fuse();
            Box::new(slog::Duplicate(drain_stdout, drain_file).fuse())
        }
    };

    // Convert the drain’s error type and wrap it in an asynchronous drain.
    let drain = drain.map_err(|_| unreachable!()).fuse();
    let drain_async = Async::new(drain).build().fuse();

    // Create the root logger.
    let root_logger = Logger::root(
        drain_async,
        slog::o!("version" => env!("CARGO_PKG_VERSION")),
    );

    // Initialize the slog-stdlog bridge so standard log macros route to slog.
    slog_stdlog::init().unwrap();

    // Set this logger as the global logger.
    slog_scope::set_global_logger(root_logger)
}

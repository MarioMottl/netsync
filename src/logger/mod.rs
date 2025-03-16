use slog::{Drain, Logger};
use slog_async::Async;
use slog_scope::GlobalLoggerGuard;
use slog_term::{FullFormat, PlainDecorator};
use std::env;
use std::fs::OpenOptions;

pub fn init_logger() -> GlobalLoggerGuard {
    if env::var("RUST_LOG").is_err() {
        unsafe { env::set_var("RUST_LOG", "info") }
    }
    let decorator_stdout = PlainDecorator::new(std::io::stdout());
    let drain_stdout = FullFormat::new(decorator_stdout).build().fuse();

    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("netsync.log")
        .expect("Failed to open netsync.log");
    let decorator_file = PlainDecorator::new(file);
    let drain_file = FullFormat::new(decorator_file).build().fuse();

    let drain_all = slog::Duplicate(drain_stdout, drain_file).fuse();

    let drain_async = Async::new(drain_all).build().fuse();

    let root_logger = Logger::root(
        drain_async,
        slog::o!("version" => env!("CARGO_PKG_VERSION")),
    );

    slog_stdlog::init().unwrap();

    slog_scope::set_global_logger(root_logger)
}

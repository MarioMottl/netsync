use std::env;

pub fn init_logger(level: &str) {
    if env::var("RUST_LOG").is_err() {
        unsafe { env::set_var("RUST_LOG", level) }
    }
    env_logger::init();
}

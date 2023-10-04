use eyre::Result;

pub fn with_status<T, F: FnOnce() -> Result<T>>(msg: &str, op: F) -> Result<T> {
    log_status(msg);

    let result = op();

    if result.is_ok() {
        log_ok();
    } else {
        log_error();
    }

    result
}

fn log_status(msg: &str) {
    eprint!("> {}...", msg);
}

fn log_ok() {
    eprintln!("\x1b[1;32m OK\x1b[0m");
}

fn log_error() {
    eprintln!("\x1b[1;31m ERROR\x1b[0m\n");
}

use eyre::Result;

pub fn with_status<T, F: FnOnce() -> Result<T>>(msg: &str, op: F) -> Result<T> {
    eprint!("> {}...", msg);

    let result = op();

    match result {
        Ok(_) => eprintln!("\x1b[1;32m OK\x1b[0m"),
        Err(_) => eprintln!("\x1b[1;31m ERROR\x1b[0m\n"),
    }

    result
}

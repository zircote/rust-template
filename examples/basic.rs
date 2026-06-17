//! Basic usage of the `rust_template` example API.
//!
//! Run it with:
//!
//! ```sh
//! cargo run --example basic
//! ```
//!
//! NOTE: `add`, `divide`, `process`, and `Config` are placeholder example
//! API shipped with the template. Replace them with your crate's real
//! surface — and update this example accordingly.

// Examples demonstrate output, so writing to stdout here is intentional.
#![allow(clippy::print_stdout)]

use rust_template::{Config, Result, add, divide, process};

fn main() -> Result<()> {
    // Pure, infallible arithmetic.
    println!("add(2, 3)     = {}", add(2, 3));

    // Fallible operations return `rust_template::Result`; `?` propagates the
    // crate's `Error` type.
    println!("divide(10, 2) = {}", divide(10, 2)?);
    println!("process(\"42\") = {}", process("42")?);

    // Consuming-self builder: each `with_*` returns `Self`, so calls chain.
    let config = Config::new()
        .with_verbose(true)
        .with_max_retries(5)
        .with_timeout(30);
    println!(
        "config: verbose={}, max_retries={}, timeout_secs={}",
        config.verbose(),
        config.max_retries(),
        config.timeout_secs(),
    );

    Ok(())
}

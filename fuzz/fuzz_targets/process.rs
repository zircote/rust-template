#![no_main]

use libfuzzer_sys::fuzz_target;

// Fuzz the public `process` parser. It takes an untrusted `&str`, which makes
// it the natural fuzz surface for this crate. The harness drives it with
// arbitrary bytes and asserts the documented contract: `process` never panics,
// and on success it always returns a non-negative value.
fuzz_target!(|data: &[u8]| {
    if let Ok(input) = std::str::from_utf8(data) {
        if let Ok(value) = rust_template::process(input) {
            assert!(value >= 0, "process({input:?}) returned a negative value: {value}");
        }
    }
});

#![no_main]

use klipper_proto::parser::Parser;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // This fuzz target feeds arbitrary byte slices into the Klipper protocol
    // parser. The goal is to ensure that no input can cause a panic, crash,
    // or undefined behavior, regardless of how malformed it is.

    let parser = Parser::new();
    
    // The parser is designed to return Ok(None) or Err((...)) on invalid
    // data, but it should never panic. The fuzzer will automatically detect
    // if a panic occurs.
    let _ = parser.parse(data);
});

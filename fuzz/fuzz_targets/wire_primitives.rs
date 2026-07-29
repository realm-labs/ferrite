#![no_main]

use ferrite_protocol::java_26_2::wire::primitive::{WireReader, WireWriter};
use ferrite_protocol::java_26_2::wire::varint::{decode_i32, decode_i64};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = decode_i32(data);
    let _ = decode_i64(data);

    let mut reader = WireReader::new(data);
    let decoded_i32 = reader.read_var_i32();
    let decoded_i64 = reader.read_var_i64();
    let decoded_utf = reader.read_utf(64);
    let decoded_bytes = reader.read_byte_array(256);

    let mut writer = WireWriter::new(512);
    if let Ok(value) = decoded_i32 {
        let _ = writer.write_var_i32(value);
    }
    if let Ok(value) = decoded_i64 {
        let _ = writer.write_var_i64(value);
    }
    if let Ok(value) = decoded_utf {
        let _ = writer.write_utf(&value, 64);
    }
    if let Ok(value) = decoded_bytes {
        let _ = writer.write_byte_array(value, 256);
    }
    assert!(writer.len() <= 512);
});

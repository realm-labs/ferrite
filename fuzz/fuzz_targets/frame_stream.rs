#![no_main]

use ferrite_protocol::java_26_2::wire::compression::CompressionMode;
use ferrite_protocol::java_26_2::wire::frame::FrameLimits;
use ferrite_protocol::java_26_2::wire::stream::PacketStreamDecoder;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let limits = FrameLimits::new(4_096, 8_198).expect("fixed fuzz limits are valid");
    let mode = data
        .first()
        .and_then(|threshold| CompressionMode::enabled(usize::from(*threshold)).ok())
        .unwrap_or(CompressionMode::Disabled);
    let mut decoder = PacketStreamDecoder::new(limits, mode);
    for chunk in data.chunks(17) {
        if decoder.push(chunk).is_err() {
            break;
        }
        while matches!(decoder.next_packet(), Ok(Some(_))) {}
        if decoder.is_faulted() {
            break;
        }
    }
    if !decoder.is_faulted() {
        let _ = decoder.finish();
    }
    assert!(decoder.buffered_bytes() <= limits.maximum_buffered_bytes());
});

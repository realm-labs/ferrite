#![no_main]

use ferrite_replay::codec::{
    CanonicalDecode, CanonicalEncode, Decoder, decode_exact, encode_to_vec,
};
use ferrite_replay::envelope::{CommandEnvelope, EventEnvelope};
use ferrite_replay::log::{ReplayFrame, ReplayHeader, ReplayLog};
use libfuzzer_sys::fuzz_target;
use std::fmt::Debug;

fn round_trip<T>(data: &[u8])
where
    T: CanonicalDecode + CanonicalEncode + Debug + Eq,
{
    if let Ok(value) = decode_exact::<T>(data) {
        let encoded = encode_to_vec(&value).unwrap();
        assert_eq!(decode_exact::<T>(&encoded).unwrap(), value);
    }
}

fuzz_target!(|data: &[u8]| {
    round_trip::<CommandEnvelope>(data);
    round_trip::<EventEnvelope>(data);
    round_trip::<ReplayHeader>(data);
    round_trip::<ReplayFrame>(data);
    round_trip::<ReplayLog>(data);

    let mut decoder = Decoder::new(data);
    let _ = decoder.read_bool();
    let _ = decoder.read_var_u64();
    let _ = decoder.read_f32();
    let _ = decoder.read_f64();
    let _ = decoder.read_string(1024);
    let _ = decoder.finish();
});

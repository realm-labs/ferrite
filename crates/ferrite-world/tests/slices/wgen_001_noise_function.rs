use std::sync::{Arc, Mutex};

use ferrite_world::generation::density::{DensityBounds, DensityContext, DensityLeaf};
use ferrite_world::generation::noise_function::{
    NoiseFunction, NoiseHolder, NoiseSampler, NoiseWiringCache, NoiseWiringError, ShiftKind,
};

#[test]
fn unwired_holder_returns_zero_with_provisional_two_bound() {
    let function = NoiseFunction::Noise {
        holder: Arc::new(NoiseHolder::unwired()),
        xz_scale: -2.0,
        y_scale: 0.0,
    };

    assert_eq!(function.sample(context()), 0.0);
    assert_eq!(
        function.bounds(),
        DensityBounds {
            minimum: -2.0,
            maximum: 2.0,
        }
    );
}

#[test]
fn all_three_shift_permutations_use_quarter_scale_then_times_four() {
    let sampler = Arc::new(RecordingNoise::new());
    let holder = Arc::new(NoiseHolder::wired(sampler.clone()));
    for kind in [ShiftKind::Shift, ShiftKind::ShiftA, ShiftKind::ShiftB] {
        let function = NoiseFunction::Shift {
            holder: holder.clone(),
            kind,
        };
        assert_eq!(function.sample(context()), 4.0);
    }

    assert_eq!(
        *sampler.calls.lock().unwrap(),
        [(2.0, 3.0, 4.0), (2.0, 0.0, 4.0), (4.0, 2.0, 0.0)]
    );
}

#[test]
fn shifted_noise_evaluates_x_then_y_then_z_before_sampling() {
    let order = Arc::new(Mutex::new(Vec::new()));
    let sampler = Arc::new(RecordingNoise::new());
    let function = NoiseFunction::ShiftedNoise {
        holder: Arc::new(NoiseHolder::wired(sampler.clone())),
        shift_x: Arc::new(ProbeShift::new("x", 1.0, order.clone())),
        shift_y: Arc::new(ProbeShift::new("y", 2.0, order.clone())),
        shift_z: Arc::new(ProbeShift::new("z", 3.0, order.clone())),
        xz_scale: 0.5,
        y_scale: -1.0,
    };

    assert_eq!(function.sample(context()), 1.0);
    assert_eq!(*order.lock().unwrap(), ["x", "y", "z"]);
    assert_eq!(*sampler.calls.lock().unwrap(), [(5.0, -10.0, 11.0)]);
}

#[test]
fn keyed_wiring_reuses_identity_and_rejects_direct_holders() {
    let mut cache = NoiseWiringCache::default();
    let creations = Arc::new(Mutex::new(0));
    let first = cache
        .wire(Some("minecraft:test"), |_| {
            *creations.lock().unwrap() += 1;
            Arc::new(RecordingNoise::new())
        })
        .unwrap();
    let second = cache
        .wire(Some("minecraft:test"), |_| {
            panic!("cached key must not recreate")
        })
        .unwrap();

    assert!(Arc::ptr_eq(&first, &second));
    assert_eq!(*creations.lock().unwrap(), 1);
    assert_eq!(cache.len(), 1);
    assert!(matches!(
        cache.wire(None, |_| Arc::new(RecordingNoise::new())),
        Err(NoiseWiringError::DirectHolder)
    ));
}

fn context() -> DensityContext {
    DensityContext { x: 8, y: 12, z: 16 }
}

#[derive(Debug)]
struct RecordingNoise {
    calls: Mutex<Vec<(f64, f64, f64)>>,
}

impl RecordingNoise {
    fn new() -> Self {
        Self {
            calls: Mutex::new(Vec::new()),
        }
    }
}

impl NoiseSampler for RecordingNoise {
    fn sample(&self, x: f64, y: f64, z: f64) -> f64 {
        self.calls.lock().unwrap().push((x, y, z));
        1.0
    }

    fn maximum(&self) -> f64 {
        3.0
    }
}

#[derive(Debug)]
struct ProbeShift {
    name: &'static str,
    value: f64,
    order: Arc<Mutex<Vec<&'static str>>>,
}

impl ProbeShift {
    fn new(name: &'static str, value: f64, order: Arc<Mutex<Vec<&'static str>>>) -> Self {
        Self { name, value, order }
    }
}

impl DensityLeaf for ProbeShift {
    fn sample(&self, _context: DensityContext) -> f64 {
        self.order.lock().unwrap().push(self.name);
        self.value
    }

    fn bounds(&self) -> DensityBounds {
        DensityBounds {
            minimum: self.value,
            maximum: self.value,
        }
    }
}

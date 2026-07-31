use ferrite_world::generation::density::DensityContext;
use ferrite_world::generation::density_cache::{
    Cache2D, CacheAllInCell, CacheOnce, DensityCacheError, DensityRuntime, FlatCache, Interpolated,
};

#[test]
fn cache_2d_initial_sentinel_returns_zero_and_array_fill_bypasses_cache() {
    let mut cache = Cache2D::default();
    let sentinel = DensityContext {
        x: 1_875_066,
        y: 9,
        z: 1_875_066,
    };
    assert_eq!(
        cache.sample(sentinel, |_| panic!("sentinel is a cache hit")),
        0.0
    );

    let mut output = [0.0];
    cache.fill(&[sentinel], &mut output, |context| f64::from(context.y));
    assert_eq!(output, [9.0]);
}

#[test]
fn flat_cache_uses_arithmetic_quarts_and_falls_back_outside_rectangle() {
    let cache = FlatCache::new(-1, -1, 1, |context| f64::from(context.x * 100 + context.z));

    assert_eq!(
        cache.sample(DensityContext { x: -1, y: 7, z: -1 }, |_| 99.0),
        -404.0
    );
    assert_eq!(
        cache.sample(DensityContext { x: 8, y: 7, z: 8 }, |_| 99.0),
        99.0
    );
}

#[test]
fn cache_once_preserves_initial_zero_then_prioritizes_current_array() {
    let mut cache = CacheOnce::default();
    assert_eq!(
        cache.sample(context(3), runtime(), |_| panic!("initial epoch hits zero")),
        0.0
    );
    let contexts = [context(3), context(4)];
    let mut output = [0.0; 2];
    let mut array_runtime = runtime();
    array_runtime.array_counter = 1;
    cache.fill(&contexts, &mut output, array_runtime, |value| {
        f64::from(value.y)
    });
    assert_eq!(output, [3.0, 4.0]);

    array_runtime.array_index = 1;
    assert_eq!(
        cache.sample(context(99), array_runtime, |_| panic!("array has priority")),
        4.0
    );
}

#[test]
fn all_in_cell_fills_reverse_y_and_rejects_owner_outside_lifecycle() {
    let mut cache = CacheAllInCell::new(2, 2);
    cache.fill(context(10), |value| {
        f64::from(value.y * 100 + value.x * 10 + value.z)
    });
    let mut active = runtime();
    active.interpolation_running = true;

    assert_eq!(
        cache.sample(DensityContext { x: 0, y: 11, z: 0 }, active, |_| -1.0),
        Ok(1_100.0)
    );
    assert_eq!(cache.sample(context(10), active, |_| -1.0), Ok(1_000.0));
    assert_eq!(
        cache.sample(context(10), runtime(), |_| -1.0),
        Err(DensityCacheError::OutsideInterpolation)
    );
}

#[test]
fn interpolator_direct_and_staged_trilinear_paths_match_and_lifecycle_is_strict() {
    let mut interpolator = Interpolated::new(0, 0, 0, 4, 8, 1, 1);
    let mut child = |value: DensityContext| f64::from(value.x + value.y + value.z);
    interpolator.start(&mut child).unwrap();
    assert_eq!(
        interpolator.start(&mut child),
        Err(DensityCacheError::InterpolationAlreadyRunning)
    );
    interpolator.advance_x(0, &mut child).unwrap();
    interpolator.select_cell(0, 0).unwrap();
    interpolator.update_y(0.5);
    interpolator.update_x(0.5);
    interpolator.update_z(0.5);
    let mut active = runtime();
    active.interpolation_running = true;

    assert_eq!(
        interpolator
            .sample(context(0), active, |_| -1.0, [0.5, 0.5, 0.5])
            .unwrap(),
        8.0
    );
    active.filling_cell = true;
    assert_eq!(
        interpolator
            .sample(context(0), active, |_| -1.0, [0.5, 0.5, 0.5])
            .unwrap(),
        8.0
    );
    interpolator.swap_slices().unwrap();
    interpolator.stop().unwrap();
    assert_eq!(
        interpolator.stop(),
        Err(DensityCacheError::InterpolationNotRunning)
    );
}

fn context(y: i32) -> DensityContext {
    DensityContext { x: 0, y, z: 0 }
}

fn runtime() -> DensityRuntime {
    DensityRuntime {
        owner: true,
        interpolation_running: false,
        filling_cell: false,
        interpolation_counter: 0,
        array_counter: 0,
        array_index: 0,
    }
}

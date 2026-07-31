use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use ferrite_world::generation::density::{
    BinaryOperation, DensityBounds, DensityContext, DensityExpr, DensityLeaf, Spline, SplinePoint,
    SplineValue, UnaryOperation,
};

#[test]
fn multiply_zero_and_minimum_declared_bound_short_circuit_second_argument() {
    let zero_second = Arc::new(Probe::new(7.0, 7.0, 7.0));
    let multiply = DensityExpr::Binary {
        operation: BinaryOperation::Multiply,
        first: Box::new(DensityExpr::Constant(-0.0)),
        second: Box::new(DensityExpr::Leaf(zero_second.clone())),
    };
    assert_eq!(
        multiply.sample(context(0)).unwrap().to_bits(),
        0.0_f64.to_bits()
    );
    assert_eq!(zero_second.calls.load(Ordering::SeqCst), 0);

    let min_second = Arc::new(Probe::new(3.0, 2.0, 4.0));
    let minimum = DensityExpr::Binary {
        operation: BinaryOperation::Minimum,
        first: Box::new(DensityExpr::Constant(1.0)),
        second: Box::new(DensityExpr::Leaf(min_second.clone())),
    };
    assert_eq!(minimum.sample(context(0)).unwrap(), 1.0);
    assert_eq!(min_second.calls.load(Ordering::SeqCst), 0);
}

#[test]
fn range_and_interval_endpoints_select_exactly_one_branch() {
    let range = DensityExpr::RangeChoice {
        input: Box::new(DensityExpr::Constant(2.0)),
        minimum_inclusive: 1.0,
        maximum_exclusive: 2.0,
        in_range: Box::new(DensityExpr::Constant(10.0)),
        out_of_range: Box::new(DensityExpr::Constant(20.0)),
    };
    assert_eq!(range.sample(context(0)).unwrap(), 20.0);

    let interval = DensityExpr::IntervalSelect {
        input: Box::new(DensityExpr::Constant(2.0)),
        thresholds: vec![1.0, 2.0],
        intervals: vec![
            DensityExpr::Constant(10.0),
            DensityExpr::Constant(20.0),
            DensityExpr::Constant(30.0),
        ],
    };
    assert_eq!(interval.sample(context(0)).unwrap(), 30.0);
}

#[test]
fn mapped_functions_preserve_java_signed_zero_and_nan_behavior() {
    let half = unary(UnaryOperation::HalfNegative, -0.0);
    assert!(half.sample(context(0)).unwrap().is_sign_negative());

    let invert = unary(UnaryOperation::Invert, -0.0);
    assert_eq!(invert.sample(context(0)).unwrap(), f64::NEG_INFINITY);

    let clamp_nan = DensityExpr::Clamp {
        input: Box::new(DensityExpr::Constant(f64::NAN)),
        minimum: -1.0,
        maximum: 1.0,
    };
    assert!(clamp_nan.sample(context(0)).unwrap().is_nan());
}

#[test]
fn top_surface_rounds_down_then_uses_strict_positive_density() {
    let density = DensityExpr::YClampedGradient {
        from_y: 8,
        to_y: 16,
        from_value: 0.0,
        to_value: 1.0,
    };
    let top = DensityExpr::FindTopSurface {
        density: Box::new(density),
        upper_bound: Box::new(DensityExpr::Constant(19.0)),
        lower_bound: -8,
        cell_height: 8,
    };
    assert_eq!(top.sample(context(0)).unwrap(), 16.0);

    let strict_zero = DensityExpr::FindTopSurface {
        density: Box::new(DensityExpr::Constant(0.0)),
        upper_bound: Box::new(DensityExpr::Constant(8.0)),
        lower_bound: 0,
        cell_height: 8,
    };
    assert_eq!(strict_zero.sample(context(0)).unwrap(), 0.0);
}

#[test]
fn spline_uses_float_hermite_interpolation_and_endpoint_extension() {
    let spline = DensityExpr::Spline(Spline {
        coordinate: Box::new(DensityExpr::Leaf(Arc::new(YCoordinate))),
        points: vec![
            SplinePoint {
                location: 0.0,
                value: SplineValue::Constant(2.0),
                derivative: 0.5,
            },
            SplinePoint {
                location: 10.0,
                value: SplineValue::Constant(8.0),
                derivative: 0.25,
            },
        ],
    });

    assert_eq!(spline.sample(context(5)).unwrap(), 5.3125);
    assert_eq!(spline.sample(context(-1)).unwrap(), 1.5);
}

fn unary(operation: UnaryOperation, value: f64) -> DensityExpr {
    DensityExpr::Unary {
        operation,
        input: Box::new(DensityExpr::Constant(value)),
    }
}

fn context(y: i32) -> DensityContext {
    DensityContext { x: 0, y, z: 0 }
}

#[derive(Debug)]
struct Probe {
    value: f64,
    bounds: DensityBounds,
    calls: AtomicUsize,
}

impl Probe {
    fn new(value: f64, minimum: f64, maximum: f64) -> Self {
        Self {
            value,
            bounds: DensityBounds { minimum, maximum },
            calls: AtomicUsize::new(0),
        }
    }
}

impl DensityLeaf for Probe {
    fn sample(&self, _context: DensityContext) -> f64 {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.value
    }

    fn bounds(&self) -> DensityBounds {
        self.bounds
    }
}

#[derive(Debug)]
struct YCoordinate;

impl DensityLeaf for YCoordinate {
    fn sample(&self, context: DensityContext) -> f64 {
        f64::from(context.y)
    }

    fn bounds(&self) -> DensityBounds {
        DensityBounds {
            minimum: f64::from(i32::MIN),
            maximum: f64::from(i32::MAX),
        }
    }
}

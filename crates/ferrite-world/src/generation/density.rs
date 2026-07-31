//! Pure, seed-independent density-function composition.

use std::sync::Arc;

use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DensityContext {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DensityBounds {
    pub minimum: f64,
    pub maximum: f64,
}

pub trait DensityLeaf: Send + Sync {
    fn sample(&self, context: DensityContext) -> f64;

    fn bounds(&self) -> DensityBounds;
}

#[derive(Clone)]
pub enum DensityExpr {
    Leaf(Arc<dyn DensityLeaf>),
    Constant(f64),
    YClampedGradient {
        from_y: i32,
        to_y: i32,
        from_value: f64,
        to_value: f64,
    },
    Clamp {
        input: Box<Self>,
        minimum: f64,
        maximum: f64,
    },
    Unary {
        operation: UnaryOperation,
        input: Box<Self>,
    },
    Binary {
        operation: BinaryOperation,
        first: Box<Self>,
        second: Box<Self>,
    },
    RangeChoice {
        input: Box<Self>,
        minimum_inclusive: f64,
        maximum_exclusive: f64,
        in_range: Box<Self>,
        out_of_range: Box<Self>,
    },
    IntervalSelect {
        input: Box<Self>,
        thresholds: Vec<f64>,
        intervals: Vec<Self>,
    },
    FindTopSurface {
        density: Box<Self>,
        upper_bound: Box<Self>,
        lower_bound: i32,
        cell_height: u32,
    },
    Spline(Spline),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOperation {
    Abs,
    Square,
    Cube,
    HalfNegative,
    QuarterNegative,
    Invert,
    Squeeze,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOperation {
    Add,
    Multiply,
    Minimum,
    Maximum,
}

#[derive(Clone)]
pub struct Spline {
    pub coordinate: Box<DensityExpr>,
    pub points: Vec<SplinePoint>,
}

#[derive(Clone)]
pub struct SplinePoint {
    pub location: f32,
    pub value: SplineValue,
    pub derivative: f32,
}

#[derive(Clone)]
pub enum SplineValue {
    Constant(f32),
    Spline(Box<Spline>),
}

impl DensityExpr {
    pub fn sample(&self, context: DensityContext) -> Result<f64, DensityError> {
        match self {
            Self::Leaf(leaf) => Ok(leaf.sample(context)),
            Self::Constant(value) => Ok(*value),
            Self::YClampedGradient {
                from_y,
                to_y,
                from_value,
                to_value,
            } => Ok(clamped_map(
                context.y,
                *from_y,
                *to_y,
                *from_value,
                *to_value,
            )),
            Self::Clamp {
                input,
                minimum,
                maximum,
            } => {
                let value = input.sample(context)?;
                Ok(if value < *minimum {
                    *minimum
                } else {
                    java_min(value, *maximum)
                })
            }
            Self::Unary { operation, input } => Ok(operation.apply(input.sample(context)?)),
            Self::Binary {
                operation,
                first,
                second,
            } => {
                let first_value = first.sample(context)?;
                match operation {
                    BinaryOperation::Add => Ok(first_value + second.sample(context)?),
                    BinaryOperation::Multiply if first_value == 0.0 => Ok(0.0),
                    BinaryOperation::Multiply => Ok(first_value * second.sample(context)?),
                    BinaryOperation::Minimum if first_value < second.bounds().minimum => {
                        Ok(first_value)
                    }
                    BinaryOperation::Minimum => Ok(java_min(first_value, second.sample(context)?)),
                    BinaryOperation::Maximum if first_value > second.bounds().maximum => {
                        Ok(first_value)
                    }
                    BinaryOperation::Maximum => Ok(java_max(first_value, second.sample(context)?)),
                }
            }
            Self::RangeChoice {
                input,
                minimum_inclusive,
                maximum_exclusive,
                in_range,
                out_of_range,
            } => {
                let value = input.sample(context)?;
                if value >= *minimum_inclusive && value < *maximum_exclusive {
                    in_range.sample(context)
                } else {
                    out_of_range.sample(context)
                }
            }
            Self::IntervalSelect {
                input,
                thresholds,
                intervals,
            } => {
                validate_intervals(thresholds, intervals)?;
                let value = input.sample(context)?;
                let index = thresholds
                    .iter()
                    .position(|threshold| value < *threshold)
                    .unwrap_or(thresholds.len());
                intervals[index].sample(context)
            }
            Self::FindTopSurface {
                density,
                upper_bound,
                lower_bound,
                cell_height,
            } => {
                if *cell_height == 0 {
                    return Err(DensityError::ZeroCellHeight);
                }
                let cell_height =
                    i32::try_from(*cell_height).map_err(|_| DensityError::CellHeightTooLarge)?;
                let upper = upper_bound.sample(context)? as i32;
                let mut y = upper.div_euclid(cell_height) * cell_height;
                if y <= *lower_bound {
                    return Ok(f64::from(*lower_bound));
                }
                while y > *lower_bound {
                    if density.sample(DensityContext { y, ..context })? > 0.0 {
                        return Ok(f64::from(y));
                    }
                    y = y.saturating_sub(cell_height);
                }
                Ok(f64::from(*lower_bound))
            }
            Self::Spline(spline) => Ok(f64::from(spline.sample(context)?)),
        }
    }

    pub fn bounds(&self) -> DensityBounds {
        match self {
            Self::Leaf(leaf) => leaf.bounds(),
            Self::Constant(value) => DensityBounds {
                minimum: *value,
                maximum: *value,
            },
            Self::YClampedGradient {
                from_value,
                to_value,
                ..
            } => ordered_bounds(*from_value, *to_value),
            Self::Clamp {
                minimum, maximum, ..
            } => DensityBounds {
                minimum: *minimum,
                maximum: *maximum,
            },
            Self::Unary { operation, input } => operation.bounds(input.bounds()),
            Self::Binary {
                operation,
                first,
                second,
            } => operation.bounds(first.bounds(), second.bounds()),
            Self::RangeChoice {
                in_range,
                out_of_range,
                ..
            } => union(in_range.bounds(), out_of_range.bounds()),
            Self::IntervalSelect { intervals, .. } => intervals
                .iter()
                .map(Self::bounds)
                .reduce(union)
                .unwrap_or(DensityBounds {
                    minimum: f64::NEG_INFINITY,
                    maximum: f64::INFINITY,
                }),
            Self::FindTopSurface {
                upper_bound,
                lower_bound,
                ..
            } => DensityBounds {
                minimum: f64::from(*lower_bound),
                maximum: java_max(f64::from(*lower_bound), upper_bound.bounds().maximum),
            },
            Self::Spline(_) => DensityBounds {
                minimum: f64::NEG_INFINITY,
                maximum: f64::INFINITY,
            },
        }
    }
}

impl UnaryOperation {
    fn apply(self, value: f64) -> f64 {
        match self {
            Self::Abs => value.abs(),
            Self::Square => value * value,
            Self::Cube => value * value * value,
            Self::HalfNegative if value > 0.0 => value,
            Self::HalfNegative => value / 2.0,
            Self::QuarterNegative if value > 0.0 => value,
            Self::QuarterNegative => value / 4.0,
            Self::Invert => 1.0 / value,
            Self::Squeeze => {
                let clamped = value.clamp(-1.0, 1.0);
                clamped / 2.0 - clamped * clamped * clamped / 24.0
            }
        }
    }

    fn bounds(self, input: DensityBounds) -> DensityBounds {
        match self {
            Self::Abs | Self::Square => {
                let maximum = java_max(input.minimum.abs(), input.maximum.abs());
                let minimum = if input.minimum <= 0.0 && input.maximum >= 0.0 {
                    0.0
                } else {
                    java_min(input.minimum.abs(), input.maximum.abs())
                };
                if self == Self::Square {
                    DensityBounds {
                        minimum: minimum * minimum,
                        maximum: maximum * maximum,
                    }
                } else {
                    DensityBounds { minimum, maximum }
                }
            }
            Self::Cube => ordered_bounds(
                input.minimum * input.minimum * input.minimum,
                input.maximum * input.maximum * input.maximum,
            ),
            Self::HalfNegative => DensityBounds {
                minimum: if input.minimum > 0.0 {
                    input.minimum
                } else {
                    input.minimum / 2.0
                },
                maximum: if input.maximum > 0.0 {
                    input.maximum
                } else {
                    input.maximum / 2.0
                },
            },
            Self::QuarterNegative => DensityBounds {
                minimum: if input.minimum > 0.0 {
                    input.minimum
                } else {
                    input.minimum / 4.0
                },
                maximum: if input.maximum > 0.0 {
                    input.maximum
                } else {
                    input.maximum / 4.0
                },
            },
            Self::Invert => DensityBounds {
                minimum: f64::NEG_INFINITY,
                maximum: f64::INFINITY,
            },
            Self::Squeeze => DensityBounds {
                minimum: Self::Squeeze.apply(input.minimum),
                maximum: Self::Squeeze.apply(input.maximum),
            },
        }
    }
}

impl BinaryOperation {
    fn bounds(self, first: DensityBounds, second: DensityBounds) -> DensityBounds {
        match self {
            Self::Add => DensityBounds {
                minimum: first.minimum + second.minimum,
                maximum: first.maximum + second.maximum,
            },
            Self::Multiply => {
                let values = [
                    first.minimum * second.minimum,
                    first.minimum * second.maximum,
                    first.maximum * second.minimum,
                    first.maximum * second.maximum,
                ];
                DensityBounds {
                    minimum: values.iter().copied().fold(f64::INFINITY, java_min),
                    maximum: values.iter().copied().fold(f64::NEG_INFINITY, java_max),
                }
            }
            Self::Minimum => DensityBounds {
                minimum: java_min(first.minimum, second.minimum),
                maximum: java_min(first.maximum, second.maximum),
            },
            Self::Maximum => DensityBounds {
                minimum: java_max(first.minimum, second.minimum),
                maximum: java_max(first.maximum, second.maximum),
            },
        }
    }
}

impl Spline {
    fn sample(&self, context: DensityContext) -> Result<f32, DensityError> {
        if self.points.is_empty() {
            return Err(DensityError::EmptySpline);
        }
        let coordinate = self.coordinate.sample(context)? as f32;
        let first = &self.points[0];
        if coordinate < first.location {
            return Ok(
                first.value.sample(context)? + first.derivative * (coordinate - first.location)
            );
        }
        let last = self.points.last().expect("nonempty spline");
        if coordinate > last.location {
            return Ok(last.value.sample(context)? + last.derivative * (coordinate - last.location));
        }
        let upper = self
            .points
            .iter()
            .position(|point| coordinate < point.location)
            .unwrap_or(self.points.len() - 1);
        if upper == 0 {
            return first.value.sample(context);
        }
        let lower = upper - 1;
        let left = &self.points[lower];
        let right = &self.points[upper];
        let span = right.location - left.location;
        let t = (coordinate - left.location) / span;
        let left_value = left.value.sample(context)?;
        let right_value = right.value.sample(context)?;
        let difference = right_value - left_value;
        let left_curve = left.derivative * span - difference;
        let right_curve = -right.derivative * span + difference;
        Ok(lerp(t, left_value, right_value) + t * (1.0 - t) * lerp(t, left_curve, right_curve))
    }
}

impl SplineValue {
    fn sample(&self, context: DensityContext) -> Result<f32, DensityError> {
        match self {
            Self::Constant(value) => Ok(*value),
            Self::Spline(spline) => spline.sample(context),
        }
    }
}

fn validate_intervals(thresholds: &[f64], intervals: &[DensityExpr]) -> Result<(), DensityError> {
    if intervals.len() < 2 || thresholds.len() + 1 != intervals.len() {
        return Err(DensityError::InvalidIntervals);
    }
    if thresholds.windows(2).any(|pair| pair[0] > pair[1]) {
        return Err(DensityError::UnorderedThresholds);
    }
    Ok(())
}

fn clamped_map(y: i32, from_y: i32, to_y: i32, from_value: f64, to_value: f64) -> f64 {
    if y <= from_y {
        from_value
    } else if y >= to_y {
        to_value
    } else {
        let delta = f64::from(y - from_y) / f64::from(to_y - from_y);
        from_value + delta * (to_value - from_value)
    }
}

fn ordered_bounds(first: f64, second: f64) -> DensityBounds {
    DensityBounds {
        minimum: java_min(first, second),
        maximum: java_max(first, second),
    }
}

fn union(first: DensityBounds, second: DensityBounds) -> DensityBounds {
    DensityBounds {
        minimum: java_min(first.minimum, second.minimum),
        maximum: java_max(first.maximum, second.maximum),
    }
}

fn java_min(first: f64, second: f64) -> f64 {
    if first.is_nan() || second.is_nan() {
        f64::NAN
    } else if first == 0.0 && second == 0.0 {
        f64::from_bits(first.to_bits() | second.to_bits())
    } else if first < second {
        first
    } else {
        second
    }
}

fn java_max(first: f64, second: f64) -> f64 {
    if first.is_nan() || second.is_nan() {
        f64::NAN
    } else if first == 0.0 && second == 0.0 {
        f64::from_bits(first.to_bits() & second.to_bits())
    } else if first > second {
        first
    } else {
        second
    }
}

fn lerp(delta: f32, start: f32, end: f32) -> f32 {
    start + delta * (end - start)
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DensityError {
    #[error("interval-select requires N functions and N-1 thresholds")]
    InvalidIntervals,
    #[error("interval-select thresholds must be nondecreasing")]
    UnorderedThresholds,
    #[error("find-top-surface cell height must be positive")]
    ZeroCellHeight,
    #[error("find-top-surface cell height does not fit a Java integer")]
    CellHeightTooLarge,
    #[error("spline point list must be nonempty")]
    EmptySpline,
}

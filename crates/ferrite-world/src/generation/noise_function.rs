//! Five normal-noise coordinate functions and keyed-holder wiring.

use std::collections::HashMap;
use std::sync::Arc;

use thiserror::Error;

use crate::generation::density::{DensityBounds, DensityContext, DensityLeaf};
use crate::generation::noise::NormalNoise;

pub trait NoiseSampler: Send + Sync {
    fn sample(&self, x: f64, y: f64, z: f64) -> f64;

    fn maximum(&self) -> f64;
}

impl NoiseSampler for NormalNoise {
    fn sample(&self, x: f64, y: f64, z: f64) -> f64 {
        NormalNoise::sample(self, x, y, z)
    }

    fn maximum(&self) -> f64 {
        NormalNoise::maximum(self)
    }
}

#[derive(Clone)]
pub struct NoiseHolder {
    sampler: Option<Arc<dyn NoiseSampler>>,
}

impl NoiseHolder {
    pub fn unwired() -> Self {
        Self { sampler: None }
    }

    pub fn wired(sampler: Arc<dyn NoiseSampler>) -> Self {
        Self {
            sampler: Some(sampler),
        }
    }

    pub fn sample(&self, x: f64, y: f64, z: f64) -> f64 {
        self.sampler
            .as_ref()
            .map_or(0.0, |noise| noise.sample(x, y, z))
    }

    pub fn maximum(&self) -> f64 {
        self.sampler.as_ref().map_or(2.0, |noise| noise.maximum())
    }

    pub fn is_wired(&self) -> bool {
        self.sampler.is_some()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShiftKind {
    Shift,
    ShiftA,
    ShiftB,
}

#[derive(Clone)]
pub enum NoiseFunction {
    Noise {
        holder: Arc<NoiseHolder>,
        xz_scale: f64,
        y_scale: f64,
    },
    Shift {
        holder: Arc<NoiseHolder>,
        kind: ShiftKind,
    },
    ShiftedNoise {
        holder: Arc<NoiseHolder>,
        shift_x: Arc<dyn DensityLeaf>,
        shift_y: Arc<dyn DensityLeaf>,
        shift_z: Arc<dyn DensityLeaf>,
        xz_scale: f64,
        y_scale: f64,
    },
}

impl DensityLeaf for NoiseFunction {
    fn sample(&self, context: DensityContext) -> f64 {
        match self {
            Self::Noise {
                holder,
                xz_scale,
                y_scale,
            } => holder.sample(
                f64::from(context.x) * *xz_scale,
                f64::from(context.y) * *y_scale,
                f64::from(context.z) * *xz_scale,
            ),
            Self::Shift { holder, kind } => {
                let [x, y, z] = match kind {
                    ShiftKind::Shift => [context.x, context.y, context.z],
                    ShiftKind::ShiftA => [context.x, 0, context.z],
                    ShiftKind::ShiftB => [context.z, context.x, 0],
                };
                holder.sample(
                    f64::from(x) * 0.25,
                    f64::from(y) * 0.25,
                    f64::from(z) * 0.25,
                ) * 4.0
            }
            Self::ShiftedNoise {
                holder,
                shift_x,
                shift_y,
                shift_z,
                xz_scale,
                y_scale,
            } => {
                let x_shift = shift_x.sample(context);
                let y_shift = shift_y.sample(context);
                let z_shift = shift_z.sample(context);
                holder.sample(
                    f64::from(context.x) * *xz_scale + x_shift,
                    f64::from(context.y) * *y_scale + y_shift,
                    f64::from(context.z) * *xz_scale + z_shift,
                )
            }
        }
    }

    fn bounds(&self) -> DensityBounds {
        let magnitude = match self {
            Self::Noise { holder, .. } | Self::ShiftedNoise { holder, .. } => holder.maximum(),
            Self::Shift { holder, .. } => holder.maximum() * 4.0,
        };
        DensityBounds {
            minimum: -magnitude,
            maximum: magnitude,
        }
    }
}

#[derive(Default)]
pub struct NoiseWiringCache {
    wired: HashMap<String, Arc<NoiseHolder>>,
}

impl NoiseWiringCache {
    pub fn wire(
        &mut self,
        key: Option<&str>,
        mut create: impl FnMut(&str) -> Arc<dyn NoiseSampler>,
    ) -> Result<Arc<NoiseHolder>, NoiseWiringError> {
        let key = key.ok_or(NoiseWiringError::DirectHolder)?;
        if let Some(holder) = self.wired.get(key) {
            return Ok(holder.clone());
        }
        let holder = Arc::new(NoiseHolder::wired(create(key)));
        self.wired.insert(key.to_owned(), holder.clone());
        Ok(holder)
    }

    pub fn len(&self) -> usize {
        self.wired.len()
    }

    pub fn is_empty(&self) -> bool {
        self.wired.is_empty()
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum NoiseWiringError {
    #[error("direct or inline noise holders have no resource key for random-state wiring")]
    DirectHolder,
}

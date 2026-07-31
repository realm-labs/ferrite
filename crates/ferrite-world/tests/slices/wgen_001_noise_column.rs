use ferrite_foundation::coordinate::BlockPos;
use ferrite_world::generation::noise_column::{
    NoiseColumnSampler, base_column, base_height, interpolated_noise_value,
};
use ferrite_world::generation::noise_fill::NoiseSettings;
use ferrite_world::id::BlockStateId;

const AIR: BlockStateId = BlockStateId::new(0);
const STONE: BlockStateId = BlockStateId::new(1);

#[derive(Default)]
struct Sampler {
    events: Vec<String>,
}

impl NoiseColumnSampler for Sampler {
    type Error = &'static str;

    fn accessor_minimum_y(&self) -> i32 {
        -16
    }

    fn accessor_maximum_y(&self) -> i32 {
        15
    }

    fn start_interpolation(&mut self, x: i32, z: i32) -> Result<(), Self::Error> {
        self.events.push(format!("start:{x}:{z}"));
        Ok(())
    }

    fn advance_cell_x(&mut self) -> Result<(), Self::Error> {
        self.events.push("advance".into());
        Ok(())
    }

    fn select_cell(&mut self, y: i32) -> Result<(), Self::Error> {
        self.events.push(format!("cell:{y}"));
        Ok(())
    }

    fn update_for_y(&mut self, fraction: f64) -> Result<(), Self::Error> {
        self.events.push(format!("y:{fraction}"));
        Ok(())
    }

    fn update_for_x(&mut self, fraction: f64) -> Result<(), Self::Error> {
        self.events.push(format!("x:{fraction}"));
        Ok(())
    }

    fn update_for_z(&mut self, fraction: f64) -> Result<(), Self::Error> {
        self.events.push(format!("z:{fraction}"));
        Ok(())
    }

    fn material(&mut self, position: BlockPos) -> Result<BlockStateId, Self::Error> {
        self.events.push(format!("sample:{}", position.y));
        Ok(if position.y <= 3 { STONE } else { AIR })
    }

    fn stop_interpolation(&mut self) -> Result<(), Self::Error> {
        self.events.push("stop".into());
        Ok(())
    }
}

fn settings() -> NoiseSettings {
    NoiseSettings {
        minimum_y: -16,
        height: 32,
        horizontal_size: 1,
        vertical_size: 1,
    }
}

#[test]
fn base_height_scans_top_down_and_stops_one_above_first_match() {
    let mut sampler = Sampler::default();
    let height = base_height(&mut sampler, settings(), -1, 5, |state| state == STONE).unwrap();

    assert_eq!(height, 4);
    assert_eq!(sampler.events[0], "start:-1:1");
    assert_eq!(sampler.events[1], "advance");
    assert_eq!(sampler.events.last().unwrap(), "stop");
    assert!(!sampler.events.iter().any(|event| event == "sample:2"));
    assert!(sampler.events.iter().any(|event| event == "x:0.75"));
    assert!(sampler.events.iter().any(|event| event == "z:0.25"));
}

#[test]
fn base_column_is_bottom_indexed_and_samples_the_complete_clamped_height() {
    let mut sampler = Sampler::default();
    let column = base_column(&mut sampler, settings(), 0, 0)
        .unwrap()
        .unwrap();

    assert_eq!(column.minimum_y, -16);
    assert_eq!(column.states.len(), 32);
    assert_eq!(column.states[0], STONE);
    assert_eq!(column.states[19], STONE);
    assert_eq!(column.states[20], AIR);
    assert_eq!(
        sampler
            .events
            .iter()
            .filter(|event| event.starts_with("sample:"))
            .count(),
        32
    );
}

#[test]
fn empty_clamp_returns_minimum_or_null_without_starting_lifecycle() {
    let mut sampler = Sampler::default();
    let outside = NoiseSettings {
        minimum_y: 32,
        height: 16,
        horizontal_size: 1,
        vertical_size: 1,
    };
    assert_eq!(
        base_height(&mut sampler, outside, 0, 0, |_| true).unwrap(),
        -16
    );
    assert_eq!(base_column(&mut sampler, outside, 0, 0).unwrap(), None);
    assert!(sampler.events.is_empty());
}

#[test]
fn interpolated_value_uses_unclamped_setting_range_and_nan_outside() {
    assert!(interpolated_noise_value(settings(), BlockPos::new(0, -17, 0), |_| 1.0).is_nan());
    assert!(interpolated_noise_value(settings(), BlockPos::new(0, 16, 0), |_| 1.0).is_nan());
    assert_eq!(
        interpolated_noise_value(settings(), BlockPos::new(2, 15, 3), |position| {
            f64::from(position.x + position.y + position.z)
        }),
        20.0
    );
}

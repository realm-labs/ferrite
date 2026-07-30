//! Checked caller-owned randomness for data-driven item runtimes.

pub trait GameplayRandom {
    fn next_int(&mut self, bound: u32) -> u32;

    fn next_float(&mut self) -> f32;

    fn next_bool(&mut self) -> bool;
}

pub fn checked_int(
    random: &mut dyn GameplayRandom,
    bound: u32,
) -> Result<u32, GameplayRandomError> {
    if bound == 0 {
        return Err(GameplayRandomError::ZeroBound);
    }
    let draw = random.next_int(bound);
    if draw >= bound {
        return Err(GameplayRandomError::DrawOutOfRange { draw, bound });
    }
    Ok(draw)
}

pub fn checked_float(random: &mut dyn GameplayRandom) -> Result<f32, GameplayRandomError> {
    let draw = random.next_float();
    if !draw.is_finite() || !(0.0..1.0).contains(&draw) {
        return Err(GameplayRandomError::FloatOutOfRange);
    }
    Ok(draw)
}

pub fn shuffle<T>(
    values: &mut [T],
    random: &mut dyn GameplayRandom,
) -> Result<(), GameplayRandomError> {
    for upper in (1..values.len()).rev() {
        let selected = checked_int(random, (upper + 1) as u32)? as usize;
        values.swap(upper, selected);
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameplayRandomError {
    ZeroBound,
    DrawOutOfRange { draw: u32, bound: u32 },
    FloatOutOfRange,
}

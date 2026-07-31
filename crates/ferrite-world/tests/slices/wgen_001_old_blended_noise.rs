use ferrite_world::generation::feature::random::LegacyRandom;
use ferrite_world::generation::noise::ImprovedNoise;
use ferrite_world::generation::old_blended_noise::{
    OldBlendedNoise, OldBlendedNoiseConfig, OldBlendedNoiseError,
};

#[test]
fn construction_allocates_sixteen_sixteen_eight_improved_noises_in_order() {
    let mut actual_random = LegacyRandom::new(17);
    let noise = OldBlendedNoise::new(&mut actual_random, overworld()).unwrap();

    let mut expected_random = LegacyRandom::new(17);
    for _ in 0..40 {
        let _ = ImprovedNoise::new(&mut expected_random);
    }

    assert_eq!(actual_random.next_i32(), expected_random.next_i32());
    assert_eq!(noise.stacks().0.levels().len(), 16);
    assert_eq!(noise.stacks().1.levels().len(), 16);
    assert_eq!(noise.stacks().2.levels().len(), 8);
}

#[test]
fn identical_seed_and_parameters_produce_identical_density_and_symmetric_bounds() {
    let mut first_random = LegacyRandom::new(99);
    let mut second_random = LegacyRandom::new(99);
    let first = OldBlendedNoise::new(&mut first_random, overworld()).unwrap();
    let second = OldBlendedNoise::new(&mut second_random, overworld()).unwrap();

    assert_eq!(first.sample(12, -23, 45), second.sample(12, -23, 45));
    let bounds = first.bounds();
    assert_eq!(bounds.0, -bounds.1);
    assert!(bounds.1 > 0.0);
}

#[test]
fn codec_scale_endpoints_are_inclusive_and_outside_values_reject() {
    let mut random = LegacyRandom::new(0);
    let valid = OldBlendedNoiseConfig {
        xz_scale: 0.001,
        y_scale: 1_000.0,
        xz_factor: 0.001,
        y_factor: 1_000.0,
        smear_scale_multiplier: 8.0,
    };
    assert!(OldBlendedNoise::new(&mut random, valid).is_ok());

    let mut random = LegacyRandom::new(0);
    assert_eq!(
        OldBlendedNoise::new(
            &mut random,
            OldBlendedNoiseConfig {
                xz_scale: 0.0,
                ..overworld()
            },
        )
        .unwrap_err(),
        OldBlendedNoiseError::InvalidConfiguration
    );
}

fn overworld() -> OldBlendedNoiseConfig {
    OldBlendedNoiseConfig {
        xz_scale: 0.25,
        y_scale: 0.125,
        xz_factor: 80.0,
        y_factor: 160.0,
        smear_scale_multiplier: 8.0,
    }
}

//! Closed inheritance-family random decisions and special producer facts.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Parent {
    Actor,
    Partner,
}

#[must_use]
pub const fn random_parent(boolean_draw: bool) -> Parent {
    if boolean_draw {
        Parent::Actor
    } else {
        Parent::Partner
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MooshroomVariant {
    Actor,
    Partner,
    MutatedOther,
}

#[must_use]
pub const fn mooshroom_variant(
    parents_equal: bool,
    parent_draw: bool,
    mutation_draw_below_1024: u16,
) -> MooshroomVariant {
    if parents_equal && mutation_draw_below_1024 == 0 {
        MooshroomVariant::MutatedOther
    } else if parent_draw {
        MooshroomVariant::Actor
    } else {
        MooshroomVariant::Partner
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RabbitVariant {
    Biome,
    Actor,
    Partner,
}

#[must_use]
pub const fn rabbit_variant(
    inherit_draw_below_twenty: u8,
    partner_is_rabbit: bool,
    parent_draw: bool,
) -> RabbitVariant {
    if inherit_draw_below_twenty == 0 {
        RabbitVariant::Biome
    } else if partner_is_rabbit && parent_draw {
        RabbitVariant::Partner
    } else {
        RabbitVariant::Actor
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AxolotlVariant {
    RareRegistry,
    Actor,
    Partner,
}

#[must_use]
pub const fn axolotl_variant(rare_draw: u16, parent_draw: bool) -> AxolotlVariant {
    if rare_draw.is_multiple_of(1_200) {
        AxolotlVariant::RareRegistry
    } else if parent_draw {
        AxolotlVariant::Actor
    } else {
        AxolotlVariant::Partner
    }
}

#[must_use]
pub const fn goat_screaming(selected_parent_screams: bool, chance_draw: f32) -> bool {
    selected_parent_screams || chance_draw < 0.02
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HorseVariant {
    pub coat_source: u8,
    pub markings_source: u8,
}

#[must_use]
pub const fn horse_variant(coat_draw_below_nine: u8, markings_draw_below_five: u8) -> HorseVariant {
    HorseVariant {
        coat_source: match coat_draw_below_nine % 9 {
            0..=3 => 0,
            4..=7 => 1,
            _ => 2,
        },
        markings_source: match markings_draw_below_five % 5 {
            0..=1 => 0,
            2..=3 => 1,
            _ => 2,
        },
    }
}

#[must_use]
pub const fn llama_strength(
    actor_strength: u8,
    partner_strength: u8,
    bounded_draw: u8,
    increase_draw: f32,
) -> u8 {
    let maximum = if actor_strength > partner_strength {
        actor_strength
    } else {
        partner_strength
    };
    let bound = if maximum == 0 { 1 } else { maximum };
    let base = 1 + bounded_draw % bound;
    if increase_draw < 0.03 {
        let increased = base.saturating_add(1);
        if increased > 5 { 5 } else { increased }
    } else {
        base
    }
}

#[must_use]
pub fn reflected_horse_attribute(
    actor: f64,
    partner: f64,
    minimum: f64,
    maximum: f64,
    first_uniform: f64,
    second_uniform: f64,
    third_uniform: f64,
) -> f64 {
    let margin = (maximum - minimum) * 0.15;
    let spread = (actor - partner).abs() + margin * 2.0;
    let center = (actor + partner) * 0.5;
    let triangular = (first_uniform + second_uniform + third_uniform - 1.5) * spread;
    let value = center + triangular;
    if value > maximum {
        maximum - (value - maximum)
    } else if value < minimum {
        minimum + (minimum - value)
    } else {
        value
    }
}

#[must_use]
pub const fn equine_parentable(
    tame: bool,
    adult: bool,
    full_health: bool,
    in_love: bool,
    vehicle: bool,
    passenger: bool,
) -> bool {
    tame && adult && full_health && in_love && !vehicle && !passenger
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FoodFamily {
    Tagged,
    NautilusTaming,
    NautilusFood,
    Never,
    RejectLove,
    CustomGrowthOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProducerFacts {
    pub persistent_child: bool,
    pub creates_immediate_child: bool,
    pub uses_brain_behavior: bool,
}

#[must_use]
pub const fn producer_facts(family: ProducerFamily) -> ProducerFacts {
    match family {
        ProducerFamily::Axolotl | ProducerFamily::Hoglin => ProducerFacts {
            persistent_child: true,
            creates_immediate_child: true,
            uses_brain_behavior: matches!(family, ProducerFamily::Hoglin),
        },
        ProducerFamily::Turtle
        | ProducerFamily::Frog
        | ProducerFamily::Sniffer
        | ProducerFamily::Allay => ProducerFacts {
            persistent_child: matches!(family, ProducerFamily::Allay),
            creates_immediate_child: matches!(family, ProducerFamily::Allay),
            uses_brain_behavior: matches!(family, ProducerFamily::Frog | ProducerFamily::Sniffer),
        },
        ProducerFamily::Villager => ProducerFacts {
            persistent_child: false,
            creates_immediate_child: true,
            uses_brain_behavior: true,
        },
        ProducerFamily::Ordinary => ProducerFacts {
            persistent_child: false,
            creates_immediate_child: true,
            uses_brain_behavior: false,
        },
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProducerFamily {
    Ordinary,
    Axolotl,
    Hoglin,
    Turtle,
    Frog,
    Sniffer,
    Villager,
    Allay,
}

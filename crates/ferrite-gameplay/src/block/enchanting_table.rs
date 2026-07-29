//! Enchanting-table menu/name boundaries, particles, and client book animation.

pub const BLOCK_ID: u16 = 385;
pub const ITEM_ID: u16 = 461;
pub const BLOCK_ENTITY_PROTOCOL_ID: u32 = 13;
pub const BLOCK_STATE_ID: u32 = 9_451;
pub const MENU_PROTOCOL_ID: u16 = 13;
pub const ENCHANT_PARTICLE_ID: u16 = 26;
pub const MAX_STACK: u8 = 64;
pub const LIGHT_LEVEL: u8 = 7;
pub const SHAPE_HEIGHT: u8 = 12;
pub const HARDNESS: f32 = 5.0;
pub const RESISTANCE: f32 = 1_200.0;
pub const DEFAULT_TITLE: &str = "container.enchant";
pub const BOOKSHELF_PROBE_COUNT: usize = 32;
pub const REQUIRES_CORRECT_TOOL: bool = true;
pub const USES_SHAPE_FOR_LIGHT_OCCLUSION: bool = true;

pub const fn enchanting_use_without_item_admitted(
    main_hand_attempt: bool,
    secondary_use: bool,
    main_hand_nonempty: bool,
    off_hand_nonempty: bool,
) -> bool {
    main_hand_attempt && !(secondary_use && (main_hand_nonempty || off_hand_nonempty))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableSide {
    Server,
    Client,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnchantingTableData {
    pub custom_name: Option<String>,
}

impl EnchantingTableData {
    pub fn load(stored_name: StoredCustomName) -> Self {
        Self {
            custom_name: match stored_name {
                StoredCustomName::Valid(name) => Some(name),
                StoredCustomName::Missing | StoredCustomName::Malformed => None,
            },
        }
    }

    pub fn display_name(&self) -> &str {
        self.custom_name.as_deref().unwrap_or(DEFAULT_TITLE)
    }

    pub fn apply_custom_name_component(&mut self, name: Option<String>) {
        self.custom_name = name;
    }

    pub fn collected_custom_name_component(&self) -> Option<&str> {
        self.custom_name.as_deref()
    }

    pub fn saved_custom_name(&self) -> Option<&str> {
        self.custom_name.as_deref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoredCustomName {
    Missing,
    Valid(String),
    Malformed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableUse {
    pub success: bool,
    pub opens_menu: bool,
    pub title: Option<String>,
    pub menu_protocol_id: Option<u16>,
    pub creates_level_access: bool,
}

pub fn enchanting_table_use(
    side: TableSide,
    matching_block_entity: bool,
    data: Option<&EnchantingTableData>,
) -> TableUse {
    let provider = matching_block_entity.then_some(data).flatten();
    let opens_menu = side == TableSide::Server && provider.is_some();
    TableUse {
        success: true,
        opens_menu,
        title: opens_menu.then(|| {
            provider
                .expect("provider checked")
                .display_name()
                .to_owned()
        }),
        menu_protocol_id: opens_menu.then_some(MENU_PROTOCOL_ID),
        creates_level_access: opens_menu,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnchantingTableDrop {
    pub custom_name: Option<String>,
}

pub fn enchanting_table_loot(
    survives_explosion: bool,
    data: Option<&EnchantingTableData>,
) -> Option<EnchantingTableDrop> {
    survives_explosion.then(|| EnchantingTableDrop {
        custom_name: data.and_then(|table| table.custom_name.clone()),
    })
}

pub const fn enchanting_table_pick() -> EnchantingTableDrop {
    EnchantingTableDrop { custom_name: None }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BookshelfProbe {
    pub provider: bool,
    pub transmitter: bool,
}

impl BookshelfProbe {
    pub const INVALID: Self = Self {
        provider: false,
        transmitter: false,
    };

    pub const fn valid(self) -> bool {
        self.provider && self.transmitter
    }
}

pub trait TableRandom {
    fn next_int(&mut self, bound: u32) -> u32;
    fn next_float(&mut self) -> f32;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EnchantParticle {
    pub offset: [i32; 3],
    pub position: [f64; 3],
    pub velocity: [f64; 3],
}

pub fn bookshelf_offsets() -> [[i32; 3]; BOOKSHELF_PROBE_COUNT] {
    let mut offsets = [[0; 3]; BOOKSHELF_PROBE_COUNT];
    let mut index = 0;
    for z in -2_i32..=2 {
        for y in 0..=1 {
            for x in -2_i32..=2 {
                if x.abs() == 2 || z.abs() == 2 {
                    offsets[index] = [x, y, z];
                    index += 1;
                }
            }
        }
    }
    offsets
}

pub fn enchanting_particles<R: TableRandom>(
    probes: &[BookshelfProbe; BOOKSHELF_PROBE_COUNT],
    random: &mut R,
) -> Vec<EnchantParticle> {
    enchanting_particle_scan(random, |index, _| probes[index])
}

pub fn enchanting_particle_scan<R, F>(random: &mut R, mut probe_at: F) -> Vec<EnchantParticle>
where
    R: TableRandom,
    F: FnMut(usize, [i32; 3]) -> BookshelfProbe,
{
    let offsets = bookshelf_offsets();
    let mut particles = Vec::new();
    for (index, offset) in offsets.into_iter().enumerate() {
        if random.next_int(16) != 0 || !probe_at(index, offset).valid() {
            continue;
        }
        let x_random = random.next_float();
        let y_random = random.next_float();
        let z_random = random.next_float();
        particles.push(EnchantParticle {
            offset,
            position: [0.5, 2.0, 0.5],
            velocity: [
                f64::from(offset[0] as f32 + x_random - 0.5),
                f64::from(offset[1] as f32 - y_random - 1.0),
                f64::from(offset[2] as f32 + z_random - 0.5),
            ],
        });
    }
    particles
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClientPlayer {
    pub id: u64,
    pub position: [f64; 3],
    pub spectator: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BookAnimation {
    pub time: i32,
    pub flip: f32,
    pub previous_flip: f32,
    pub target_flip: f32,
    pub flip_acceleration: f32,
    pub open: f32,
    pub previous_open: f32,
    pub rotation: f32,
    pub previous_rotation: f32,
    pub target_rotation: f32,
}

impl Default for BookAnimation {
    fn default() -> Self {
        Self {
            time: 0,
            flip: 0.0,
            previous_flip: 0.0,
            target_flip: 0.0,
            flip_acceleration: 0.0,
            open: 0.0,
            previous_open: 0.0,
            rotation: 0.0,
            previous_rotation: 0.0,
            target_rotation: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BookTickOutcome {
    pub nearest_player: Option<u64>,
    pub page_selected: bool,
    pub chance_draw_consumed: bool,
}

impl BookAnimation {
    pub fn tick<R: TableRandom>(
        &mut self,
        block_position: [i32; 3],
        players: &[ClientPlayer],
        random: &mut R,
    ) -> BookTickOutcome {
        self.previous_open = self.open;
        self.previous_rotation = self.rotation;
        let center = [
            f64::from(block_position[0]) + 0.5,
            f64::from(block_position[1]) + 0.5,
            f64::from(block_position[2]) + 0.5,
        ];
        let nearest = nearest_player(center, players);
        let mut page_selected = false;
        let mut chance_draw_consumed = false;
        if let Some(player) = nearest {
            let dx = player.position[0] - center[0];
            let dz = player.position[2] - center[2];
            self.target_rotation = dz.atan2(dx) as f32;
            self.open += 0.1;
            let select_page = if self.open < 0.5 {
                true
            } else {
                chance_draw_consumed = true;
                random.next_int(40) == 0
            };
            if select_page {
                let old = self.target_flip;
                loop {
                    self.target_flip += random.next_int(4) as f32 - random.next_int(4) as f32;
                    if old != self.target_flip {
                        break;
                    }
                }
                page_selected = true;
            }
        } else {
            self.target_rotation += 0.02;
            self.open -= 0.1;
        }

        self.rotation = wrap_radians(self.rotation);
        self.target_rotation = wrap_radians(self.target_rotation);
        let rotation_delta = wrap_radians(self.target_rotation - self.rotation);
        self.rotation += rotation_delta * 0.4;
        self.open = self.open.clamp(0.0, 1.0);
        self.time = self.time.wrapping_add(1);
        self.previous_flip = self.flip;
        let flip_delta = ((self.target_flip - self.flip) * 0.4).clamp(-0.2, 0.2);
        self.flip_acceleration += (flip_delta - self.flip_acceleration) * 0.9;
        self.flip += self.flip_acceleration;

        BookTickOutcome {
            nearest_player: nearest.map(|player| player.id),
            page_selected,
            chance_draw_consumed,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EnchantingTableRender {
    pub flip: f32,
    pub open: f32,
    pub time: f32,
    pub yaw: f32,
    pub translation: [f32; 3],
    pub z_rotation_degrees: f32,
    pub left_page: f32,
    pub right_page: f32,
}

pub fn enchanting_table_render(
    animation: BookAnimation,
    partial_ticks: f32,
) -> EnchantingTableRender {
    let flip = lerp(partial_ticks, animation.previous_flip, animation.flip);
    let open = lerp(partial_ticks, animation.previous_open, animation.open);
    let time = animation.time as f32 + partial_ticks;
    let yaw_delta = wrap_radians(animation.rotation - animation.previous_rotation);
    let yaw = animation.previous_rotation + yaw_delta * partial_ticks;
    EnchantingTableRender {
        flip,
        open,
        time,
        yaw,
        translation: [0.5, 0.85 + (time * 0.1).sin() * 0.01, 0.5],
        z_rotation_degrees: 80.0,
        left_page: (fraction(flip + 0.25) * 1.6 - 0.3).clamp(0.0, 1.0),
        right_page: (fraction(flip + 0.75) * 1.6 - 0.3).clamp(0.0, 1.0),
    }
}

fn nearest_player(center: [f64; 3], players: &[ClientPlayer]) -> Option<ClientPlayer> {
    let mut nearest = None;
    let mut nearest_distance = 9.0;
    for player in players.iter().filter(|player| !player.spectator) {
        let dx = player.position[0] - center[0];
        let dy = player.position[1] - center[1];
        let dz = player.position[2] - center[2];
        let distance = dx * dx + dy * dy + dz * dz;
        if distance < nearest_distance {
            nearest = Some(*player);
            nearest_distance = distance;
        }
    }
    nearest
}

fn wrap_radians(mut value: f32) -> f32 {
    while value >= std::f32::consts::PI {
        value -= std::f32::consts::TAU;
    }
    while value < -std::f32::consts::PI {
        value += std::f32::consts::TAU;
    }
    value
}

const fn lerp(delta: f32, start: f32, end: f32) -> f32 {
    start + delta * (end - start)
}

fn fraction(value: f32) -> f32 {
    value - value.floor()
}

use ferrite_foundation::coordinate::{BlockPos, ChunkPos};
use ferrite_simulation::random_tick::activity::{
    BoundedRandom, DEFAULT_RANDOM_TICK_SPEED, GAMEPLAY_CHUNK_STAGES, GameplayChunkStage,
    HolderAccess, MAXIMUM_RANDOM_TICK_SPEED, MINIMUM_RANDOM_TICK_SPEED, RandomTickChunk,
    RandomTickState, SectionRandomTickCounts, chunk_cache_tick_plan, planned_attempts,
    random_tick_chunk_order, tick_chunk,
};
use ferrite_simulation::random_tick::position::RandomPositionStream;
use ferrite_simulation::random_tick::ticket::{
    FLAG_CAN_EXPIRE_IF_UNLOADED, FLAG_KEEP_DIMENSION_ACTIVE, FLAG_LOADING, FLAG_PERSIST,
    FLAG_SIMULATION, Ticket, TicketKind, TicketStorage, UpdatingHolderState,
};
use ferrite_simulation::random_tick::tracker::{
    ABSENT_SIMULATION_LEVEL, BLOCK_TICKING_LEVEL, ENTITY_TICKING_LEVEL, SimulationChunkTracker,
    is_block_ticking, is_entity_ticking, pack_chunk, player_simulation_source_level,
};
use std::cell::RefCell;
use std::collections::VecDeque;

#[test]
fn position_stream_matches_the_locked_signed_java_vector() {
    let mut stream = RandomPositionStream::new(0x1234_5678);
    let base = BlockPos::new(100, -64, -200);
    let expected = [
        (1_930_163_911, BlockPos::new(101, -62, -187)),
        (-1_785_538_636, BlockPos::new(113, -60, -195)),
        (-47_744_389, BlockPos::new(114, -55, -186)),
        (870_671_056, BlockPos::new(104, -55, -192)),
        (-669_049_905, BlockPos::new(103, -57, -194)),
    ];
    for (value, position) in expected {
        assert_eq!(stream.next(base, 15), position);
        assert_eq!(stream.value(), value);
    }
}

#[test]
fn activity_levels_keep_random_ticks_stricter_than_block_ticking() {
    assert!(is_entity_ticking(30));
    assert!(is_entity_ticking(ENTITY_TICKING_LEVEL));
    assert!(!is_entity_ticking(BLOCK_TICKING_LEVEL));
    assert!(is_block_ticking(BLOCK_TICKING_LEVEL));
    assert!(!is_block_ticking(ABSENT_SIMULATION_LEVEL));
    assert_eq!(player_simulation_source_level(0), 31);
    assert_eq!(player_simulation_source_level(10), 21);
    assert_eq!(player_simulation_source_level(31), 0);
    assert_eq!(player_simulation_source_level(32), 0);
}

#[test]
fn tracker_reproduces_fastutil_zero_slot_resize_and_scan_order() {
    let mut tracker = SimulationChunkTracker::new();
    for key in 0_i64..30 {
        tracker.set_level_by_key(key, key as u8);
    }
    assert_eq!(
        tracker
            .compatibility_key_order()
            .into_iter()
            .map(|(key, _)| key)
            .collect::<Vec<_>>(),
        [
            0, 25, 15, 28, 10, 12, 3, 18, 17, 27, 13, 2, 21, 20, 22, 29, 5, 14, 16, 9, 6, 24, 19,
            1, 11, 7, 8, 26, 4, 23,
        ]
    );

    for key in [3, 7, 12, 18, 25] {
        tracker.set_level_by_key(key, ABSENT_SIMULATION_LEVEL);
    }
    tracker.set_level_by_key(42, 5);
    tracker.set_level_by_key(-1, 6);
    tracker.set_level_by_key(0x0000_0001_0000_0000, 7);
    assert_eq!(
        tracker.compatibility_key_order(),
        [
            (0, 0),
            (42, 5),
            (15, 15),
            (28, 28),
            (10, 10),
            (17, 17),
            (27, 27),
            (13, 13),
            (2, 2),
            (0x0000_0001_0000_0000, 7),
            (21, 21),
            (20, 20),
            (22, 22),
            (29, 29),
            (5, 5),
            (14, 14),
            (16, 16),
            (9, 9),
            (6, 6),
            (24, 24),
            (19, 19),
            (1, 1),
            (-1, 6),
            (11, 11),
            (8, 8),
            (26, 26),
            (4, 4),
            (23, 23),
        ]
    );
}

#[test]
fn equal_activity_sets_retain_distinct_resize_histories() {
    let mut direct = SimulationChunkTracker::new();
    let mut resized = SimulationChunkTracker::new();
    for key in 0_i64..24 {
        direct.set_level_by_key(key, 31);
        resized.set_level_by_key(key, 31);
    }
    for key in 24_i64..30 {
        resized.set_level_by_key(key, 31);
    }
    for key in 24_i64..30 {
        resized.set_level_by_key(key, ABSENT_SIMULATION_LEVEL);
    }
    let direct_order = direct.compatibility_key_order();
    let resized_order = resized.compatibility_key_order();
    assert_ne!(direct_order, resized_order);
    let mut direct_keys: Vec<_> = direct_order.into_iter().map(|(key, _)| key).collect();
    let mut resized_keys: Vec<_> = resized_order.into_iter().map(|(key, _)| key).collect();
    direct_keys.sort_unstable();
    resized_keys.sort_unstable();
    assert_eq!(direct_keys, resized_keys);
}

#[test]
fn chunk_order_filters_level_holder_and_ticking_chunk_without_resorting() {
    let chunks = [
        ChunkPos::new(0, 0),
        ChunkPos::new(1, 0),
        ChunkPos::new(2, 0),
        ChunkPos::new(3, 0),
        ChunkPos::new(4, 0),
    ];
    let mut tracker = SimulationChunkTracker::new();
    for (chunk, level) in chunks.into_iter().zip([30, 31, 32, 29, 28]) {
        tracker.set_level(chunk, level);
    }
    let compatibility = tracker.compatibility_order();
    let expected: Vec<_> = compatibility
        .iter()
        .filter_map(|(chunk, level)| {
            (*level <= 31 && *chunk != chunks[3] && *chunk != chunks[4]).then_some(*chunk)
        })
        .collect();
    let order = random_tick_chunk_order(&tracker, |chunk| {
        if chunk == chunks[3] {
            HolderAccess::MissingVisibleHolder
        } else if chunk == chunks[4] {
            HolderAccess::MissingTickingChunk
        } else {
            HolderAccess::TickingChunk
        }
    });
    assert_eq!(order, expected);
}

#[test]
fn all_nine_ticket_types_have_locked_timeouts_and_flags() {
    let expected = [
        (TicketKind::PlayerSpawn, 20, FLAG_LOADING),
        (TicketKind::SpawnSearch, 1, FLAG_LOADING),
        (TicketKind::Dragon, 0, FLAG_LOADING | FLAG_SIMULATION),
        (TicketKind::PlayerLoading, 0, FLAG_LOADING),
        (
            TicketKind::PlayerSimulation,
            0,
            FLAG_SIMULATION | FLAG_KEEP_DIMENSION_ACTIVE,
        ),
        (
            TicketKind::Forced,
            0,
            FLAG_PERSIST | FLAG_LOADING | FLAG_SIMULATION | FLAG_KEEP_DIMENSION_ACTIVE,
        ),
        (
            TicketKind::Portal,
            300,
            FLAG_PERSIST | FLAG_LOADING | FLAG_SIMULATION | FLAG_KEEP_DIMENSION_ACTIVE,
        ),
        (
            TicketKind::EnderPearl,
            40,
            FLAG_LOADING | FLAG_SIMULATION | FLAG_KEEP_DIMENSION_ACTIVE,
        ),
        (
            TicketKind::Unknown,
            1,
            FLAG_LOADING | FLAG_CAN_EXPIRE_IF_UNLOADED,
        ),
    ];
    assert_eq!(TicketKind::ALL.len(), expected.len());
    for (kind, timeout, flags) in expected {
        assert_eq!(kind.timeout(), timeout);
        assert_eq!(kind.flags(), flags);
        assert_eq!(kind.has_timeout(), timeout != 0);
        assert_eq!(kind.persists(), flags & FLAG_PERSIST != 0);
        assert_eq!(kind.loads(), flags & FLAG_LOADING != 0);
        assert_eq!(kind.simulates(), flags & FLAG_SIMULATION != 0);
        assert_eq!(
            kind.keeps_dimension_active(),
            flags & FLAG_KEEP_DIMENSION_ACTIVE != 0
        );
    }
}

#[test]
fn ticket_selection_ignores_loading_only_types_and_uses_lowest_level() {
    let chunk = ChunkPos::new(3, -4);
    let mut storage = TicketStorage::default();
    storage.add(chunk, Ticket::new(TicketKind::PlayerLoading, 1));
    storage.add(chunk, Ticket::new(TicketKind::Unknown, 0));
    assert_eq!(storage.simulation_level(chunk), ABSENT_SIMULATION_LEVEL);
    assert_eq!(storage.loading_level(chunk), 0);

    storage.add(chunk, Ticket::new(TicketKind::Dragon, 31));
    storage.add(chunk, Ticket::new(TicketKind::Portal, 20));
    storage.add(chunk, Ticket::new(TicketKind::PlayerSimulation, 25));
    assert_eq!(storage.simulation_level(chunk), 20);
    assert_eq!(storage.loading_level(chunk), 0);
    assert_eq!(pack_chunk(chunk), -17_179_869_181);
}

#[test]
fn duplicate_ticket_resets_timeout_and_expiry_is_strictly_negative() {
    let chunk = ChunkPos::new(0, 0);
    let mut storage = TicketStorage::default();
    assert!(storage.add(chunk, Ticket::new(TicketKind::SpawnSearch, 4)));
    storage.purge_stale(true, |_| UpdatingHolderState::Missing);
    assert_eq!(storage.tickets(chunk)[0].ticks_left(), 0);
    assert!(!storage.add(chunk, Ticket::new(TicketKind::SpawnSearch, 4)));
    assert_eq!(storage.tickets(chunk)[0].ticks_left(), 1);
    assert!(
        storage
            .purge_stale(true, |_| UpdatingHolderState::Missing)
            .is_empty()
    );
    assert_eq!(storage.tickets(chunk)[0].ticks_left(), 0);
    let expired = storage.purge_stale(true, |_| UpdatingHolderState::Missing);
    assert_eq!(expired.len(), 1);
    assert_eq!(expired[0].kind, TicketKind::SpawnSearch);
    assert!(storage.tickets(chunk).is_empty());
}

#[test]
fn frozen_purge_and_unsaved_holder_preserve_timeout_except_unknown() {
    let chunk = ChunkPos::new(1, 1);
    let mut storage = TicketStorage::default();
    storage.add(chunk, Ticket::new(TicketKind::PlayerSpawn, 4));
    storage.add(chunk, Ticket::new(TicketKind::Unknown, 5));
    storage.purge_stale(false, |_| panic!("frozen purge must not query holders"));
    assert_eq!(storage.tickets(chunk)[0].ticks_left(), 20);
    assert_eq!(storage.tickets(chunk)[1].ticks_left(), 1);

    storage.purge_stale(true, |_| UpdatingHolderState::NotReadyForSaving);
    assert_eq!(storage.tickets(chunk)[0].ticks_left(), 20);
    assert_eq!(storage.tickets(chunk)[1].ticks_left(), 0);
    let expired = storage.purge_stale(true, |_| UpdatingHolderState::NotReadyForSaving);
    assert_eq!(expired[0].kind, TicketKind::Unknown);
    assert_eq!(storage.tickets(chunk)[0].ticks_left(), 20);

    storage.purge_stale(true, |_| UpdatingHolderState::ReadyForSaving);
    assert_eq!(storage.tickets(chunk)[0].ticks_left(), 19);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MockState {
    id: u8,
    block: bool,
    fluid: bool,
}

impl RandomTickState for MockState {
    fn block_is_randomly_ticking(&self) -> bool {
        self.block
    }

    fn fluid_is_randomly_ticking(&self) -> bool {
        self.fluid
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MockSection {
    eligible: bool,
    state: MockState,
}

#[derive(Debug)]
struct MockChunk {
    position: ChunkPos,
    bottom_section_y: i32,
    sections: Vec<MockSection>,
}

impl RandomTickChunk for MockChunk {
    type State = MockState;

    fn chunk_position(&self) -> ChunkPos {
        self.position
    }

    fn section_count(&self) -> usize {
        self.sections.len()
    }

    fn section_bottom_y(&self, section_index: usize) -> i32 {
        self.bottom_section_y
            .wrapping_add(section_index as i32)
            .wrapping_mul(16)
    }

    fn section_is_randomly_ticking(&self, section_index: usize) -> bool {
        self.sections[section_index].eligible
    }

    fn state_at(
        &self,
        section_index: usize,
        _local_x: u8,
        _local_y: u8,
        _local_z: u8,
    ) -> Self::State {
        self.sections[section_index].state
    }
}

#[derive(Debug)]
struct TapeRandom {
    values: VecDeque<i32>,
    calls: Vec<i32>,
}

impl TapeRandom {
    fn new(values: impl IntoIterator<Item = i32>) -> Self {
        Self {
            values: values.into_iter().collect(),
            calls: Vec::new(),
        }
    }
}

impl BoundedRandom for TapeRandom {
    fn next_i32(&mut self, upper_exclusive: i32) -> i32 {
        self.calls.push(upper_exclusive);
        self.values
            .pop_front()
            .expect("test random tape must cover every draw")
    }
}

#[test]
fn chunk_sampling_keeps_precipitation_section_and_snapshot_order_exact() {
    let old = MockState {
        id: 1,
        block: true,
        fluid: true,
    };
    let none = MockState {
        id: 0,
        block: false,
        fluid: false,
    };
    let later = MockState {
        id: 2,
        block: true,
        fluid: false,
    };
    let mut chunk = MockChunk {
        position: ChunkPos::new(2, -3),
        bottom_section_y: -4,
        sections: vec![
            MockSection {
                eligible: true,
                state: old,
            },
            MockSection {
                eligible: false,
                state: later,
            },
        ],
    };
    let events = RefCell::new(Vec::new());
    let mut positions = RandomPositionStream::new(0);
    let mut random = TapeRandom::new([1, 0, 1, 6]);
    let report = tick_chunk(
        &mut chunk,
        3,
        &mut positions,
        &mut random,
        |_, position, _| events.borrow_mut().push(("precip", 0, position.y)),
        |chunk, position, state, random| {
            events.borrow_mut().push(("block", state.id, position.y));
            if state.id == 1 {
                chunk.sections[0].state = none;
                chunk.sections[0].eligible = false;
                chunk.sections[1].eligible = true;
                assert_eq!(random.next_i32(7), 6);
            }
        },
        |_, position, state, _| {
            events.borrow_mut().push(("fluid", state.id, position.y));
        },
    );
    assert_eq!(report.precipitation_draws, 3);
    assert_eq!(report.precipitation_callbacks, 1);
    assert_eq!(report.admitted_sections, 2);
    assert_eq!(report.position_samples, 7);
    assert_eq!(report.block_callbacks, 4);
    assert_eq!(report.fluid_callbacks, 1);
    assert_eq!(random.calls, [48, 48, 48, 7]);
    let events = events.into_inner();
    assert_eq!(events[0].0, "precip");
    assert_eq!(&events[1..3], [("block", 1, -50), ("fluid", 1, -50)]);
    assert!(
        events[3..]
            .iter()
            .all(|event| event.0 == "block" && event.1 == 2)
    );
}

#[test]
fn speed_zero_and_negative_consume_nothing_and_maximum_is_not_clamped() {
    for speed in [0, -1] {
        let mut chunk = MockChunk {
            position: ChunkPos::new(0, 0),
            bottom_section_y: 0,
            sections: vec![MockSection {
                eligible: true,
                state: MockState {
                    id: 1,
                    block: true,
                    fluid: true,
                },
            }],
        };
        let mut positions = RandomPositionStream::new(9);
        let mut random = TapeRandom::new([]);
        let report = tick_chunk(
            &mut chunk,
            speed,
            &mut positions,
            &mut random,
            |_, _, _| panic!("no precipitation"),
            |_, _, _, _| panic!("no block"),
            |_, _, _, _| panic!("no fluid"),
        );
        assert_eq!(report, Default::default());
        assert_eq!(positions.value(), 9);
        assert!(random.calls.is_empty());
    }
    assert_eq!(MINIMUM_RANDOM_TICK_SPEED, 0);
    assert_eq!(DEFAULT_RANDOM_TICK_SPEED, 3);
    assert_eq!(MAXIMUM_RANDOM_TICK_SPEED, i32::MAX);
    assert_eq!(
        planned_attempts(i32::MAX, 24),
        (i32::MAX as u64, i32::MAX as u64 * 24)
    );
}

#[test]
fn section_counts_are_signed_short_admission_snapshots() {
    let mut counts = SectionRandomTickCounts::default();
    assert!(!counts.is_randomly_ticking());
    counts.replace(false, false, true, false);
    assert_eq!(counts.block(), 1);
    assert!(counts.is_randomly_ticking());
    counts.replace(true, false, false, true);
    assert_eq!(counts.block(), 0);
    assert_eq!(counts.fluid(), 1);

    counts.recalculate([
        MockState {
            id: 1,
            block: true,
            fluid: true,
        },
        MockState {
            id: 2,
            block: false,
            fluid: true,
        },
    ]);
    assert_eq!(counts.block(), 1);
    assert_eq!(counts.fluid(), 2);

    let mut wrapped = SectionRandomTickCounts::from_raw(i16::MAX, 0);
    wrapped.replace(false, false, true, false);
    assert_eq!(wrapped.block(), i16::MIN);
    assert!(!wrapped.is_randomly_ticking());
}

#[test]
fn cache_plan_propagates_distance_updates_even_when_frozen_or_debug() {
    let frozen = chunk_cache_tick_plan(false, false);
    assert!(!frozen.purge_stale_tickets);
    assert!(frozen.run_distance_updates);
    assert!(frozen.update_inhabited_time);
    assert!(!frozen.run_random_chunk_work);

    let debug = chunk_cache_tick_plan(true, true);
    assert!(debug.purge_stale_tickets);
    assert!(debug.run_distance_updates);
    assert!(debug.update_inhabited_time);
    assert!(!debug.run_random_chunk_work);

    assert_eq!(
        GAMEPLAY_CHUNK_STAGES,
        [
            GameplayChunkStage::ConstructNaturalSpawnState,
            GameplayChunkStage::ReadSpawnAndRandomTickRules,
            GameplayChunkStage::ShuffleSpawningChunks,
            GameplayChunkStage::NaturalSpawning,
            GameplayChunkStage::Thunder,
            GameplayChunkStage::PrecipitationAndRandomTicks,
        ]
    );
}

use ferrite_foundation::bounds::BlockBounds;
use ferrite_foundation::coordinate::{BlockPos, ChunkPos};
use ferrite_simulation::scheduled_tick::container::ChunkTickContainer;
use ferrite_simulation::scheduled_tick::level::{
    LevelScheduledTicks, LevelTickAdmission, SCHEDULED_TICK_CAP, ScheduleOutcome,
    ScheduledTickQueue,
};
use ferrite_simulation::scheduled_tick::record::{
    SavedTick, ScheduledTick, SubTickCounter, TickPriority,
};

fn pos(x: i32, y: i32, z: i32) -> BlockPos {
    BlockPos::new(x, y, z)
}

fn tick(
    type_identity: u16,
    position: BlockPos,
    trigger_tick: i64,
    priority: TickPriority,
    sub_tick_order: i64,
) -> ScheduledTick<u16> {
    ScheduledTick::new(
        type_identity,
        position,
        trigger_tick,
        priority,
        sub_tick_order,
    )
}

fn queue_with_chunks(chunks: impl IntoIterator<Item = ChunkPos>) -> ScheduledTickQueue<u16> {
    let mut queue = ScheduledTickQueue::new();
    for chunk in chunks {
        queue.register_container(chunk, ChunkTickContainer::new());
    }
    queue
}

fn all_active(_: ChunkPos) -> bool {
    true
}

#[test]
fn priority_codec_counter_and_creation_use_java_boundaries() {
    assert_eq!(TickPriority::from_value(-99), TickPriority::ExtremelyHigh);
    assert_eq!(TickPriority::from_value(-2), TickPriority::VeryHigh);
    assert_eq!(TickPriority::from_value(0), TickPriority::Normal);
    assert_eq!(TickPriority::from_value(2), TickPriority::VeryLow);
    assert_eq!(TickPriority::from_value(99), TickPriority::ExtremelyLow);
    assert_eq!(TickPriority::High.value(), -1);

    let mut counter = SubTickCounter::new(i64::MAX);
    let created = counter.create(7, pos(1, 2, 3), i64::MAX, 1, TickPriority::Low);
    assert_eq!(created.trigger_tick, i64::MIN);
    assert_eq!(created.sub_tick_order, i64::MAX);
    assert_eq!(counter.value(), i64::MIN);
}

#[test]
fn unregistered_requests_are_rejected_and_duplicates_never_replace_first() {
    let position = pos(1, 2, 3);
    let mut queue = ScheduledTickQueue::new();
    assert_eq!(
        queue.schedule(tick(1, position, 20, TickPriority::Normal, 0)),
        ScheduleOutcome::UnregisteredChunk
    );
    queue.register_container(position.chunk(), ChunkTickContainer::new());
    assert_eq!(
        queue.schedule(tick(1, position, 20, TickPriority::Normal, 0)),
        ScheduleOutcome::Queued
    );
    assert_eq!(
        queue.schedule(tick(1, position, 1, TickPriority::ExtremelyHigh, 1)),
        ScheduleOutcome::Duplicate
    );
    assert_eq!(
        queue.schedule(tick(2, position, 1, TickPriority::ExtremelyHigh, 2)),
        ScheduleOutcome::Queued
    );

    let mut observed = Vec::new();
    queue.tick(1, SCHEDULED_TICK_CAP, all_active, |_, due| {
        observed.push(due.type_identity)
    });
    assert_eq!(observed, [2]);
    queue.tick(20, SCHEDULED_TICK_CAP, all_active, |_, due| {
        observed.push(due.type_identity)
    });
    assert_eq!(observed, [2, 1]);
}

#[test]
fn local_trigger_order_precedes_cross_chunk_priority_merge() {
    let mut queue = queue_with_chunks([ChunkPos::new(0, 0), ChunkPos::new(1, 0)]);
    queue.schedule(tick(1, pos(0, 0, 0), -100, TickPriority::ExtremelyLow, 0));
    queue.schedule(tick(2, pos(1, 0, 0), 0, TickPriority::ExtremelyHigh, 1));
    queue.schedule(tick(3, pos(16, 0, 0), -10, TickPriority::Normal, 2));

    let mut observed = Vec::new();
    queue.tick(0, SCHEDULED_TICK_CAP, all_active, |_, due| {
        observed.push(due.type_identity)
    });
    assert_eq!(observed, [3, 1, 2]);
}

#[test]
fn collection_is_a_snapshot_and_current_membership_moves_before_callback() {
    let position = pos(0, 0, 0);
    let next_position = pos(1, 0, 0);
    let mut queue = queue_with_chunks([position.chunk()]);
    queue.schedule(tick(1, position, 4, TickPriority::Normal, 0));
    queue.schedule(tick(2, next_position, 4, TickPriority::Normal, 1));

    let mut observed = Vec::new();
    queue.tick(4, SCHEDULED_TICK_CAP, all_active, |scheduler, due| {
        assert!(!scheduler.has_scheduled_tick(due.position, &due.type_identity));
        assert!(!scheduler.will_tick_this_tick(due.position, &due.type_identity));
        if due.type_identity == 1 {
            assert!(scheduler.will_tick_this_tick(next_position, &2));
            assert_eq!(
                scheduler.schedule(tick(1, position, 4, TickPriority::ExtremelyHigh, 2)),
                ScheduleOutcome::Queued
            );
        }
        observed.push(due.type_identity);
    });
    assert_eq!(observed, [1, 2]);
    assert!(queue.has_scheduled_tick(position, &1));

    queue.tick(4, SCHEDULED_TICK_CAP, all_active, |_, due| {
        observed.push(due.type_identity)
    });
    assert_eq!(observed, [1, 2, 1]);
}

#[test]
fn inactive_due_work_waits_and_type_mismatch_is_consumed() {
    let inactive = ChunkPos::new(0, 0);
    let active = ChunkPos::new(1, 0);
    let mut queue = queue_with_chunks([inactive, active]);
    queue.schedule(tick(1, pos(0, 0, 0), 5, TickPriority::Normal, 0));
    queue.schedule(tick(2, pos(16, 0, 0), 5, TickPriority::Normal, 1));

    let mut callbacks = Vec::new();
    queue.tick_matching(
        100,
        SCHEDULED_TICK_CAP,
        |chunk| chunk == active,
        |_, identity| *identity == 1,
        |_, due| callbacks.push(due.type_identity),
    );
    assert!(callbacks.is_empty());
    assert!(queue.has_scheduled_tick(pos(0, 0, 0), &1));
    assert!(!queue.has_scheduled_tick(pos(16, 0, 0), &2));

    queue.tick_matching(
        100,
        SCHEDULED_TICK_CAP,
        all_active,
        |_, identity| *identity == 1,
        |_, due| callbacks.push(due.type_identity),
    );
    assert_eq!(callbacks, [1]);
}

#[test]
fn block_and_fluid_caps_are_independent_backlog_limits() {
    let chunk = ChunkPos::new(0, 0);
    let mut level = LevelScheduledTicks::<u16, u16>::default();
    level
        .blocks
        .register_container(chunk, ChunkTickContainer::new());
    level
        .fluids
        .register_container(chunk, ChunkTickContainer::new());
    for y in 0..=SCHEDULED_TICK_CAP as i32 {
        let position = pos(0, y, 0);
        level
            .blocks
            .schedule(tick(1, position, 0, TickPriority::Normal, i64::from(y)));
        level
            .fluids
            .schedule(tick(2, position, 0, TickPriority::Normal, i64::from(y)));
    }

    let mut block_callbacks = 0;
    let mut fluid_callbacks = 0;
    let counts = level.tick(
        LevelTickAdmission {
            runs_normally: true,
            debug_level: false,
        },
        0,
        &mut all_active,
        &mut |_, _| block_callbacks += 1,
        &mut |_, _| fluid_callbacks += 1,
    );
    assert_eq!(counts.blocks, SCHEDULED_TICK_CAP);
    assert_eq!(counts.fluids, SCHEDULED_TICK_CAP);
    assert_eq!(block_callbacks, SCHEDULED_TICK_CAP);
    assert_eq!(fluid_callbacks, SCHEDULED_TICK_CAP);
    assert_eq!(level.blocks.count(), 1);
    assert_eq!(level.fluids.count(), 1);

    let counts = level.tick(
        LevelTickAdmission {
            runs_normally: true,
            debug_level: false,
        },
        0,
        &mut all_active,
        &mut |_, _| block_callbacks += 1,
        &mut |_, _| fluid_callbacks += 1,
    );
    assert_eq!(counts.blocks, 1);
    assert_eq!(counts.fluids, 1);
}

#[test]
fn frozen_and_debug_levels_skip_both_queues_without_consuming_work() {
    let chunk = ChunkPos::new(0, 0);
    let mut level = LevelScheduledTicks::<u16, u16>::default();
    level
        .blocks
        .register_container(chunk, ChunkTickContainer::new());
    level
        .fluids
        .register_container(chunk, ChunkTickContainer::new());
    level
        .blocks
        .schedule(tick(1, pos(0, 0, 0), 0, TickPriority::Normal, 0));
    level
        .fluids
        .schedule(tick(2, pos(0, 0, 0), 0, TickPriority::Normal, 0));
    for admission in [
        LevelTickAdmission {
            runs_normally: false,
            debug_level: false,
        },
        LevelTickAdmission {
            runs_normally: true,
            debug_level: true,
        },
    ] {
        assert_eq!(
            level.tick(
                admission,
                20,
                &mut all_active,
                &mut |_, _| panic!("frozen block callback"),
                &mut |_, _| panic!("frozen fluid callback"),
            ),
            Default::default()
        );
    }
    assert_eq!(level.blocks.count(), 1);
    assert_eq!(level.fluids.count(), 1);
}

#[test]
fn save_narrows_delay_and_reload_rebuilds_negative_sub_orders() {
    let saved = tick(
        1,
        pos(0, 0, 0),
        i64::from(i32::MAX) + 10,
        TickPriority::High,
        99,
    )
    .to_saved(0);
    assert_eq!(saved.delay, i32::MIN + 9);

    let pending = vec![
        SavedTick::new(1, pos(0, 0, 0), 20, TickPriority::Normal),
        SavedTick::new(2, pos(1, 0, 0), -2, TickPriority::High),
    ];
    let mut container = ChunkTickContainer::from_saved(pending);
    assert_eq!(container.count(), 2);
    assert!(container.has_scheduled_tick(pos(0, 0, 0), &1));
    assert_eq!(container.pack(50).len(), 2);
    container.unpack(50);
    let mut restored = container.all_ticks();
    restored.sort_by_key(|entry| entry.sub_tick_order);
    assert_eq!(restored[0].sub_tick_order, -2);
    assert_eq!(restored[0].trigger_tick, 70);
    assert_eq!(restored[1].sub_tick_order, -1);
    assert_eq!(restored[1].trigger_tick, 48);
}

#[test]
fn unloading_preserves_container_and_saved_positive_delay_starts_at_reload() {
    let chunk = ChunkPos::new(0, 0);
    let mut queue = queue_with_chunks([chunk]);
    queue.schedule(tick(1, pos(0, 0, 0), 120, TickPriority::Normal, 0));
    let container = queue.unregister_container(chunk).unwrap();
    let saved = container.pack(100);
    assert_eq!(saved[0].delay, 20);
    assert_eq!(
        queue.schedule(tick(2, pos(0, 1, 0), 0, TickPriority::Normal, 1)),
        ScheduleOutcome::UnregisteredChunk
    );

    queue.register_container(chunk, ChunkTickContainer::from_saved(saved));
    assert!(queue.unpack_container(chunk, 1_000));
    let mut callbacks = 0;
    queue.tick(1_019, SCHEDULED_TICK_CAP, all_active, |_, _| callbacks += 1);
    assert_eq!(callbacks, 0);
    queue.tick(1_020, SCHEDULED_TICK_CAP, all_active, |_, _| callbacks += 1);
    assert_eq!(callbacks, 1);
}

#[test]
fn clear_area_is_inclusive_and_keeps_lazy_query_membership_stale_until_cleanup() {
    let mut queue = queue_with_chunks([ChunkPos::new(0, 0)]);
    queue.schedule(tick(1, pos(0, 0, 0), 0, TickPriority::Normal, 0));
    queue.schedule(tick(2, pos(1, 0, 0), 0, TickPriority::Normal, 1));
    queue.schedule(tick(3, pos(1, 0, 0), 20, TickPriority::Low, 2));
    let cleared = BlockBounds::new(pos(1, 0, 0), pos(1, 0, 0)).unwrap();
    let mut callbacks = Vec::new();
    queue.tick(0, SCHEDULED_TICK_CAP, all_active, |scheduler, due| {
        callbacks.push(due.type_identity);
        if due.type_identity == 1 {
            assert!(scheduler.will_tick_this_tick(pos(1, 0, 0), &2));
            scheduler.clear_area(cleared);
            assert!(scheduler.will_tick_this_tick(pos(1, 0, 0), &2));
        }
    });
    assert_eq!(callbacks, [1]);
    assert_eq!(queue.count(), 0);
    assert!(!queue.will_tick_this_tick(pos(1, 0, 0), &2));
}

#[test]
fn copy_area_includes_run_collected_and_queued_records_with_shifted_sub_orders() {
    let mut queue = queue_with_chunks([ChunkPos::new(0, 0), ChunkPos::new(1, 0)]);
    for (identity, x, sub_order) in [(1, 0, 2), (2, 1, 4), (3, 2, 9)] {
        queue.schedule(tick(
            identity,
            pos(x, 0, 0),
            0,
            TickPriority::Normal,
            sub_order,
        ));
    }
    let source_area = BlockBounds::new(pos(0, 0, 0), pos(2, 0, 0)).unwrap();
    queue.tick(0, 2, all_active, |scheduler, due| {
        if due.type_identity == 1 {
            scheduler.copy_area(source_area, pos(16, 0, 0));
        }
    });

    let mut copied = queue.container(ChunkPos::new(1, 0)).unwrap().all_ticks();
    copied.sort_by_key(|entry| entry.position.x);
    assert_eq!(
        copied
            .iter()
            .map(|entry| (entry.type_identity, entry.position.x, entry.sub_tick_order))
            .collect::<Vec<_>>(),
        [(1, 16, 10), (2, 17, 12), (3, 18, 17)]
    );
}

#[test]
fn copy_from_another_queue_obeys_destination_deduplication() {
    let area = BlockBounds::new(pos(0, 0, 0), pos(1, 0, 0)).unwrap();
    let mut source = queue_with_chunks([ChunkPos::new(0, 0)]);
    source.schedule(tick(1, pos(0, 0, 0), 10, TickPriority::Low, 4));
    source.schedule(tick(2, pos(1, 0, 0), 11, TickPriority::High, 7));
    let mut destination = queue_with_chunks([ChunkPos::new(1, 0)]);
    destination.schedule(tick(1, pos(16, 0, 0), 99, TickPriority::Normal, 0));
    destination.copy_area_from(&source, area, pos(16, 0, 0));
    assert_eq!(destination.count(), 2);
    assert!(destination.has_scheduled_tick(pos(17, 0, 0), &2));
}

#[test]
fn equal_restored_heads_use_ferrites_stable_chunk_fallback_only() {
    fn restored_order(registration: [ChunkPos; 2]) -> Vec<ChunkPos> {
        let mut queue = ScheduledTickQueue::new();
        for chunk in registration {
            let position = pos(chunk.x * 16, 0, chunk.z * 16);
            queue.register_container(
                chunk,
                ChunkTickContainer::from_saved(vec![SavedTick::new(
                    1,
                    position,
                    0,
                    TickPriority::Normal,
                )]),
            );
            queue.unpack_container(chunk, 0);
        }
        let mut order = Vec::new();
        queue.tick(0, SCHEDULED_TICK_CAP, all_active, |_, due| {
            order.push(due.position.chunk())
        });
        order
    }

    let first = ChunkPos::new(-1, 0);
    let second = ChunkPos::new(1, 0);
    assert_eq!(restored_order([first, second]), [first, second]);
    assert_eq!(restored_order([second, first]), [first, second]);
}

#[test]
fn pack_orders_live_records_by_sub_order_and_keeps_saved_fields() {
    let mut container = ChunkTickContainer::new();
    container.schedule(tick(1, pos(0, 0, 0), 110, TickPriority::VeryHigh, 8));
    container.schedule(tick(2, pos(1, 0, 0), 90, TickPriority::VeryLow, 3));
    let packed = container.pack(100);
    assert_eq!(
        packed
            .iter()
            .map(|entry| (entry.type_identity, entry.delay, entry.priority))
            .collect::<Vec<_>>(),
        [
            (2, -10, TickPriority::VeryLow),
            (1, 10, TickPriority::VeryHigh)
        ]
    );
}

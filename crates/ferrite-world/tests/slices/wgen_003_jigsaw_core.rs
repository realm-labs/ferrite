use std::collections::{BTreeMap, VecDeque};
use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::direction::Direction;
use ferrite_world::generation::feature::random::GenerationRandom;
use ferrite_world::generation::structure::BlockBox;
use ferrite_world::generation::structure::jigsaw::{
    AliasBinding, Connector, ElementKind, ExpansionConfig, FreeSpace, JigsawError, JigsawPiece,
    JigsawStartConfig, JigsawStartRequest, Joint, Padding, PoolElement, PriorityQueue, Projection,
    Rotation, TemplatePool, can_attach, expand_pieces, generate_jigsaw_start, resolve_aliases,
};

#[test]
fn pool_weights_expand_and_empty_fallback_is_explicit() {
    let stone = element("stone", [3, 2, 1]);
    let pool = TemplatePool::new(
        "minecraft:empty",
        vec![(stone.clone(), 2), (PoolElement::empty(), 1)],
    )
    .unwrap();
    assert_eq!(
        pool.expanded(),
        [stone.clone(), stone, PoolElement::empty()]
    );
    assert_eq!(pool.maximum_y_span(), 2);
    assert_eq!(
        TemplatePool::new("empty", vec![(PoolElement::empty(), 0)]),
        Err(JigsawError::Weight(0))
    );
}

#[test]
fn alias_bindings_use_list_order_one_draw_per_choice_and_reject_duplicates() {
    let mut random = ScriptRandom::new([1, 0]);
    let aliases = resolve_aliases(
        &[
            AliasBinding::Random {
                alias: "a".into(),
                targets: vec![("one".into(), 1), ("two".into(), 2)],
            },
            AliasBinding::RandomGroup(vec![
                (
                    vec![AliasBinding::Direct {
                        alias: "b".into(),
                        target: "three".into(),
                    }],
                    1,
                ),
                (
                    vec![AliasBinding::Direct {
                        alias: "b".into(),
                        target: "four".into(),
                    }],
                    1,
                ),
            ]),
        ],
        &mut random,
    )
    .unwrap();
    assert_eq!(aliases["a"], "two");
    assert_eq!(aliases["b"], "three");
    assert_eq!(random.bounds, [3, 2]);

    assert_eq!(
        resolve_aliases(
            &[
                AliasBinding::Direct {
                    alias: "same".into(),
                    target: "one".into(),
                },
                AliasBinding::Direct {
                    alias: "same".into(),
                    target: "two".into(),
                },
            ],
            &mut ScriptRandom::new([]),
        ),
        Err(JigsawError::DuplicateAlias("same".into()))
    );
}

#[test]
fn source_joint_alone_controls_top_alignment() {
    let source = connector(Direction::East, Direction::Up, Joint::Aligned, "door");
    let mut target = connector(Direction::West, Direction::North, Joint::Rollable, "unused");
    target.name = "door".into();
    assert!(!can_attach(&source, &target));

    let mut rollable = source;
    rollable.joint = Joint::Rollable;
    assert!(can_attach(&rollable, &target));
}

#[test]
fn connectors_shuffle_before_stable_descending_priority_sort() {
    let mut value = element("room", [2, 2, 2]);
    value.connectors = vec![
        prioritized("a", 1),
        prioritized("b", 2),
        prioritized("c", 1),
    ];
    let ordered = value.ordered_connectors(Rotation::None, &mut ScriptRandom::new([0, 1]));
    assert_eq!(ordered[0].name, "b");
    assert_eq!(
        ordered[1..]
            .iter()
            .map(|connector| connector.name.as_str())
            .collect::<Vec<_>>(),
        ["c", "a"]
    );
}

#[test]
fn priority_queue_is_highest_first_fifo_and_allows_preemption() {
    let mut queue = PriorityQueue::default();
    queue.push(1, "low-a");
    queue.push(1, "low-b");
    queue.push(3, "high");
    assert_eq!(queue.pop(), Some("high"));
    assert_eq!(queue.pop(), Some("low-a"));
    queue.push(5, "new-high");
    assert_eq!(queue.pop(), Some("new-high"));
    assert_eq!(queue.pop(), Some("low-b"));
    assert!(queue.is_empty());
}

#[test]
fn free_space_checks_containment_then_quarter_deflated_collision() {
    let allowed = BlockBox::new(BlockPos::new(0, 0, 0), BlockPos::new(20, 20, 20)).unwrap();
    let mut free = FreeSpace::new(allowed);
    free.subtract(BlockBox::new(BlockPos::new(5, 5, 5), BlockPos::new(10, 10, 10)).unwrap());

    assert!(free.admits_deflated_quarter(
        BlockBox::new(BlockPos::new(11, 5, 5), BlockPos::new(14, 8, 8)).unwrap()
    ));
    assert!(!free.admits_deflated_quarter(
        BlockBox::new(BlockPos::new(10, 5, 5), BlockPos::new(14, 8, 8)).unwrap()
    ));
    assert!(!free.admits_deflated_quarter(
        BlockBox::new(BlockPos::new(19, 0, 0), BlockPos::new(21, 1, 1)).unwrap()
    ));
}

#[test]
fn shared_zero_padding_is_distinct_from_an_encoded_zero_record() {
    assert!(Padding::ZERO.is_shared_zero());
    assert!(!Padding::new(0, 0).is_shared_zero());
}

#[test]
fn start_transaction_aligns_named_connector_projects_and_preserves_zero_depth_quirk() {
    let mut start = element("start", [3, 4, 5]);
    let mut selected = connector(Direction::East, Direction::Up, Joint::Rollable, "unused");
    selected.local_position = BlockPos::new(1, 2, 3);
    selected.name = "minecraft:start".into();
    selected.pool = "missing".into();
    start.connectors.push(selected);
    let pools = BTreeMap::from([(
        "start".into(),
        TemplatePool::new("empty", vec![(start, 1)]).unwrap(),
    )]);
    let config = JigsawStartConfig {
        dimension_min_y: -64,
        dimension_max_y: 319,
        padding: Padding::ZERO,
        maximum_depth: 1,
        horizontal_distance: 80,
        vertical_distance: 80,
        use_expansion_hack: false,
    };
    let generated = generate_jigsaw_start(
        JigsawStartRequest {
            position: BlockPos::new(10, 20, 30),
            pool: "start",
            connector_name: Some("minecraft:start"),
            aliases: &BTreeMap::new(),
            config,
        },
        &pools,
        &mut ScriptRandom::new([0; 16]),
        |x, z| {
            assert_eq!((x, z), (10, 29));
            Some(7)
        },
        |_, _| 0,
    )
    .unwrap();
    assert_eq!(generated.stub_position, BlockPos::new(10, 29, 29));
    assert_eq!(generated.pieces.len(), 1);
    assert_eq!(generated.pieces[0].position, BlockPos::new(9, 26, 27));
    assert_eq!(generated.pieces[0].bounding_box.minimum.y, 26);
    assert_eq!(generated.pieces[0].ground_level_delta, 1);

    let zero_depth = generate_jigsaw_start(
        JigsawStartRequest {
            position: BlockPos::new(10, 20, 30),
            pool: "start",
            connector_name: None,
            aliases: &BTreeMap::new(),
            config: JigsawStartConfig {
                maximum_depth: 0,
                ..config
            },
        },
        &pools,
        &mut ScriptRandom::new([0; 8]),
        |_, _| None,
        |_, _| 0,
    )
    .unwrap();
    assert!(zero_depth.pieces.is_empty());
}

#[test]
fn encoded_zero_padding_enforces_dimension_bounds_while_shared_zero_skips_them() {
    let start = element("start", [1, 2, 1]);
    let pools = BTreeMap::from([(
        "start".into(),
        TemplatePool::new("empty", vec![(start, 1)]).unwrap(),
    )]);
    let base = JigsawStartConfig {
        dimension_min_y: 0,
        dimension_max_y: 0,
        padding: Padding::new(0, 0),
        maximum_depth: 0,
        horizontal_distance: 1,
        vertical_distance: 1,
        use_expansion_hack: false,
    };
    assert!(
        generate_jigsaw_start(
            JigsawStartRequest {
                position: BlockPos::new(0, 0, 0),
                pool: "start",
                connector_name: None,
                aliases: &BTreeMap::new(),
                config: base,
            },
            &pools,
            &mut ScriptRandom::new([0; 4]),
            |_, _| None,
            |_, _| 0,
        )
        .is_none()
    );
    assert!(
        generate_jigsaw_start(
            JigsawStartRequest {
                position: BlockPos::new(0, 0, 0),
                pool: "start",
                connector_name: None,
                aliases: &BTreeMap::new(),
                config: JigsawStartConfig {
                    padding: Padding::ZERO,
                    ..base
                },
            },
            &pools,
            &mut ScriptRandom::new([0; 4]),
            |_, _| None,
            |_, _| 0,
        )
        .is_some()
    );
}

#[test]
fn maximum_depth_uses_fallback_and_retains_depth_plus_one_as_terminal() {
    let mut center_element = element("center", [1, 1, 1]);
    center_element.connectors = vec![connector(
        Direction::East,
        Direction::Up,
        Joint::Rollable,
        "door",
    )];
    let center = piece(center_element);
    let mut child = element("fallback-child", [1, 1, 1]);
    let mut target = connector(Direction::West, Direction::North, Joint::Aligned, "unused");
    target.name = "door".into();
    child.connectors.push(target);
    let pools = BTreeMap::from([
        (
            "pool".into(),
            TemplatePool::new("fallback", vec![(element("ignored-primary", [1, 1, 1]), 1)])
                .unwrap(),
        ),
        (
            "fallback".into(),
            TemplatePool::new("empty", vec![(child, 1)]).unwrap(),
        ),
        (
            "empty".into(),
            TemplatePool::new("empty", Vec::new()).unwrap(),
        ),
    ]);

    let pieces = expand_pieces(
        center,
        &pools,
        &BTreeMap::new(),
        ExpansionConfig {
            allowed: BlockBox::new(BlockPos::new(-10, -10, -10), BlockPos::new(10, 10, 10))
                .unwrap(),
            maximum_depth: 1,
            use_expansion_hack: false,
        },
        &mut ScriptRandom::new([0; 16]),
        |_, _| 0,
    );
    assert_eq!(pieces.len(), 2);
    assert_eq!(pieces[1].depth, 1);
    assert!(matches!(
        pieces[1].element.kind,
        ElementKind::Single { ref template, .. } if template == "fallback-child"
    ));
    assert_eq!(pieces[0].junctions.len(), 1);
    assert_eq!(pieces[1].junctions.len(), 1);
}

#[test]
fn empty_candidate_terminates_before_a_later_nonempty_fallback_entry() {
    let mut center_element = element("center", [1, 1, 1]);
    center_element.connectors = vec![connector(
        Direction::East,
        Direction::Up,
        Joint::Rollable,
        "door",
    )];
    let mut child = element("late", [1, 1, 1]);
    let mut target = connector(Direction::West, Direction::Up, Joint::Rollable, "unused");
    target.name = "door".into();
    child.connectors.push(target);
    let pools = BTreeMap::from([
        (
            "pool".into(),
            TemplatePool::new("fallback", Vec::new()).unwrap(),
        ),
        (
            "fallback".into(),
            TemplatePool::new("empty", vec![(PoolElement::empty(), 1), (child, 1)]).unwrap(),
        ),
    ]);
    let pieces = expand_pieces(
        piece(center_element),
        &pools,
        &BTreeMap::new(),
        ExpansionConfig {
            allowed: BlockBox::new(BlockPos::new(-10, -10, -10), BlockPos::new(10, 10, 10))
                .unwrap(),
            maximum_depth: 1,
            use_expansion_hack: false,
        },
        &mut ScriptRandom::new([0, 1]),
        |_, _| 0,
    );
    assert_eq!(pieces.len(), 1);
}

fn element(name: &str, size: [i32; 3]) -> PoolElement {
    PoolElement {
        kind: ElementKind::Single {
            template: name.into(),
            legacy: false,
        },
        projection: Projection::Rigid,
        size,
        connectors: Vec::new(),
        ground_level_delta: 1,
        processor_list: None,
    }
}

fn connector(front: Direction, top: Direction, joint: Joint, target: &str) -> Connector {
    Connector {
        local_position: BlockPos::new(0, 0, 0),
        front,
        top,
        joint,
        name: "name".into(),
        target: target.into(),
        pool: "pool".into(),
        selection_priority: 0,
        placement_priority: 0,
    }
}

fn prioritized(name: &str, priority: i32) -> Connector {
    let mut connector = connector(Direction::North, Direction::Up, Joint::Rollable, "target");
    connector.name = name.into();
    connector.selection_priority = priority;
    connector
}

fn piece(element: PoolElement) -> JigsawPiece {
    JigsawPiece {
        bounding_box: element
            .box_at(BlockPos::new(0, 0, 0), Rotation::None)
            .unwrap(),
        element,
        position: BlockPos::new(0, 0, 0),
        rotation: Rotation::None,
        ground_level_delta: 0,
        junctions: Vec::new(),
        depth: 0,
    }
}

struct ScriptRandom {
    values: VecDeque<u32>,
    bounds: Vec<u32>,
}

impl ScriptRandom {
    fn new(values: impl IntoIterator<Item = u32>) -> Self {
        Self {
            values: values.into_iter().collect(),
            bounds: Vec::new(),
        }
    }
}

impl GenerationRandom for ScriptRandom {
    fn next_u32(&mut self, bound: NonZeroU32) -> u32 {
        self.bounds.push(bound.get());
        self.values.pop_front().unwrap_or(0) % bound.get()
    }

    fn next_f32(&mut self) -> f32 {
        unreachable!()
    }

    fn next_f64(&mut self) -> f64 {
        unreachable!()
    }

    fn next_gaussian(&mut self) -> f64 {
        unreachable!()
    }
}

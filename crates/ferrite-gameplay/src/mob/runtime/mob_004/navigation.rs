//! Path creation/search limits, recomputation, following, and stuck recovery.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PathBudgets {
    pub max_path_length: f32,
    pub base_visit_budget: u32,
    pub adjusted_visit_budget: u32,
}

#[must_use]
pub fn path_budgets(
    follow_range: f32,
    required_path_length: f32,
    visit_multiplier: f32,
    reset_for_required_length: bool,
) -> PathBudgets {
    let max_path_length = follow_range.max(required_path_length);
    let visit_length = if reset_for_required_length {
        max_path_length
    } else {
        follow_range
    };
    let base_visit_budget = (visit_length * 16.0).floor().max(0.0) as u32;
    PathBudgets {
        max_path_length,
        base_visit_budget,
        adjusted_visit_budget: (base_visit_budget as f32 * visit_multiplier)
            .floor()
            .max(0.0) as u32,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathCreation {
    RejectEmptyTargets,
    RejectBelowMinimum,
    RejectCannotUpdate,
    ReuseLivePath,
    Search,
}

#[must_use]
pub const fn path_creation(
    targets_empty: bool,
    below_level_minimum: bool,
    can_update: bool,
    live_path_target_requested: bool,
) -> PathCreation {
    if targets_empty {
        PathCreation::RejectEmptyTargets
    } else if below_level_minimum {
        PathCreation::RejectBelowMinimum
    } else if !can_update {
        PathCreation::RejectCannotUpdate
    } else if live_path_target_requested {
        PathCreation::ReuseLivePath
    } else {
        PathCreation::Search
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveToPath {
    ClearAndFail,
    TrimCauldronsAndStart,
    FailMissingOrDone,
}

#[must_use]
pub const fn move_to_path(path_present: bool, path_has_node: bool, path_done: bool) -> MoveToPath {
    if !path_present {
        MoveToPath::ClearAndFail
    } else if !path_has_node || path_done {
        MoveToPath::FailMissingOrDone
    } else {
        MoveToPath::TrimCauldronsAndStart
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NavigationTickOrder {
    pub increment_tick: bool,
    pub retry_delayed_recompute_first: bool,
    pub follow_when_permitted: bool,
    pub write_waypoint_to_move_control_last: bool,
}

pub const NAVIGATION_TICK_ORDER: NavigationTickOrder = NavigationTickOrder {
    increment_tick: true,
    retry_delayed_recompute_first: true,
    follow_when_permitted: true,
    write_waypoint_to_move_control_last: true,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SearchNeighbor {
    pub g: f32,
    pub h: f32,
    pub expandable: bool,
}

#[must_use]
pub fn search_neighbor(
    current_g: f32,
    edge_distance: f32,
    cost_malus: f32,
    nearest_target_distance: f32,
    euclidean_from_start: f32,
    walked_distance: f32,
    max_path_length: f32,
) -> SearchNeighbor {
    SearchNeighbor {
        g: current_g + edge_distance + cost_malus,
        h: 1.5 * nearest_target_distance,
        expandable: euclidean_from_start < max_path_length && walked_distance < max_path_length,
    }
}

#[must_use]
pub const fn expansion_allowed(processed_count: u32, adjusted_visit_budget: u32) -> bool {
    processed_count < adjusted_visit_budget
}

#[must_use]
pub const fn target_reached(manhattan_distance: u32, reach_range: u32) -> bool {
    manhattan_distance <= reach_range
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PathAlternative {
    pub distance_to_target: f32,
    pub node_count: u32,
    pub reached: bool,
}

#[must_use]
pub fn choose_alternative(alternatives_in_iteration_order: &[PathAlternative]) -> Option<usize> {
    let reached = alternatives_in_iteration_order
        .iter()
        .enumerate()
        .filter(|(_, path)| path.reached)
        .min_by_key(|(_, path)| path.node_count)
        .map(|(index, _)| index);
    reached.or_else(|| {
        alternatives_in_iteration_order
            .iter()
            .enumerate()
            .min_by(|(_, left), (_, right)| {
                left.distance_to_target
                    .total_cmp(&right.distance_to_target)
                    .then(left.node_count.cmp(&right.node_count))
            })
            .map(|(index, _)| index)
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Recompute {
    pub recompute_now: bool,
    pub mark_delayed: bool,
}

#[must_use]
pub const fn recompute(game_time: i64, last_recompute_time: i64, can_update: bool) -> Recompute {
    let cadence = game_time.wrapping_sub(last_recompute_time) > 20;
    Recompute {
        recompute_now: cadence && can_update,
        mark_delayed: !cadence || !can_update,
    }
}

#[must_use]
pub fn waypoint_reached(
    horizontal_x: f64,
    horizontal_z: f64,
    vertical: f64,
    width_tolerance: f64,
    vertical_limit: f64,
) -> bool {
    horizontal_x.abs() < width_tolerance
        && horizontal_z.abs() < width_tolerance
        && vertical.abs() < vertical_limit
}

#[must_use]
pub fn displacement_stuck(
    navigation_ticks_since_sample: u32,
    displacement: f64,
    speed: f64,
) -> bool {
    let speed_factor = if speed >= 1.0 { speed } else { speed * speed };
    navigation_ticks_since_sample > 100 && displacement < speed_factor * 100.0 * 0.25
}

#[must_use]
pub fn expected_node_ticks(distance: f64, mob_speed: f64) -> Option<f64> {
    (mob_speed != 0.0).then_some(distance / mob_speed * 20.0)
}

#[must_use]
pub fn node_timed_out(accumulated_time: f64, expected_ticks: Option<f64>) -> bool {
    expected_ticks.is_some_and(|expected| accumulated_time > expected * 3.0)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NodeChange {
    pub recompute_expected_limit: bool,
    pub reset_accumulated_timeout: bool,
}

pub const NODE_CHANGE: NodeChange = NodeChange {
    recompute_expected_limit: true,
    reset_accumulated_timeout: false,
};

#[must_use]
pub const fn corner_cut_allowed(
    fire_neighbor: bool,
    damaging_neighbor: bool,
    walkable_door: bool,
    geometry_allows: bool,
) -> bool {
    geometry_allows && !fire_neighbor && !damaging_neighbor && !walkable_door
}

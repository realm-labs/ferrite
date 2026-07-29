use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::direction::Direction;

pub const DEFAULT_MAX_CHAINED_UPDATES: i32 = 1_000_000;
pub const ORDINARY_DIRECTIONS: [Direction; 6] = [
    Direction::West,
    Direction::East,
    Direction::Down,
    Direction::Up,
    Direction::North,
    Direction::South,
];
pub const SHAPE_DIRECTIONS: [Direction; 6] = [
    Direction::West,
    Direction::East,
    Direction::North,
    Direction::South,
    Direction::Down,
    Direction::Up,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NeighborWorkKind {
    RereadReceiver,
    CapturedReceiver,
    Shape,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NeighborWork {
    pub origin: BlockPos,
    pub receivers: Vec<BlockPos>,
    pub kind: NeighborWorkKind,
}

impl NeighborWork {
    pub fn single(origin: BlockPos, receiver: BlockPos, kind: NeighborWorkKind) -> Self {
        Self {
            origin,
            receivers: vec![receiver],
            kind,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NeighborStep {
    pub origin: BlockPos,
    pub receiver: BlockPos,
    pub kind: NeighborWorkKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NeighborCollectorReport {
    pub steps: Vec<NeighborStep>,
    pub submitted: usize,
    pub first_discarded: Option<BlockPos>,
}

#[derive(Debug, Clone)]
struct WorkCursor {
    work: NeighborWork,
    next_receiver: usize,
}

#[derive(Debug)]
pub struct NeighborCollector {
    limit: i32,
}

impl NeighborCollector {
    pub const fn new(limit: i32) -> Self {
        Self { limit }
    }

    pub fn run<F>(&self, root: NeighborWork, mut execute: F) -> NeighborCollectorReport
    where
        F: FnMut(NeighborStep) -> Vec<NeighborWork>,
    {
        if self.limit == 0 {
            return NeighborCollectorReport {
                steps: Vec::new(),
                submitted: 0,
                first_discarded: Some(root.origin),
            };
        }
        let mut stack = vec![WorkCursor {
            work: root,
            next_receiver: 0,
        }];
        let mut submitted = 1;
        let mut first_discarded = None;
        let mut steps = Vec::new();

        while let Some(mut cursor) = stack.pop() {
            let Some(&receiver) = cursor.work.receivers.get(cursor.next_receiver) else {
                continue;
            };
            cursor.next_receiver += 1;
            let step = NeighborStep {
                origin: cursor.work.origin,
                receiver,
                kind: cursor.work.kind,
            };
            if cursor.next_receiver < cursor.work.receivers.len() {
                stack.push(cursor);
            }
            let mut admitted = Vec::new();
            for nested in execute(step) {
                if self.limit >= 0 && submitted >= self.limit as usize {
                    first_discarded.get_or_insert(nested.origin);
                } else {
                    submitted += 1;
                    admitted.push(WorkCursor {
                        work: nested,
                        next_receiver: 0,
                    });
                }
            }
            stack.extend(admitted.into_iter().rev());
            steps.push(step);
        }

        NeighborCollectorReport {
            steps,
            submitted,
            first_discarded,
        }
    }
}

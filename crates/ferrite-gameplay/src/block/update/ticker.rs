use ferrite_foundation::coordinate::BlockPos;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockEntityTicker {
    pub position: BlockPos,
    pub removed: bool,
    pub has_ticker: bool,
    pub normal_gameplay_gates_pass: bool,
    pub compatible_state: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TickerAction {
    Prune(BlockPos),
    Skip(BlockPos),
    Tick(BlockPos),
    Incompatible(BlockPos),
}

#[derive(Debug, Default)]
pub struct BlockEntityTickerList {
    active: Vec<BlockEntityTicker>,
    pending: Vec<BlockEntityTicker>,
    iterating: bool,
    cursor: usize,
}

impl BlockEntityTickerList {
    pub fn register_or_rebind(&mut self, ticker: BlockEntityTicker) {
        if let Some(existing) = self
            .active
            .iter_mut()
            .chain(self.pending.iter_mut())
            .find(|existing| existing.position == ticker.position)
        {
            *existing = ticker;
            return;
        }
        if self.iterating {
            self.pending.push(ticker);
        } else {
            self.active.push(ticker);
        }
    }

    pub fn begin_phase(&mut self) {
        assert!(!self.iterating, "block-entity phase is already active");
        self.active.append(&mut self.pending);
        self.iterating = true;
        self.cursor = 0;
    }

    pub fn next_action(&mut self, normal_gameplay: bool) -> Option<TickerAction> {
        assert!(self.iterating, "begin_phase must precede ticker iteration");
        if self.cursor < self.active.len() {
            let ticker = self.active[self.cursor];
            if ticker.removed || !ticker.has_ticker {
                self.active.remove(self.cursor);
                return Some(TickerAction::Prune(ticker.position));
            }
            self.cursor += 1;
            if !normal_gameplay || !ticker.normal_gameplay_gates_pass {
                return Some(TickerAction::Skip(ticker.position));
            }
            if !ticker.compatible_state {
                return Some(TickerAction::Incompatible(ticker.position));
            }
            Some(TickerAction::Tick(ticker.position))
        } else {
            None
        }
    }

    pub fn finish_phase(&mut self) {
        assert!(self.iterating, "begin_phase must precede finish_phase");
        self.iterating = false;
        self.cursor = 0;
    }

    pub fn active_positions(&self) -> impl ExactSizeIterator<Item = BlockPos> + '_ {
        self.active.iter().map(|ticker| ticker.position)
    }

    pub fn pending_len(&self) -> usize {
        self.pending.len()
    }
}

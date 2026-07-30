//! Ordered loot-table evaluation over registry-dispatched data nodes.

use crate::item::runtime::loot::context::LootContext;
use crate::item::runtime::loot::model::{
    ExpandedLootEntry, LootCondition, LootEntry, LootFunction, LootNumberProvider, LootOutput,
    LootPool, LootTable,
};
use crate::item::runtime::random::{GameplayRandom, GameplayRandomError, checked_int};
use crate::item::runtime::stack::ItemStack;
use ferrite_foundation::resource::ResourceId;
use std::collections::{BTreeMap, BTreeSet};

pub trait LootDispatch {
    fn test_condition(
        &mut self,
        condition: &LootCondition,
        context: &LootContext,
        random: &mut dyn GameplayRandom,
    ) -> Result<bool, LootEvaluationError>;

    fn number_int(
        &mut self,
        provider: &LootNumberProvider,
        context: &LootContext,
        random: &mut dyn GameplayRandom,
    ) -> Result<i32, LootEvaluationError>;

    fn number_float(
        &mut self,
        provider: &LootNumberProvider,
        context: &LootContext,
        random: &mut dyn GameplayRandom,
    ) -> Result<f32, LootEvaluationError>;

    fn expand_entry(
        &mut self,
        entry: &LootEntry,
        context: &LootContext,
        random: &mut dyn GameplayRandom,
        output: &mut Vec<ExpandedLootEntry>,
    ) -> Result<(), LootEvaluationError>;

    fn entry_weight(
        &mut self,
        entry: ExpandedLootEntry,
        luck: f32,
    ) -> Result<i32, LootEvaluationError>;

    fn create_outputs(
        &mut self,
        entry: ExpandedLootEntry,
        context: &LootContext,
        random: &mut dyn GameplayRandom,
        output: &mut Vec<LootOutput>,
    ) -> Result<(), LootEvaluationError>;

    fn apply_function(
        &mut self,
        function: &LootFunction,
        context: &LootContext,
        random: &mut dyn GameplayRandom,
        stack: ItemStack,
    ) -> Result<ItemStack, LootEvaluationError>;

    fn item_enabled(&self, stack: &ItemStack) -> bool;
}

pub struct LootEvaluator<'a, D> {
    tables: &'a BTreeMap<ResourceId, LootTable>,
    context: &'a LootContext,
    dispatch: &'a mut D,
    random: &'a mut dyn GameplayRandom,
    visited: BTreeSet<ResourceId>,
    warnings: Vec<LootWarning>,
}

impl<'a, D: LootDispatch> LootEvaluator<'a, D> {
    pub fn new(
        tables: &'a BTreeMap<ResourceId, LootTable>,
        context: &'a LootContext,
        dispatch: &'a mut D,
        random: &'a mut dyn GameplayRandom,
    ) -> Self {
        Self {
            tables,
            context,
            dispatch,
            random,
            visited: BTreeSet::new(),
            warnings: Vec::new(),
        }
    }

    pub fn evaluate(mut self, table: &ResourceId) -> Result<LootEvaluation, LootEvaluationError> {
        let raw = self.evaluate_raw(table)?;
        let stacks = split_normal_stacks(raw, |stack| self.dispatch.item_enabled(stack));
        Ok(LootEvaluation {
            stacks,
            warnings: self.warnings,
        })
    }

    pub fn evaluate_raw(
        &mut self,
        table_key: &ResourceId,
    ) -> Result<Vec<ItemStack>, LootEvaluationError> {
        let table = self
            .tables
            .get(table_key)
            .ok_or_else(|| LootEvaluationError::MissingTable(table_key.clone()))?
            .clone();
        if !self.visited.insert(table.key.clone()) {
            self.warnings
                .push(LootWarning::RecursiveTable(table.key.clone()));
            return Ok(Vec::new());
        }

        let result = self.evaluate_table_body(&table);
        self.visited.remove(&table.key);
        result
    }

    fn evaluate_table_body(
        &mut self,
        table: &LootTable,
    ) -> Result<Vec<ItemStack>, LootEvaluationError> {
        let mut stacks = Vec::new();
        for pool in &table.pools {
            let pool_stacks = self.evaluate_pool(pool)?;
            for stack in pool_stacks {
                stacks.push(self.apply_functions(stack, &table.functions)?);
            }
        }
        Ok(stacks)
    }

    fn evaluate_pool(&mut self, pool: &LootPool) -> Result<Vec<ItemStack>, LootEvaluationError> {
        for condition in &pool.conditions {
            if !self
                .dispatch
                .test_condition(condition, self.context, self.random)?
            {
                return Ok(Vec::new());
            }
        }
        let rolls = self
            .dispatch
            .number_int(&pool.rolls, self.context, self.random)?;
        let bonus = self
            .dispatch
            .number_float(&pool.bonus_rolls, self.context, self.random)?;
        if !bonus.is_finite() {
            return Err(LootEvaluationError::NonFiniteNumber);
        }
        let roll_count = rolls.wrapping_add((bonus * self.context.luck).floor() as i32);
        let mut result = Vec::new();
        for _ in 0..roll_count.max(0) {
            for output in self.select_outputs(pool)? {
                match output {
                    LootOutput::Stack(stack) => {
                        result.push(self.apply_functions(stack, &pool.functions)?);
                    }
                    LootOutput::Table(table) => {
                        for stack in self.evaluate_raw(&table)? {
                            result.push(self.apply_functions(stack, &pool.functions)?);
                        }
                    }
                    LootOutput::DynamicDrop(key) => {
                        if let Some(stacks) = self.context.dynamic_drops.get(&key) {
                            for stack in stacks.clone() {
                                result.push(self.apply_functions(stack, &pool.functions)?);
                            }
                        }
                    }
                }
            }
        }
        Ok(result)
    }

    fn select_outputs(&mut self, pool: &LootPool) -> Result<Vec<LootOutput>, LootEvaluationError> {
        let mut expanded = Vec::new();
        for entry in &pool.entries {
            self.dispatch
                .expand_entry(entry, self.context, self.random, &mut expanded)?;
        }
        let mut candidates = Vec::new();
        let mut total_weight = 0_i32;
        for entry in expanded {
            let weight = self.dispatch.entry_weight(entry, self.context.luck)?;
            if weight <= 0 {
                continue;
            }
            total_weight = total_weight
                .checked_add(weight)
                .ok_or(LootEvaluationError::WeightOverflow)?;
            candidates.push(entry);
        }
        if candidates.is_empty() || total_weight == 0 {
            return Ok(Vec::new());
        }
        let selected = if candidates.len() == 1 {
            candidates[0]
        } else {
            let mut draw = checked_int(self.random, total_weight as u32)? as i32;
            let mut selected = None;
            for candidate in candidates {
                draw -= self.dispatch.entry_weight(candidate, self.context.luck)?;
                if draw < 0 {
                    selected = Some(candidate);
                    break;
                }
            }
            selected.ok_or(LootEvaluationError::SelectionExhausted)?
        };
        let mut output = Vec::new();
        self.dispatch
            .create_outputs(selected, self.context, self.random, &mut output)?;
        Ok(output)
    }

    fn apply_functions(
        &mut self,
        mut stack: ItemStack,
        functions: &[LootFunction],
    ) -> Result<ItemStack, LootEvaluationError> {
        for function in functions {
            stack = self
                .dispatch
                .apply_function(function, self.context, self.random, stack)?;
        }
        Ok(stack)
    }
}

pub fn split_normal_stacks(
    raw: Vec<ItemStack>,
    enabled: impl Fn(&ItemStack) -> bool,
) -> Vec<ItemStack> {
    let mut result = Vec::new();
    for stack in raw {
        if stack.is_empty() || !enabled(&stack) {
            continue;
        }
        if stack.count < stack.maximum {
            result.push(stack);
            continue;
        }
        let mut remaining = stack.count;
        while remaining > 0 {
            let count = remaining.min(stack.maximum);
            let mut copy = stack.clone();
            copy.count = count;
            result.push(copy);
            remaining -= count;
        }
    }
    result
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsumeOnce {
    pub accepted: Vec<ItemStack>,
    pub refused: Vec<ItemStack>,
}

pub fn consume_generated_once(
    generated: Vec<ItemStack>,
    mut consumer: impl FnMut(&ItemStack) -> bool,
) -> ConsumeOnce {
    let mut accepted = Vec::new();
    let mut refused = Vec::new();
    for stack in generated {
        if consumer(&stack) {
            accepted.push(stack);
        } else {
            refused.push(stack);
        }
    }
    ConsumeOnce { accepted, refused }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LootEvaluation {
    pub stacks: Vec<ItemStack>,
    pub warnings: Vec<LootWarning>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LootWarning {
    RecursiveTable(ResourceId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LootEvaluationError {
    Random(GameplayRandomError),
    MissingTable(ResourceId),
    DispatchRejected(ResourceId),
    NonFiniteNumber,
    WeightOverflow,
    SelectionExhausted,
}

impl From<GameplayRandomError> for LootEvaluationError {
    fn from(value: GameplayRandomError) -> Self {
        Self::Random(value)
    }
}

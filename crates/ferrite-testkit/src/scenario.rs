//! Authored deterministic behavior scenarios.

use crate::seed::TestSeed;
use crate::snapshot::{Snapshot, SnapshotError, SnapshotMismatch};
use ferrite_foundation::resource::{ResourceId, ResourceIdError};
use serde::Deserialize;
use std::fs;
use std::path::Path;
use thiserror::Error;

pub const SCENARIO_SCHEMA_VERSION: u32 = 1;
pub const MAX_SCENARIO_STEPS: usize = 100_000;
pub const MAX_ACTION_BYTES: usize = 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scenario {
    id: ResourceId,
    seed: TestSeed,
    steps: Vec<ScenarioStep>,
}

impl Scenario {
    pub fn from_toml(source: &str) -> Result<Self, ScenarioError> {
        let definition: ScenarioDefinition = toml::from_str(source)?;
        Self::try_from(definition)
    }

    pub fn read(path: &Path) -> Result<Self, ScenarioError> {
        Self::from_toml(&fs::read_to_string(path)?)
    }

    pub const fn id(&self) -> &ResourceId {
        &self.id
    }

    pub const fn seed(&self) -> TestSeed {
        self.seed
    }

    pub fn steps(&self) -> &[ScenarioStep] {
        &self.steps
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioStep {
    tick: u64,
    action: ScenarioAction,
}

impl ScenarioStep {
    pub const fn tick(&self) -> u64 {
        self.tick
    }

    pub const fn action(&self) -> &ScenarioAction {
        &self.action
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScenarioAction {
    Apply { kind: ResourceId, payload: Vec<u8> },
    AssertSnapshot { expected: Snapshot },
}

pub trait ScenarioTarget {
    type Error: std::error::Error + Send + Sync + 'static;

    fn reset(&mut self, seed: TestSeed) -> Result<(), Self::Error>;
    fn advance_to(&mut self, tick: u64) -> Result<(), Self::Error>;
    fn apply(&mut self, kind: &ResourceId, payload: &[u8]) -> Result<(), Self::Error>;
    fn snapshot(&mut self) -> Result<Snapshot, Self::Error>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioReport {
    executed_steps: usize,
    final_tick: u64,
    final_snapshot: Snapshot,
}

impl ScenarioReport {
    pub const fn executed_steps(&self) -> usize {
        self.executed_steps
    }

    pub const fn final_tick(&self) -> u64 {
        self.final_tick
    }

    pub const fn final_snapshot(&self) -> &Snapshot {
        &self.final_snapshot
    }
}

pub fn run<T: ScenarioTarget>(
    scenario: &Scenario,
    target: &mut T,
) -> Result<ScenarioReport, ScenarioRunError> {
    target
        .reset(scenario.seed)
        .map_err(|error| ScenarioRunError::Target(error.to_string()))?;
    let mut final_tick = 0;
    for (index, step) in scenario.steps.iter().enumerate() {
        target
            .advance_to(step.tick)
            .map_err(|error| ScenarioRunError::Target(error.to_string()))?;
        final_tick = step.tick;
        match &step.action {
            ScenarioAction::Apply { kind, payload } => target
                .apply(kind, payload)
                .map_err(|error| ScenarioRunError::Target(error.to_string()))?,
            ScenarioAction::AssertSnapshot { expected } => {
                let actual = target
                    .snapshot()
                    .map_err(|error| ScenarioRunError::Target(error.to_string()))?;
                expected
                    .compare(&actual)
                    .map_err(|source| ScenarioRunError::Snapshot { index, source })?;
            }
        }
    }
    let final_snapshot = target
        .snapshot()
        .map_err(|error| ScenarioRunError::Target(error.to_string()))?;
    Ok(ScenarioReport {
        executed_steps: scenario.steps.len(),
        final_tick,
        final_snapshot,
    })
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ScenarioDefinition {
    schema_version: u32,
    id: String,
    seed: u64,
    step: Vec<StepDefinition>,
}

#[derive(Debug, Deserialize)]
struct StepDefinition {
    tick: u64,
    #[serde(flatten)]
    action: ActionDefinition,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
enum ActionDefinition {
    Apply { kind: String, payload: Vec<u8> },
    AssertSnapshot { expected: Vec<u8> },
}

impl TryFrom<ScenarioDefinition> for Scenario {
    type Error = ScenarioError;

    fn try_from(definition: ScenarioDefinition) -> Result<Self, Self::Error> {
        if definition.schema_version != SCENARIO_SCHEMA_VERSION {
            return Err(ScenarioError::UnsupportedSchema {
                actual: definition.schema_version,
                expected: SCENARIO_SCHEMA_VERSION,
            });
        }
        if definition.step.len() > MAX_SCENARIO_STEPS {
            return Err(ScenarioError::TooManySteps {
                actual: definition.step.len(),
                maximum: MAX_SCENARIO_STEPS,
            });
        }
        let mut previous_tick = 0;
        let steps = definition
            .step
            .into_iter()
            .enumerate()
            .map(|(index, step)| {
                if index != 0 && step.tick < previous_tick {
                    return Err(ScenarioError::TickWentBackwards {
                        index,
                        previous: previous_tick,
                        actual: step.tick,
                    });
                }
                previous_tick = step.tick;
                let action = match step.action {
                    ActionDefinition::Apply { kind, payload } => {
                        if payload.len() > MAX_ACTION_BYTES {
                            return Err(ScenarioError::ActionTooLarge {
                                index,
                                actual: payload.len(),
                                maximum: MAX_ACTION_BYTES,
                            });
                        }
                        ScenarioAction::Apply {
                            kind: kind.parse()?,
                            payload,
                        }
                    }
                    ActionDefinition::AssertSnapshot { expected } => {
                        ScenarioAction::AssertSnapshot {
                            expected: Snapshot::new(expected)?,
                        }
                    }
                };
                Ok(ScenarioStep {
                    tick: step.tick,
                    action,
                })
            })
            .collect::<Result<Vec<_>, ScenarioError>>()?;
        Ok(Self {
            id: definition.id.parse()?,
            seed: TestSeed::new(definition.seed),
            steps,
        })
    }
}

#[derive(Debug, Error)]
pub enum ScenarioError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Toml(#[from] toml::de::Error),
    #[error(transparent)]
    ResourceId(#[from] ResourceIdError),
    #[error(transparent)]
    Snapshot(#[from] SnapshotError),
    #[error("scenario schema version {actual} is unsupported; expected {expected}")]
    UnsupportedSchema { actual: u32, expected: u32 },
    #[error("scenario has {actual} steps, exceeding the {maximum}-step limit")]
    TooManySteps { actual: usize, maximum: usize },
    #[error("scenario step {index} moved backwards from tick {previous} to {actual}")]
    TickWentBackwards {
        index: usize,
        previous: u64,
        actual: u64,
    },
    #[error("scenario action {index} has {actual} bytes, exceeding the {maximum}-byte limit")]
    ActionTooLarge {
        index: usize,
        actual: usize,
        maximum: usize,
    },
}

#[derive(Debug, Error)]
pub enum ScenarioRunError {
    #[error("scenario target failed: {0}")]
    Target(String),
    #[error("snapshot assertion at step {index} failed: {source}")]
    Snapshot {
        index: usize,
        source: SnapshotMismatch,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::Infallible;

    #[derive(Default)]
    struct Target {
        tick: u64,
        bytes: Vec<u8>,
    }

    impl ScenarioTarget for Target {
        type Error = Infallible;

        fn reset(&mut self, seed: TestSeed) -> Result<(), Self::Error> {
            self.tick = 0;
            self.bytes = seed.get().to_le_bytes().to_vec();
            Ok(())
        }

        fn advance_to(&mut self, tick: u64) -> Result<(), Self::Error> {
            self.tick = tick;
            Ok(())
        }

        fn apply(&mut self, _: &ResourceId, payload: &[u8]) -> Result<(), Self::Error> {
            self.bytes.extend_from_slice(payload);
            Ok(())
        }

        fn snapshot(&mut self) -> Result<Snapshot, Self::Error> {
            Ok(Snapshot::new(self.bytes.clone()).unwrap())
        }
    }

    #[test]
    fn parses_and_runs_a_deterministic_scenario() {
        let source = r#"
schema_version = 1
id = "ferrite:test/basic"
seed = 42

[[step]]
tick = 2
action = "apply"
kind = "ferrite:test/append"
payload = [1, 2]

[[step]]
tick = 3
action = "assert_snapshot"
expected = [42, 0, 0, 0, 0, 0, 0, 0, 1, 2]
"#;
        let scenario = Scenario::from_toml(source).unwrap();
        let report = run(&scenario, &mut Target::default()).unwrap();
        assert_eq!(report.executed_steps(), 2);
        assert_eq!(report.final_tick(), 3);
        assert_eq!(report.final_snapshot().bytes().len(), 10);
    }

    #[test]
    fn rejects_unknown_fields_and_backwards_ticks() {
        let unknown = "schema_version=1\nid='ferrite:test/a'\nseed=1\nextra=true\nstep=[]";
        assert!(Scenario::from_toml(unknown).is_err());
        let backwards = r#"
schema_version = 1
id = "ferrite:test/a"
seed = 1
[[step]]
tick = 2
action = "apply"
kind = "ferrite:test/a"
payload = []
[[step]]
tick = 1
action = "apply"
kind = "ferrite:test/a"
payload = []
"#;
        assert!(matches!(
            Scenario::from_toml(backwards),
            Err(ScenarioError::TickWentBackwards { .. })
        ));
    }
}

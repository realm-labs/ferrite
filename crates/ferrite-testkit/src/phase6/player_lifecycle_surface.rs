//! Executable PlayerLifecycle root-surface conformance.

use ferrite_gameplay::player::lifecycle::admission::{
    AdmissionGate, AdmissionRejection, AdmissionRequest, admit,
};
use ferrite_gameplay::player::lifecycle::model::{
    GameMode, LifecycleEffect, PermissionLevel, RespawnOutcome, RespawnRequest,
};
use ferrite_gameplay::player::lifecycle::runtime::PlayerLifecycle;
use ferrite_replay::envelope::{
    CommandEnvelope, CommandSource, EnvelopePayload, SequenceNumber, TickNumber,
};
use ferrite_replay::hash::{RegionHashRecord, StateHash};
use ferrite_replay::log::{ReplayFrame, ReplayHeader, ReplayLog};
use ferrite_replay::verify::{ObservedFrame, ReplayTarget, VerificationReport, verify_replay};
use ferrite_simulation::random::{DeterministicRng, RandomAlgorithm};
use std::num::NonZeroU64;

use crate::phase6::fixtures::{player, region};

const PROPERTY_CASES: usize = 128;
const FUZZ_CASES: usize = 256;
const REPLAY_FRAMES: usize = 6;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerLifecycleSurfaceReport {
    pub golden_digest: String,
    pub property_cases: usize,
    pub fuzz_cases: usize,
    pub fault_cases: usize,
    pub replay_frames: usize,
    pub client_trace_events: usize,
}

pub fn run_player_lifecycle_surface() -> PlayerLifecycleSurfaceReport {
    let trace = golden_trace();
    run_admission_properties();
    run_operation_fuzz();
    run_fault_vectors();
    run_replay_vectors();
    PlayerLifecycleSurfaceReport {
        golden_digest: digest(&trace),
        property_cases: PROPERTY_CASES,
        fuzz_cases: FUZZ_CASES,
        fault_cases: 8,
        replay_frames: REPLAY_FRAMES,
        client_trace_events: trace.len(),
    }
}

fn golden_trace() -> Vec<LifecycleEffect> {
    let id = player(1);
    let mut lifecycle = PlayerLifecycle::new(4).expect("fixture capacity is valid");
    let mut trace = lifecycle
        .join(id, 1, false)
        .expect("fixture join is admitted");
    trace.extend(
        lifecycle
            .set_game_mode(id, GameMode::Spectator)
            .expect("fixture player remains live"),
    );
    trace.extend(
        lifecycle
            .set_game_mode(id, GameMode::Survival)
            .expect("fixture player remains live"),
    );
    for permission in [
        PermissionLevel::All,
        PermissionLevel::Moderators,
        PermissionLevel::GameMasters,
        PermissionLevel::Admins,
        PermissionLevel::Owners,
    ] {
        trace.extend(
            lifecycle
                .set_permission(id, permission)
                .expect("fixture player remains live"),
        );
    }
    trace.extend(
        lifecycle
            .teleport(id, true)
            .expect("fixture player remains live"),
    );
    trace.extend(lifecycle.die(id).expect("fixture player remains live"));
    let RespawnOutcome::Replaced { effects, .. } = lifecycle
        .respawn(
            id,
            RespawnRequest {
                keep_inventory: false,
                hardcore: false,
            },
        )
        .expect("fixture respawn is valid")
    else {
        panic!("dead fixture player must be replaced");
    };
    trace.extend(effects);
    lifecycle
        .mark_won_game(id)
        .expect("fixture player remains live");
    let RespawnOutcome::Replaced {
        keep_all: true,
        effects,
    } = lifecycle
        .respawn(
            id,
            RespawnRequest {
                keep_inventory: false,
                hardcore: true,
            },
        )
        .expect("won-game replacement is valid")
    else {
        panic!("won-game fixture player must keep all state");
    };
    trace.extend(effects);
    trace.extend(
        lifecycle
            .disconnect(id)
            .expect("fixture disconnect is valid"),
    );
    trace
}

fn run_admission_properties() {
    for case in 0..PROPERTY_CASES {
        let request = AdmissionRequest {
            user_banned: case & 1 != 0,
            whitelist_enabled: case & 2 != 0,
            whitelisted: case & 4 != 0,
            ip_banned: case & 8 != 0,
            current_players: usize::from(case & 16 != 0),
            capacity: 1,
            bypass_capacity: case & 32 != 0,
        };
        let first = admit(request);
        let second = admit(request);
        assert_eq!(first, second);
        let expected = if request.user_banned {
            Some(AdmissionRejection::UserBanned)
        } else if request.whitelist_enabled && !request.whitelisted {
            Some(AdmissionRejection::NotWhitelisted)
        } else if request.ip_banned {
            Some(AdmissionRejection::IpBanned)
        } else if request.current_players >= request.capacity && !request.bypass_capacity {
            Some(AdmissionRejection::ServerFull)
        } else {
            None
        };
        assert_eq!(first.rejection, expected);
        assert_eq!(first.checked.first(), Some(&AdmissionGate::UserBan));
        if first.checked.contains(&AdmissionGate::Capacity) {
            assert_eq!(
                first.checked,
                [
                    AdmissionGate::UserBan,
                    AdmissionGate::Whitelist,
                    AdmissionGate::IpBan,
                    AdmissionGate::Capacity,
                ]
            );
        }
    }
}

fn run_operation_fuzz() {
    let mut random = DeterministicRng::from_seed(0x706c_6179_6572);
    for case in 0..FUZZ_CASES {
        let id = player(case as u128 + 1);
        let transferred = random.uniform_u64(NonZeroU64::new(2).unwrap()) != 0;
        let mut first = PlayerLifecycle::new(1).unwrap();
        let mut second = PlayerLifecycle::new(1).unwrap();
        assert_eq!(
            first.join(id, 1, transferred).unwrap(),
            second.join(id, 1, transferred).unwrap()
        );
        let mode = match random.uniform_u64(NonZeroU64::new(4).unwrap()) {
            0 => GameMode::Survival,
            1 => GameMode::Creative,
            2 => GameMode::Adventure,
            _ => GameMode::Spectator,
        };
        assert_eq!(
            first.set_game_mode(id, mode).unwrap(),
            second.set_game_mode(id, mode).unwrap()
        );
        if random.uniform_u64(NonZeroU64::new(2).unwrap()) == 0 {
            assert_eq!(first.die(id).unwrap(), second.die(id).unwrap());
        } else {
            first.mark_won_game(id).unwrap();
            second.mark_won_game(id).unwrap();
        }
        let request = RespawnRequest {
            keep_inventory: random.uniform_u64(NonZeroU64::new(2).unwrap()) != 0,
            hardcore: random.uniform_u64(NonZeroU64::new(2).unwrap()) != 0,
        };
        assert_eq!(
            first.respawn(id, request).unwrap(),
            second.respawn(id, request).unwrap()
        );
        assert_eq!(first.snapshot(), second.snapshot());
    }
}

fn run_fault_vectors() {
    let id = player(1);
    assert!(PlayerLifecycle::new(0).is_err());
    let mut lifecycle = PlayerLifecycle::new(1).unwrap();
    lifecycle.join(id, 1, false).unwrap();
    assert!(lifecycle.join(id, 2, false).is_err());
    assert!(lifecycle.join(player(2), 1, false).is_err());
    assert!(lifecycle.die(player(3)).is_err());
    assert!(lifecycle.disconnect(player(3)).is_err());
    lifecycle.disconnect(id).unwrap();
    assert!(lifecycle.disconnect(id).is_err());
    assert!(lifecycle.set_game_mode(id, GameMode::Creative).is_err());
    assert!(
        lifecycle
            .set_permission(id, PermissionLevel::Owners)
            .is_err()
    );
}

fn run_replay_vectors() {
    let key = region();
    let frames = (1..=REPLAY_FRAMES)
        .map(|tick| {
            let operation = tick as u8;
            let command = CommandEnvelope::new(
                TickNumber::new(tick as u64),
                SequenceNumber::new(1),
                CommandSource::System,
                key.clone(),
                ferrite_foundation::resource::ResourceId::new("ferrite", "phase6/player-lifecycle")
                    .unwrap(),
                EnvelopePayload::new(vec![operation]).unwrap(),
            );
            let hash = replay_hash(operation, false);
            ReplayFrame::new(
                TickNumber::new(tick as u64),
                vec![command],
                Vec::new(),
                vec![RegionHashRecord::new(key.clone(), hash)],
                hash,
            )
            .unwrap()
        })
        .collect();
    let log = ReplayLog::new(
        ReplayHeader::new(
            ferrite_foundation::resource::ResourceId::new(
                "ferrite",
                "phase6-lifecycle-conformance",
            )
            .unwrap(),
            key.world(),
            StateHash::from_bytes([0x61; 32]),
            key.mapping_version(),
            RandomAlgorithm::Xoshiro256StarStarV1,
            TickNumber::new(0),
        ),
        frames,
    )
    .unwrap();
    assert!(
        verify_replay(
            &log,
            &mut LifecycleReplayTarget {
                region: key.clone(),
                perturb: false,
            },
        )
        .is_converged()
    );
    assert!(matches!(
        verify_replay(
            &log,
            &mut LifecycleReplayTarget {
                region: key,
                perturb: true,
            },
        ),
        VerificationReport::Diverged(_)
    ));
}

struct LifecycleReplayTarget {
    region: ferrite_foundation::region::SimulationRegionKey,
    perturb: bool,
}

impl ReplayTarget for LifecycleReplayTarget {
    type Error = String;

    fn begin(&mut self, _header: &ReplayHeader) -> Result<(), Self::Error> {
        Ok(())
    }

    fn execute(
        &mut self,
        tick: TickNumber,
        commands: &[CommandEnvelope],
    ) -> Result<ObservedFrame, Self::Error> {
        let operation = commands
            .first()
            .and_then(|command| command.payload().as_slice().first())
            .copied()
            .ok_or_else(|| "lifecycle replay operation is missing".to_owned())?;
        let hash = replay_hash(operation, self.perturb);
        ObservedFrame::new(
            tick,
            Vec::new(),
            vec![RegionHashRecord::new(self.region.clone(), hash)],
            hash,
        )
        .map_err(|error| error.to_string())
    }
}

fn replay_hash(operation: u8, perturb: bool) -> StateHash {
    let mut bytes = vec![operation];
    if perturb {
        bytes.push(1);
    }
    StateHash::from_bytes(*blake3::hash(&bytes).as_bytes())
}

fn digest(trace: &[LifecycleEffect]) -> String {
    let bytes = trace
        .iter()
        .flat_map(|effect| format!("{effect:?}\n").into_bytes())
        .collect::<Vec<_>>();
    blake3::hash(&bytes)
        .as_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

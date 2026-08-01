use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::identity::StableEntityId;
use ferrite_foundation::resource::ResourceId;
use ferrite_protocol::java_26_2::play::clientbound::packet::{BlockUpdate, PlayClientboundPacket};
use ferrite_server_runtime::chunk::projection::JavaTerrainRegistryMap;
use ferrite_server_runtime::composite::model::{CompositeOwner, CompositeProjection};
use ferrite_server_runtime::composite::projection::{
    DeferredProjectionKind, ProjectionAudience, SessionProjectionAction, SessionProjectionError,
    SessionProjectionQueue, decode_projection,
};
use ferrite_world::id::BlockStateId;

fn player(value: u128) -> StableEntityId {
    StableEntityId::new(value).unwrap()
}

fn player_projection(target: StableEntityId, sequence: u64) -> CompositeProjection {
    let mut payload = Vec::new();
    payload.extend_from_slice(&target.to_be_bytes());
    payload.extend_from_slice(&1_u64.to_be_bytes());
    payload.push(2);
    payload.push(0);
    payload.extend_from_slice(&0_u64.to_be_bytes());
    payload.push(0);
    CompositeProjection::new(
        CompositeOwner::PlayerService,
        sequence,
        ResourceId::new("ferrite", "composite/player/projection_v1").unwrap(),
        payload,
    )
}

fn block_projection(sequence: u64) -> CompositeProjection {
    let mut payload = Vec::new();
    for coordinate in [1_i32, 65, -2] {
        payload.extend_from_slice(&coordinate.to_be_bytes());
    }
    payload.extend_from_slice(&7_u32.to_be_bytes());
    CompositeProjection::new(
        CompositeOwner::Simulation,
        sequence,
        ResourceId::new("ferrite", "composite/simulation/block_update_v1").unwrap(),
        payload,
    )
}

fn registries() -> JavaTerrainRegistryMap {
    let mut registries = JavaTerrainRegistryMap::new(8, BlockStateId::new(0)).unwrap();
    registries
        .insert_block_state(BlockStateId::new(7), 70)
        .unwrap();
    registries
}

#[test]
fn committed_projection_decode_preserves_audience_and_semantics() {
    let owner = player(1);
    let targeted = decode_projection(&player_projection(owner, 1)).unwrap();
    assert_eq!(targeted.audience(), ProjectionAudience::Player(owner));
    assert_eq!(
        targeted.action(),
        &SessionProjectionAction::Deferred(DeferredProjectionKind::PlayerService)
    );

    let block = decode_projection(&block_projection(2)).unwrap();
    assert_eq!(block.audience(), ProjectionAudience::AllPlayers);
    assert_eq!(
        block.action(),
        &SessionProjectionAction::Block(
            ferrite_server_runtime::player::block::replication::AuthoritativeBlockUpdate {
                position: BlockPos::new(1, 65, -2),
                state: BlockStateId::new(7),
            }
        )
    );
    assert_eq!(
        block.scoped_to_region(region()).audience(),
        ProjectionAudience::RegionPlayers(region())
    );
}

#[test]
fn per_session_admission_is_atomic_and_projection_is_post_commit_bounded() {
    let owner = player(2);
    let projections = [
        decode_projection(&player_projection(owner, 1)).unwrap(),
        decode_projection(&block_projection(2)).unwrap(),
    ];
    let mut too_small = SessionProjectionQueue::new(1).unwrap();
    assert!(matches!(
        too_small.admit(owner, &region(), &projections),
        Err(SessionProjectionError::Full { capacity: 1 })
    ));
    assert!(too_small.is_empty());

    let mut queue = SessionProjectionQueue::new(2).unwrap();
    assert_eq!(queue.admit(owner, &region(), &projections).unwrap(), 2);
    let first = queue.project(1, &registries()).unwrap();
    assert!(first.packets.is_empty());
    assert_eq!(first.deferred.len(), 1);
    assert_eq!(queue.len(), 1);

    let second = queue.project(1, &registries()).unwrap();
    assert_eq!(
        second.packets,
        [PlayClientboundPacket::BlockUpdate(BlockUpdate {
            position: BlockPos::new(1, 65, -2),
            state: 70,
        })]
    );
    assert!(queue.is_empty());
}

fn region() -> ferrite_foundation::region::SimulationRegionKey {
    use ferrite_foundation::identity::{DimensionId, WorldId};
    use ferrite_foundation::region::{RegionCoord, RegionMappingVersion, SimulationRegionKey};

    SimulationRegionKey::new(
        WorldId::new(1).unwrap(),
        DimensionId::new(ResourceId::minecraft("overworld").unwrap()),
        RegionCoord::new(0, 0),
        RegionMappingVersion::V1,
    )
}

#[test]
fn projection_decode_fails_closed_on_unknown_or_malformed_records() {
    let malformed = CompositeProjection::new(
        CompositeOwner::Simulation,
        1,
        ResourceId::new("ferrite", "composite/simulation/block_update_v1").unwrap(),
        vec![0; 15],
    );
    assert!(matches!(
        decode_projection(&malformed),
        Err(SessionProjectionError::MalformedBlock)
    ));

    let unknown = CompositeProjection::new(
        CompositeOwner::Ingress,
        1,
        ResourceId::new("ferrite", "composite/unknown_v1").unwrap(),
        Vec::new(),
    );
    assert!(matches!(
        decode_projection(&unknown),
        Err(SessionProjectionError::UnsupportedKind(_))
    ));
}

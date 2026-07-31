//! Cross-crate block-prediction fixture for the behavior runner.

use ferrite_foundation::coordinate::BlockPos;
use ferrite_protocol::java_26_2::play::clientbound::block::{
    BlockClientProjection, BlockProjectionAction,
};
use ferrite_protocol::java_26_2::play::clientbound::packet::{
    BlockChangedAck, BlockUpdate, PlayClientboundPacket,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SamePositionPredictionReport {
    pub predicted_before_covering_ack: i32,
    pub state_after_covering_ack: i32,
    pub pending_after_old_ack: usize,
    pub resolved_by_covering_ack: usize,
    pub captured_authoritative_state: i32,
}

pub fn run_same_position_prediction() -> SamePositionPredictionReport {
    let position = BlockPos::new(1, 64, 2);
    let mut client = BlockClientProjection::new(16).expect("bounded projection");
    client
        .install_block(position, 10, 1)
        .expect("initial state");
    client
        .retain_prediction(position, 1, 11, [1.5, 64.0, 2.5])
        .expect("first prediction");
    client
        .retain_prediction(position, 2, 12, [1.5, 64.0, 2.5])
        .expect("newer prediction replaces only the position sequence");
    client
        .apply(
            &PlayClientboundPacket::BlockUpdate(BlockUpdate {
                position,
                state: 13,
            }),
            0,
        )
        .expect("server update stages behind prediction");
    let predicted_before_covering_ack = client
        .block_state(position)
        .expect("predicted state remains installed");
    let old_ack = client
        .apply(
            &PlayClientboundPacket::BlockChangedAck(BlockChangedAck { sequence: 1 }),
            0,
        )
        .expect("older cumulative acknowledgement");
    let BlockProjectionAction::PredictionsResolved(old_values) = old_ack else {
        panic!("acknowledgement must use the prediction-resolution route");
    };
    assert!(old_values.is_empty());
    let pending_after_old_ack = client.prediction_count();

    let covering_ack = client
        .apply(
            &PlayClientboundPacket::BlockChangedAck(BlockChangedAck { sequence: 2 }),
            0,
        )
        .expect("covering acknowledgement resolves");
    let BlockProjectionAction::PredictionsResolved(values) = covering_ack else {
        panic!("covering acknowledgement must resolve predictions");
    };
    SamePositionPredictionReport {
        predicted_before_covering_ack,
        state_after_covering_ack: client
            .block_state(position)
            .expect("authoritative state is restored"),
        pending_after_old_ack,
        resolved_by_covering_ack: values.len(),
        captured_authoritative_state: values
            .first()
            .expect("one same-position prediction resolves")
            .state,
    }
}

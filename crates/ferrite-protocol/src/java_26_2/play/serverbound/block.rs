//! Ordering boundary for the five required serverbound block-input packets.

use crate::java_26_2::play::serverbound::packet::{
    PickItemFromBlock, PlayServerboundEntryPacket, PlayerAction, Swing, UseItem, UseItemOn,
};
use crate::java_26_2::play::serverbound::session::{BlockSequenceError, PlayServerSession};

pub trait BlockSequenceRegistrar {
    type Error;

    fn register_block_sequence(&mut self, sequence: i32) -> Result<(), Self::Error>;
}

impl BlockSequenceRegistrar for PlayServerSession {
    type Error = BlockSequenceError;

    fn register_block_sequence(&mut self, sequence: i32) -> Result<(), Self::Error> {
        PlayServerSession::register_block_sequence(self, sequence)
    }
}

pub trait ServerboundBlockHandler {
    type Error;

    fn pick_item_from_block(&mut self, packet: PickItemFromBlock) -> Result<(), Self::Error>;
    fn destroy(&mut self, packet: PlayerAction) -> Result<(), Self::Error>;
    fn auxiliary_action(&mut self, packet: PlayerAction) -> Result<(), Self::Error>;
    fn swing(&mut self, packet: Swing) -> Result<(), Self::Error>;
    fn use_item_on(&mut self, packet: UseItemOn) -> Result<(), Self::Error>;
    fn use_item(&mut self, packet: UseItem) -> Result<(), Self::Error>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockDispatchOutcome {
    Handled,
    DroppedBeforeClientLoaded,
    IgnoredOtherFamily,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockDispatchError<HandlerError, SequenceError> {
    Handler(HandlerError),
    Sequence(SequenceError),
}

pub fn dispatch_block_packet<H, S>(
    packet: PlayServerboundEntryPacket,
    client_loaded: bool,
    handler: &mut H,
    sequences: &mut S,
) -> Result<BlockDispatchOutcome, BlockDispatchError<H::Error, S::Error>>
where
    H: ServerboundBlockHandler,
    S: BlockSequenceRegistrar,
{
    match packet {
        PlayServerboundEntryPacket::PickItemFromBlock(packet) => {
            handler
                .pick_item_from_block(packet)
                .map_err(BlockDispatchError::Handler)?;
        }
        PlayServerboundEntryPacket::PlayerAction(packet) if packet.action.is_destroy() => {
            if !client_loaded {
                return Ok(BlockDispatchOutcome::DroppedBeforeClientLoaded);
            }
            handler
                .destroy(packet)
                .map_err(BlockDispatchError::Handler)?;
            sequences
                .register_block_sequence(packet.sequence)
                .map_err(BlockDispatchError::Sequence)?;
        }
        PlayServerboundEntryPacket::PlayerAction(packet) => {
            handler
                .auxiliary_action(packet)
                .map_err(BlockDispatchError::Handler)?;
        }
        PlayServerboundEntryPacket::Swing(packet) => {
            handler.swing(packet).map_err(BlockDispatchError::Handler)?;
        }
        PlayServerboundEntryPacket::UseItemOn(packet) => {
            if !client_loaded {
                return Ok(BlockDispatchOutcome::DroppedBeforeClientLoaded);
            }
            sequences
                .register_block_sequence(packet.sequence)
                .map_err(BlockDispatchError::Sequence)?;
            handler
                .use_item_on(packet)
                .map_err(BlockDispatchError::Handler)?;
        }
        PlayServerboundEntryPacket::UseItem(packet) => {
            if !client_loaded {
                return Ok(BlockDispatchOutcome::DroppedBeforeClientLoaded);
            }
            sequences
                .register_block_sequence(packet.sequence)
                .map_err(BlockDispatchError::Sequence)?;
            handler
                .use_item(packet)
                .map_err(BlockDispatchError::Handler)?;
        }
        _ => return Ok(BlockDispatchOutcome::IgnoredOtherFamily),
    }
    Ok(BlockDispatchOutcome::Handled)
}

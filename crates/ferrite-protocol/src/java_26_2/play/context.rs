use thiserror::Error;

use crate::java_26_2::play::registry::PlayRegistries;
use crate::java_26_2::value::identifier::Identifier;
use crate::java_26_2::wire::primitive::WireReader;

/// Version-locked component payload dispatch supplied by the content adapter.
///
/// Data-component values are not self-delimiting. The selected component identity decides exactly
/// how many bytes this callback consumes. Ferrite keeps that generated dispatch outside the packet
/// family so raw component IDs and payload structs cannot leak into simulation.
pub trait ComponentValueDecoder {
    fn decode_value(
        &self,
        component: &Identifier,
        reader: &mut WireReader<'_>,
    ) -> Result<Vec<u8>, ComponentValueError>;
}

#[derive(Debug, Clone, Copy)]
pub struct RejectComponentValues;

impl ComponentValueDecoder for RejectComponentValues {
    fn decode_value(
        &self,
        component: &Identifier,
        _reader: &mut WireReader<'_>,
    ) -> Result<Vec<u8>, ComponentValueError> {
        Err(ComponentValueError::Unsupported {
            component: component.clone(),
        })
    }
}

#[derive(Clone, Copy)]
pub struct PlayDecodeContext<'a> {
    pub registries: &'a PlayRegistries,
    pub component_values: &'a dyn ComponentValueDecoder,
    pub dimension_section_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ComponentValueError {
    #[error("no stream codec is installed for data component {component}")]
    Unsupported { component: Identifier },
    #[error("data component {component} payload is malformed: {reason}")]
    Malformed {
        component: Identifier,
        reason: String,
    },
}

use std::collections::BTreeSet;

use thiserror::Error;

use crate::java_26_2::play::registry::{COMMAND_ARGUMENT_TYPE, PlayRegistries, PlayRegistryError};
use crate::java_26_2::value::identifier::{Identifier, IdentifierError, IdentifierReadError};
use crate::java_26_2::wire::compression::MAX_INFLATED_PACKET_LENGTH;
use crate::java_26_2::wire::error::WireError;
use crate::java_26_2::wire::primitive::{WireReader, WireWriter};

const MAX_UTF_CODE_UNITS: usize = 32_767;

#[derive(Debug, Clone, PartialEq)]
pub struct CommandTree {
    pub nodes: Vec<CommandNode>,
    pub root_index: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CommandNode {
    pub executable: bool,
    pub restricted: bool,
    pub children: Vec<i32>,
    pub redirect: Option<i32>,
    pub kind: CommandNodeKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CommandNodeKind {
    Root,
    Literal {
        name: String,
    },
    Argument {
        name: String,
        argument_type: Identifier,
        payload: CommandArgumentPayload,
        suggestion_provider: Option<Identifier>,
    },
    Placeholder,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CommandArgumentPayload {
    None,
    Float(NumericBounds<f32>),
    Double(NumericBounds<f64>),
    Integer(NumericBounds<i32>),
    Long(NumericBounds<i64>),
    String(StringArgumentKind),
    Flags(u8),
    TimeMinimum(i32),
    Registry(Identifier),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NumericBounds<T> {
    pub minimum: Option<T>,
    pub maximum: Option<T>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StringArgumentKind {
    SingleWord,
    QuotablePhrase,
    GreedyPhrase,
}

impl StringArgumentKind {
    const fn from_id(id: i32) -> Option<Self> {
        match id {
            0 => Some(Self::SingleWord),
            1 => Some(Self::QuotablePhrase),
            2 => Some(Self::GreedyPhrase),
            _ => None,
        }
    }

    const fn id(self) -> i32 {
        match self {
            Self::SingleWord => 0,
            Self::QuotablePhrase => 1,
            Self::GreedyPhrase => 2,
        }
    }
}

pub(crate) fn read(
    reader: &mut WireReader<'_>,
    registries: &PlayRegistries,
) -> Result<CommandTree, CommandTreeError> {
    let count = reader.read_count("command nodes", reader.remaining())?;
    let mut nodes = Vec::with_capacity(count);
    for _ in 0..count {
        nodes.push(read_node(reader, registries)?);
    }
    let tree = CommandTree {
        nodes,
        root_index: reader.read_var_i32()?,
    };
    tree.validate_reachable()?;
    Ok(tree)
}

pub(crate) fn write(
    writer: &mut WireWriter,
    tree: &CommandTree,
    registries: &PlayRegistries,
) -> Result<(), CommandTreeError> {
    tree.validate_reachable()?;
    writer.write_count(
        "command nodes",
        tree.nodes.len(),
        MAX_INFLATED_PACKET_LENGTH,
    )?;
    for node in &tree.nodes {
        write_node(writer, node, registries)?;
    }
    writer.write_var_i32(tree.root_index)?;
    Ok(())
}

impl CommandTree {
    pub fn validate_reachable(&self) -> Result<(), CommandTreeError> {
        let root = index(self.root_index, self.nodes.len(), "root")?;
        if matches!(self.nodes[root].kind, CommandNodeKind::Placeholder) {
            return Err(CommandTreeError::PlaceholderRoot {
                index: self.root_index,
            });
        }
        let mut visiting = BTreeSet::new();
        let mut visited = BTreeSet::new();
        self.visit(root, &mut visiting, &mut visited)
    }

    fn visit(
        &self,
        current: usize,
        visiting: &mut BTreeSet<usize>,
        visited: &mut BTreeSet<usize>,
    ) -> Result<(), CommandTreeError> {
        if visited.contains(&current) {
            return Ok(());
        }
        if !visiting.insert(current) {
            return Err(CommandTreeError::Cycle { index: current });
        }
        let node = &self.nodes[current];
        for child in &node.children {
            let child_index = index(*child, self.nodes.len(), "child")?;
            if !matches!(self.nodes[child_index].kind, CommandNodeKind::Placeholder) {
                self.visit(child_index, visiting, visited)?;
            }
        }
        if let Some(redirect) = node.redirect {
            let redirect_index = index(redirect, self.nodes.len(), "redirect")?;
            if !matches!(
                self.nodes[redirect_index].kind,
                CommandNodeKind::Placeholder
            ) {
                self.visit(redirect_index, visiting, visited)?;
            }
        }
        visiting.remove(&current);
        visited.insert(current);
        Ok(())
    }
}

fn read_node(
    reader: &mut WireReader<'_>,
    registries: &PlayRegistries,
) -> Result<CommandNode, CommandTreeError> {
    let flags = reader.read_u8()?;
    let child_count = reader.read_count("command children", reader.remaining())?;
    let mut children = Vec::with_capacity(child_count);
    for _ in 0..child_count {
        children.push(reader.read_var_i32()?);
    }
    let redirect = if flags & 0x08 != 0 {
        Some(reader.read_var_i32()?)
    } else {
        None
    };
    let kind = match flags & 0x03 {
        0 => CommandNodeKind::Root,
        1 => CommandNodeKind::Literal {
            name: reader.read_utf(MAX_UTF_CODE_UNITS)?.into_owned(),
        },
        2 => read_argument_node(reader, flags, registries)?,
        _ => CommandNodeKind::Placeholder,
    };
    Ok(CommandNode {
        executable: flags & 0x04 != 0,
        restricted: flags & 0x20 != 0,
        children,
        redirect,
        kind,
    })
}

fn read_argument_node(
    reader: &mut WireReader<'_>,
    flags: u8,
    registries: &PlayRegistries,
) -> Result<CommandNodeKind, CommandTreeError> {
    let name = reader.read_utf(MAX_UTF_CODE_UNITS)?.into_owned();
    let raw_id = reader.read_var_i32()?;
    let Ok(argument_type) = registries.resolve(COMMAND_ARGUMENT_TYPE, raw_id) else {
        if flags & 0x10 != 0 {
            read_identifier(reader)?;
        }
        return Ok(CommandNodeKind::Placeholder);
    };
    let payload = read_argument_payload(reader, &argument_type)?;
    let suggestion_provider = if flags & 0x10 != 0 {
        Some(read_identifier(reader)?)
    } else {
        None
    };
    Ok(CommandNodeKind::Argument {
        name,
        argument_type,
        payload,
        suggestion_provider,
    })
}

fn read_argument_payload(
    reader: &mut WireReader<'_>,
    argument_type: &Identifier,
) -> Result<CommandArgumentPayload, CommandTreeError> {
    let identity = argument_type.to_string();
    match identity.as_str() {
        "brigadier:float" => Ok(CommandArgumentPayload::Float(read_f32_bounds(reader)?)),
        "brigadier:double" => Ok(CommandArgumentPayload::Double(read_f64_bounds(reader)?)),
        "brigadier:integer" => Ok(CommandArgumentPayload::Integer(read_i32_bounds(reader)?)),
        "brigadier:long" => Ok(CommandArgumentPayload::Long(read_i64_bounds(reader)?)),
        "brigadier:string" => {
            let id = reader.read_var_i32()?;
            let kind = StringArgumentKind::from_id(id)
                .ok_or(CommandTreeError::InvalidStringArgumentKind { id })?;
            Ok(CommandArgumentPayload::String(kind))
        }
        "minecraft:entity" | "minecraft:score_holder" => {
            Ok(CommandArgumentPayload::Flags(reader.read_u8()?))
        }
        "minecraft:time" => Ok(CommandArgumentPayload::TimeMinimum(reader.read_i32()?)),
        "minecraft:resource_or_tag"
        | "minecraft:resource_or_tag_key"
        | "minecraft:resource"
        | "minecraft:resource_key"
        | "minecraft:resource_selector" => {
            Ok(CommandArgumentPayload::Registry(read_identifier(reader)?))
        }
        _ => Ok(CommandArgumentPayload::None),
    }
}

fn write_node(
    writer: &mut WireWriter,
    node: &CommandNode,
    registries: &PlayRegistries,
) -> Result<(), CommandTreeError> {
    let mut flags = match node.kind {
        CommandNodeKind::Root => 0,
        CommandNodeKind::Literal { .. } => 1,
        CommandNodeKind::Argument { .. } => 2,
        CommandNodeKind::Placeholder => 3,
    };
    if node.executable {
        flags |= 0x04;
    }
    if node.redirect.is_some() {
        flags |= 0x08;
    }
    if matches!(
        node.kind,
        CommandNodeKind::Argument {
            suggestion_provider: Some(_),
            ..
        }
    ) {
        flags |= 0x10;
    }
    if node.restricted {
        flags |= 0x20;
    }
    writer.write_u8(flags)?;
    writer.write_count(
        "command children",
        node.children.len(),
        MAX_INFLATED_PACKET_LENGTH,
    )?;
    for child in &node.children {
        writer.write_var_i32(*child)?;
    }
    if let Some(redirect) = node.redirect {
        writer.write_var_i32(redirect)?;
    }
    match &node.kind {
        CommandNodeKind::Root | CommandNodeKind::Placeholder => {}
        CommandNodeKind::Literal { name } => {
            writer.write_utf(name, MAX_UTF_CODE_UNITS)?;
        }
        CommandNodeKind::Argument {
            name,
            argument_type,
            payload,
            suggestion_provider,
        } => {
            writer.write_utf(name, MAX_UTF_CODE_UNITS)?;
            writer.write_var_i32(registries.raw_id(COMMAND_ARGUMENT_TYPE, argument_type)?)?;
            write_argument_payload(writer, argument_type, payload)?;
            if let Some(provider) = suggestion_provider {
                provider.write(writer)?;
            }
        }
    }
    Ok(())
}

fn write_argument_payload(
    writer: &mut WireWriter,
    argument_type: &Identifier,
    payload: &CommandArgumentPayload,
) -> Result<(), CommandTreeError> {
    let identity = argument_type.to_string();
    match (identity.as_str(), payload) {
        ("brigadier:float", CommandArgumentPayload::Float(bounds)) => {
            write_f32_bounds(writer, *bounds)?;
        }
        ("brigadier:double", CommandArgumentPayload::Double(bounds)) => {
            write_f64_bounds(writer, *bounds)?;
        }
        ("brigadier:integer", CommandArgumentPayload::Integer(bounds)) => {
            write_i32_bounds(writer, *bounds)?;
        }
        ("brigadier:long", CommandArgumentPayload::Long(bounds)) => {
            write_i64_bounds(writer, *bounds)?;
        }
        ("brigadier:string", CommandArgumentPayload::String(kind)) => {
            writer.write_var_i32(kind.id())?;
        }
        ("minecraft:entity" | "minecraft:score_holder", CommandArgumentPayload::Flags(flags)) => {
            writer.write_u8(*flags)?
        }
        ("minecraft:time", CommandArgumentPayload::TimeMinimum(minimum)) => {
            writer.write_i32(*minimum)?;
        }
        (
            "minecraft:resource_or_tag"
            | "minecraft:resource_or_tag_key"
            | "minecraft:resource"
            | "minecraft:resource_key"
            | "minecraft:resource_selector",
            CommandArgumentPayload::Registry(registry),
        ) => registry.write(writer)?,
        (_, CommandArgumentPayload::None) if argument_payload_is_empty(&identity) => {}
        _ => {
            return Err(CommandTreeError::ArgumentPayloadMismatch {
                argument_type: argument_type.clone(),
            });
        }
    }
    Ok(())
}

fn argument_payload_is_empty(identity: &str) -> bool {
    !matches!(
        identity,
        "brigadier:float"
            | "brigadier:double"
            | "brigadier:integer"
            | "brigadier:long"
            | "brigadier:string"
            | "minecraft:entity"
            | "minecraft:score_holder"
            | "minecraft:time"
            | "minecraft:resource_or_tag"
            | "minecraft:resource_or_tag_key"
            | "minecraft:resource"
            | "minecraft:resource_key"
            | "minecraft:resource_selector"
    )
}

fn read_f32_bounds(reader: &mut WireReader<'_>) -> Result<NumericBounds<f32>, CommandTreeError> {
    let flags = reader.read_u8()?;
    Ok(NumericBounds {
        minimum: (flags & 1 != 0).then(|| reader.read_f32()).transpose()?,
        maximum: (flags & 2 != 0).then(|| reader.read_f32()).transpose()?,
    })
}

fn read_f64_bounds(reader: &mut WireReader<'_>) -> Result<NumericBounds<f64>, CommandTreeError> {
    let flags = reader.read_u8()?;
    Ok(NumericBounds {
        minimum: (flags & 1 != 0).then(|| reader.read_f64()).transpose()?,
        maximum: (flags & 2 != 0).then(|| reader.read_f64()).transpose()?,
    })
}

fn read_i32_bounds(reader: &mut WireReader<'_>) -> Result<NumericBounds<i32>, CommandTreeError> {
    let flags = reader.read_u8()?;
    Ok(NumericBounds {
        minimum: (flags & 1 != 0).then(|| reader.read_i32()).transpose()?,
        maximum: (flags & 2 != 0).then(|| reader.read_i32()).transpose()?,
    })
}

fn read_i64_bounds(reader: &mut WireReader<'_>) -> Result<NumericBounds<i64>, CommandTreeError> {
    let flags = reader.read_u8()?;
    Ok(NumericBounds {
        minimum: (flags & 1 != 0).then(|| reader.read_i64()).transpose()?,
        maximum: (flags & 2 != 0).then(|| reader.read_i64()).transpose()?,
    })
}

fn write_f32_bounds(writer: &mut WireWriter, bounds: NumericBounds<f32>) -> Result<(), WireError> {
    writer.write_u8(bound_flags(&bounds))?;
    if let Some(value) = bounds.minimum {
        writer.write_f32(value)?;
    }
    if let Some(value) = bounds.maximum {
        writer.write_f32(value)?;
    }
    Ok(())
}

fn write_f64_bounds(writer: &mut WireWriter, bounds: NumericBounds<f64>) -> Result<(), WireError> {
    writer.write_u8(bound_flags(&bounds))?;
    if let Some(value) = bounds.minimum {
        writer.write_f64(value)?;
    }
    if let Some(value) = bounds.maximum {
        writer.write_f64(value)?;
    }
    Ok(())
}

fn write_i32_bounds(writer: &mut WireWriter, bounds: NumericBounds<i32>) -> Result<(), WireError> {
    writer.write_u8(bound_flags(&bounds))?;
    if let Some(value) = bounds.minimum {
        writer.write_i32(value)?;
    }
    if let Some(value) = bounds.maximum {
        writer.write_i32(value)?;
    }
    Ok(())
}

fn write_i64_bounds(writer: &mut WireWriter, bounds: NumericBounds<i64>) -> Result<(), WireError> {
    writer.write_u8(bound_flags(&bounds))?;
    if let Some(value) = bounds.minimum {
        writer.write_i64(value)?;
    }
    if let Some(value) = bounds.maximum {
        writer.write_i64(value)?;
    }
    Ok(())
}

fn bound_flags<T>(bounds: &NumericBounds<T>) -> u8 {
    u8::from(bounds.minimum.is_some()) | (u8::from(bounds.maximum.is_some()) << 1)
}

fn read_identifier(reader: &mut WireReader<'_>) -> Result<Identifier, CommandTreeError> {
    Identifier::read(reader).map_err(|error| match error {
        IdentifierReadError::Wire(error) => error.into(),
        IdentifierReadError::Invalid(error) => error.into(),
    })
}

fn index(value: i32, length: usize, kind: &'static str) -> Result<usize, CommandTreeError> {
    let index = usize::try_from(value).map_err(|_| CommandTreeError::ReferenceOutOfRange {
        kind,
        index: value,
        nodes: length,
    })?;
    if index >= length {
        Err(CommandTreeError::ReferenceOutOfRange {
            kind,
            index: value,
            nodes: length,
        })
    } else {
        Ok(index)
    }
}

#[derive(Debug, Clone, PartialEq, Error)]
pub enum CommandTreeError {
    #[error(transparent)]
    Wire(#[from] WireError),
    #[error(transparent)]
    InvalidIdentifier(#[from] IdentifierError),
    #[error(transparent)]
    Registry(#[from] PlayRegistryError),
    #[error("Brigadier string argument kind {id} is outside 0..=2")]
    InvalidStringArgumentKind { id: i32 },
    #[error("command {kind} reference {index} is outside {nodes} nodes")]
    ReferenceOutOfRange {
        kind: &'static str,
        index: i32,
        nodes: usize,
    },
    #[error("command graph contains a reachable cycle at node {index}")]
    Cycle { index: usize },
    #[error("command root {index} resolves to an inert placeholder")]
    PlaceholderRoot { index: i32 },
    #[error("command argument {argument_type} has a mismatched codec payload")]
    ArgumentPayloadMismatch { argument_type: Identifier },
}

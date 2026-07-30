use serde::Deserialize;
use sha1::{Digest, Sha1};
use std::collections::BTreeSet;
use std::error::Error;
use std::fmt::Write as _;
use std::fs;
use std::io::{Error as IoError, ErrorKind};
use std::path::PathBuf;

const CATALOG_RELATIVE: &str = "reference/minecraft-java-26.2-packets.toml";
const METADATA_ACCESSORS_RELATIVE: &str =
    "reference/minecraft-java-26.2-entity-metadata-accessors.tsv";
const EXPECTED_SCHEMA: u32 = 1;
const EXPECTED_VERSION: &str = "26.2";
const EXPECTED_PROTOCOL: u32 = 776;
const EXPECTED_COUNT: usize = 256;
const EXPECTED_LANES: usize = 9;
const EXPECTED_SHA1: &str = "f34b0956b6399c749d4638cd6d3c9226685f41fa";
const EXPECTED_METADATA_ACCESSORS: usize = 221;
const EXPECTED_METADATA_ACCESSORS_SHA1: &str = "b489eec18fc1981ebfb7ac97c54a4485fe2f938a";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CatalogLock {
    schema: u32,
    minecraft_version: String,
    protocol_version: u32,
    entries_sha1: String,
    lane: Vec<CatalogLane>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CatalogLane {
    state: String,
    direction: String,
    identities: Vec<String>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let manifest = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").ok_or_else(|| {
        invalid("Cargo did not provide CARGO_MANIFEST_DIR for packet catalog generation")
    })?);
    let catalog_path = manifest.join(CATALOG_RELATIVE);
    println!("cargo::rerun-if-changed={}", catalog_path.display());
    let source = fs::read_to_string(&catalog_path)?;
    let catalog: CatalogLock = toml::from_str(&source)?;
    validate(&catalog)?;

    let generated = render_rust(&catalog)?;
    let output_directory = PathBuf::from(
        std::env::var_os("OUT_DIR")
            .ok_or_else(|| invalid("Cargo did not provide OUT_DIR for protocol generation"))?,
    );
    fs::write(
        output_directory.join("minecraft_java_26_2_packet_catalog.rs"),
        generated,
    )?;

    let metadata_path = manifest.join(METADATA_ACCESSORS_RELATIVE);
    println!("cargo::rerun-if-changed={}", metadata_path.display());
    let metadata_source = fs::read_to_string(metadata_path)?;
    let metadata = validate_metadata_accessors(&metadata_source)?;
    fs::write(
        output_directory.join("minecraft_java_26_2_entity_metadata_accessors.rs"),
        render_metadata_accessors(&metadata)?,
    )?;
    Ok(())
}

#[derive(Debug)]
struct MetadataAccessor<'a> {
    declaring_class: &'a str,
    field: &'a str,
    slot: u8,
    serializer: u8,
}

fn validate_metadata_accessors(source: &str) -> Result<Vec<MetadataAccessor<'_>>, IoError> {
    let actual_sha1 = lower_hex(&Sha1::digest(source.as_bytes()));
    require(
        actual_sha1 == EXPECTED_METADATA_ACCESSORS_SHA1,
        format!(
            "entity metadata accessor digest is {actual_sha1}, expected \
             {EXPECTED_METADATA_ACCESSORS_SHA1}"
        ),
    )?;
    let mut accessors = Vec::new();
    let mut previous = None;
    let mut keys = BTreeSet::new();
    for line in source.lines() {
        let mut columns = line.split('\t');
        let owner = columns
            .next()
            .ok_or_else(|| invalid("metadata accessor row is missing owner"))?;
        let slot = columns
            .next()
            .ok_or_else(|| invalid("metadata accessor row is missing slot"))?
            .parse::<u8>()
            .map_err(|_| invalid(format!("invalid metadata slot in {line}")))?;
        let serializer = columns
            .next()
            .ok_or_else(|| invalid("metadata accessor row is missing serializer"))?
            .parse::<u8>()
            .map_err(|_| invalid(format!("invalid metadata serializer in {line}")))?;
        require(
            columns.next().is_none(),
            format!("metadata accessor row has extra columns: {line}"),
        )?;
        let (declaring_class, field) = owner
            .split_once('#')
            .ok_or_else(|| invalid(format!("metadata accessor owner lacks #: {owner}")))?;
        require(
            !declaring_class.is_empty() && !field.is_empty(),
            format!("metadata accessor owner is incomplete: {owner}"),
        )?;
        require(
            slot < u8::MAX,
            format!("metadata accessor {owner} uses terminator slot"),
        )?;
        require(
            serializer < 43,
            format!("metadata accessor {owner} has serializer {serializer}"),
        )?;
        require(
            previous.is_none_or(|previous: &str| previous < line),
            format!("metadata accessor rows are not strictly ordinal-sorted at {line}"),
        )?;
        require(
            keys.insert((declaring_class, field)),
            format!("duplicate metadata accessor {owner}"),
        )?;
        previous = Some(line);
        accessors.push(MetadataAccessor {
            declaring_class,
            field,
            slot,
            serializer,
        });
    }
    require(
        accessors.len() == EXPECTED_METADATA_ACCESSORS,
        format!(
            "metadata accessor table has {} rows, expected {EXPECTED_METADATA_ACCESSORS}",
            accessors.len()
        ),
    )?;
    Ok(accessors)
}

fn render_metadata_accessors(accessors: &[MetadataAccessor<'_>]) -> Result<String, IoError> {
    let mut output = String::from(
        "// @generated by ferrite-protocol/build.rs from the reviewed metadata accessor lock.\n",
    );
    output.push_str("// The source TSV, not this OUT_DIR file, is the review boundary.\n\n");
    output.push_str("pub(super) const ACCESSORS: &[MetadataAccessorDeclaration] = &[\n");
    for accessor in accessors {
        let serializer = metadata_serializer_variant(accessor.serializer)
            .ok_or_else(|| invalid("validated metadata serializer missing"))?;
        writeln!(
            output,
            "    MetadataAccessorDeclaration::new({:?}, {:?}, {}, \
             MetadataSerializer::{}),",
            accessor.declaring_class, accessor.field, accessor.slot, serializer
        )
        .map_err(|_| invalid("could not render metadata accessor"))?;
    }
    output.push_str("];\n");
    Ok(output)
}

fn metadata_serializer_variant(raw_id: u8) -> Option<&'static str> {
    const VARIANTS: [&str; 43] = [
        "Byte",
        "Int",
        "Long",
        "Float",
        "String",
        "Component",
        "OptionalComponent",
        "ItemStack",
        "Boolean",
        "Rotations",
        "BlockPos",
        "OptionalBlockPos",
        "Direction",
        "OptionalLivingEntityReference",
        "BlockState",
        "OptionalBlockState",
        "Particle",
        "Particles",
        "VillagerData",
        "OptionalUnsignedInt",
        "Pose",
        "CatVariant",
        "CatSoundVariant",
        "CowVariant",
        "CowSoundVariant",
        "WolfVariant",
        "WolfSoundVariant",
        "FrogVariant",
        "PigVariant",
        "PigSoundVariant",
        "ChickenVariant",
        "ChickenSoundVariant",
        "ZombieNautilusVariant",
        "OptionalGlobalPos",
        "PaintingVariant",
        "SnifferState",
        "ArmadilloState",
        "CopperGolemState",
        "WeatheringCopperState",
        "Vector3",
        "Quaternion",
        "ResolvableProfile",
        "HumanoidArm",
    ];
    VARIANTS.get(raw_id as usize).copied()
}

fn validate(catalog: &CatalogLock) -> Result<(), IoError> {
    require(
        catalog.schema == EXPECTED_SCHEMA,
        format!("packet catalog schema must be {EXPECTED_SCHEMA}"),
    )?;
    require(
        catalog.minecraft_version == EXPECTED_VERSION,
        format!("packet catalog version must be {EXPECTED_VERSION}"),
    )?;
    require(
        catalog.protocol_version == EXPECTED_PROTOCOL,
        format!("packet catalog protocol must be {EXPECTED_PROTOCOL}"),
    )?;
    require(
        catalog.entries_sha1 == EXPECTED_SHA1,
        format!("packet catalog declared digest must be {EXPECTED_SHA1}"),
    )?;
    require(
        catalog.lane.len() == EXPECTED_LANES,
        format!("packet catalog must contain {EXPECTED_LANES} lanes"),
    )?;

    let mut lane_keys = BTreeSet::new();
    let mut packet_keys = BTreeSet::new();
    let mut canonical = Vec::new();
    for lane in &catalog.lane {
        require(
            state_variant(&lane.state).is_some(),
            format!("unknown packet state {}", lane.state),
        )?;
        require(
            direction_variant(&lane.direction).is_some(),
            format!("unknown packet direction {}", lane.direction),
        )?;
        require(
            lane_keys.insert((&lane.state, &lane.direction)),
            format!("duplicate packet lane {}/{}", lane.state, lane.direction),
        )?;
        require(
            !lane.identities.is_empty(),
            format!("empty packet lane {}/{}", lane.state, lane.direction),
        )?;
        for (protocol_id, identity) in lane.identities.iter().enumerate() {
            require(
                valid_identity(identity),
                format!("invalid packet identity {identity}"),
            )?;
            require(
                packet_keys.insert((&lane.state, &lane.direction, identity)),
                format!(
                    "duplicate packet identity {}/{}/{}",
                    lane.state, lane.direction, identity
                ),
            )?;
            canonical.push((
                lane.state.as_str(),
                lane.direction.as_str(),
                identity.as_str(),
                protocol_id,
            ));
        }
    }
    require(
        packet_keys.len() == EXPECTED_COUNT,
        format!("packet catalog must contain {EXPECTED_COUNT} packets"),
    )?;
    canonical.sort_unstable();
    let mut bytes = String::new();
    for (state, direction, identity, protocol_id) in canonical {
        writeln!(bytes, "{state}\t{direction}\t{identity}\t{protocol_id}")
            .map_err(|_| invalid("could not render packet catalog digest input"))?;
    }
    let actual_sha1 = lower_hex(&Sha1::digest(bytes.as_bytes()));
    require(
        actual_sha1 == EXPECTED_SHA1,
        format!("packet catalog digest is {actual_sha1}, expected {EXPECTED_SHA1}"),
    )
}

fn render_rust(catalog: &CatalogLock) -> Result<String, IoError> {
    let mut output = String::from(
        "// @generated by ferrite-protocol/build.rs from the reviewed packet catalog lock.\n",
    );
    output.push_str("// The source lock, not this OUT_DIR file, is the review boundary.\n\n");
    output.push_str("pub(super) const PACKETS: &[PacketDescriptor] = &[\n");
    for lane in &catalog.lane {
        let state = state_variant(&lane.state).ok_or_else(|| invalid("validated state missing"))?;
        let direction = direction_variant(&lane.direction)
            .ok_or_else(|| invalid("validated direction missing"))?;
        for (protocol_id, identity) in lane.identities.iter().enumerate() {
            writeln!(
                output,
                "    PacketDescriptor::new(ConnectionState::{state}, \
                 PacketDirection::{direction}, PacketId::new({protocol_id}), {identity:?}),"
            )
            .map_err(|_| invalid("could not render packet descriptor"))?;
        }
    }
    output.push_str("];\n\n");
    output.push_str("pub(super) const LANES: &[LaneDescriptor] = &[\n");
    let mut offset = 0;
    for lane in &catalog.lane {
        let state = state_variant(&lane.state).ok_or_else(|| invalid("validated state missing"))?;
        let direction = direction_variant(&lane.direction)
            .ok_or_else(|| invalid("validated direction missing"))?;
        let end = offset + lane.identities.len();
        writeln!(
            output,
            "    LaneDescriptor::new(ConnectionState::{state}, \
             PacketDirection::{direction}, {offset}, {end}),"
        )
        .map_err(|_| invalid("could not render packet lane"))?;
        offset = end;
    }
    output.push_str("];\n");
    Ok(output)
}

fn state_variant(state: &str) -> Option<&'static str> {
    match state {
        "configuration" => Some("Configuration"),
        "handshake" => Some("Handshake"),
        "login" => Some("Login"),
        "play" => Some("Play"),
        "status" => Some("Status"),
        _ => None,
    }
}

fn direction_variant(direction: &str) -> Option<&'static str> {
    match direction {
        "clientbound" => Some("Clientbound"),
        "serverbound" => Some("Serverbound"),
        _ => None,
    }
}

fn valid_identity(identity: &str) -> bool {
    let Some((namespace, path)) = identity.split_once(':') else {
        return false;
    };
    !namespace.is_empty()
        && !path.is_empty()
        && namespace.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"_.-".contains(&byte)
        })
        && path.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"_./-".contains(&byte)
        })
}

fn lower_hex(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(output, "{byte:02x}").expect("writing to a String cannot fail");
    }
    output
}

fn require(condition: bool, message: String) -> Result<(), IoError> {
    if condition {
        Ok(())
    } else {
        Err(invalid(message))
    }
}

fn invalid(message: impl Into<String>) -> IoError {
    IoError::new(ErrorKind::InvalidData, message.into())
}

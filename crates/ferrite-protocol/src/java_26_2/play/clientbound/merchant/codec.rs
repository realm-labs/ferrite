use thiserror::Error;

use crate::java_26_2::play::clientbound::merchant::packet::{
    ItemCost, MerchantOffer, MerchantOffers,
};
use crate::java_26_2::play::context::{ComponentValueError, PlayDecodeContext};
use crate::java_26_2::play::item::{
    EncodedComponentValue, ItemCodecError, ItemStack, read_optional_stack, write_optional_stack,
};
use crate::java_26_2::play::registry::{
    DATA_COMPONENT_TYPE, ITEM, PlayRegistries, PlayRegistryError,
};
use crate::java_26_2::wire::compression::MAX_INFLATED_PACKET_LENGTH;
use crate::java_26_2::wire::error::WireError;
use crate::java_26_2::wire::primitive::{WireReader, WireWriter};

const COLLECTION_ALLOCATION_CAPACITY: usize = 65_536;

pub(crate) fn read(
    reader: &mut WireReader<'_>,
    context: PlayDecodeContext<'_>,
) -> Result<MerchantOffers, MerchantCodecError> {
    let container_id = reader.read_var_i32()?;
    let count = reader.read_count("merchant offers", reader.remaining())?;
    let mut offers = Vec::with_capacity(count.min(COLLECTION_ALLOCATION_CAPACITY));
    for _ in 0..count {
        offers.push(read_offer(reader, context)?);
    }
    Ok(MerchantOffers {
        container_id,
        offers,
        villager_level: reader.read_var_i32()?,
        villager_experience: reader.read_var_i32()?,
        show_progress: reader.read_bool()?,
        can_restock: reader.read_bool()?,
    })
}

pub(crate) fn write(
    writer: &mut WireWriter,
    packet: &MerchantOffers,
    registries: &PlayRegistries,
) -> Result<(), MerchantCodecError> {
    writer.write_var_i32(packet.container_id)?;
    writer.write_count(
        "merchant offers",
        packet.offers.len(),
        MAX_INFLATED_PACKET_LENGTH,
    )?;
    for offer in &packet.offers {
        write_offer(writer, offer, registries)?;
    }
    writer.write_var_i32(packet.villager_level)?;
    writer.write_var_i32(packet.villager_experience)?;
    writer.write_bool(packet.show_progress)?;
    writer.write_bool(packet.can_restock)?;
    Ok(())
}

fn read_offer(
    reader: &mut WireReader<'_>,
    context: PlayDecodeContext<'_>,
) -> Result<MerchantOffer, MerchantCodecError> {
    let cost_a = read_cost(reader, context)?;
    let result = read_optional_stack(reader, context)?;
    if result.is_empty() {
        return Err(MerchantCodecError::EmptyResult);
    }
    let cost_b = if reader.read_bool()? {
        Some(read_cost(reader, context)?)
    } else {
        None
    };
    let out_of_stock = reader.read_bool()?;
    let wire_uses = reader.read_i32()?;
    let max_uses = reader.read_i32()?;
    let experience = reader.read_i32()?;
    let special_price_difference = reader.read_i32()?;
    let price_multiplier = reader.read_f32()?;
    let demand = reader.read_i32()?;
    Ok(MerchantOffer {
        cost_a,
        result,
        cost_b,
        uses: if out_of_stock { max_uses } else { wire_uses },
        max_uses,
        experience,
        special_price_difference,
        price_multiplier,
        demand,
        reward_experience: true,
    })
}

fn write_offer(
    writer: &mut WireWriter,
    offer: &MerchantOffer,
    registries: &PlayRegistries,
) -> Result<(), MerchantCodecError> {
    write_cost(writer, &offer.cost_a, registries)?;
    if !is_nonempty_result(&offer.result) {
        return Err(MerchantCodecError::EmptyResult);
    }
    write_optional_stack(writer, &offer.result, registries)?;
    writer.write_bool(offer.cost_b.is_some())?;
    if let Some(cost) = &offer.cost_b {
        write_cost(writer, cost, registries)?;
    }
    writer.write_bool(offer.is_out_of_stock())?;
    writer.write_i32(offer.uses)?;
    writer.write_i32(offer.max_uses)?;
    writer.write_i32(offer.experience)?;
    writer.write_i32(offer.special_price_difference)?;
    writer.write_f32(offer.price_multiplier)?;
    writer.write_i32(offer.demand)?;
    Ok(())
}

fn is_nonempty_result(stack: &ItemStack) -> bool {
    stack.contents().is_some_and(|contents| {
        contents.count > 0
            && !(contents.item.namespace() == "minecraft" && contents.item.path() == "air")
    })
}

fn read_cost(
    reader: &mut WireReader<'_>,
    context: PlayDecodeContext<'_>,
) -> Result<ItemCost, MerchantCodecError> {
    let item = context.registries.resolve(ITEM, reader.read_var_i32()?)?;
    let count = reader.read_var_i32()?;
    let component_count = reader.read_count("exact item components", reader.remaining())?;
    let mut components = Vec::with_capacity(component_count.min(COLLECTION_ALLOCATION_CAPACITY));
    for _ in 0..component_count {
        let component = context
            .registries
            .resolve(DATA_COMPONENT_TYPE, reader.read_var_i32()?)?;
        let encoded_value = context.component_values.decode_value(&component, reader)?;
        components.push(EncodedComponentValue {
            component,
            encoded_value,
        });
    }
    Ok(ItemCost {
        item,
        count,
        components,
    })
}

fn write_cost(
    writer: &mut WireWriter,
    cost: &ItemCost,
    registries: &PlayRegistries,
) -> Result<(), MerchantCodecError> {
    writer.write_var_i32(registries.raw_id(ITEM, &cost.item)?)?;
    writer.write_var_i32(cost.count)?;
    writer.write_count(
        "exact item components",
        cost.components.len(),
        MAX_INFLATED_PACKET_LENGTH,
    )?;
    for component in &cost.components {
        writer.write_var_i32(registries.raw_id(DATA_COMPONENT_TYPE, &component.component)?)?;
        writer.write_bytes(&component.encoded_value)?;
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum MerchantCodecError {
    #[error(transparent)]
    Wire(#[from] WireError),
    #[error(transparent)]
    Registry(#[from] PlayRegistryError),
    #[error(transparent)]
    ComponentValue(#[from] ComponentValueError),
    #[error(transparent)]
    Item(#[from] ItemCodecError),
    #[error("merchant offer result must be a nonempty item stack")]
    EmptyResult,
}

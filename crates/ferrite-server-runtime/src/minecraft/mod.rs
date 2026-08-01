//! Minecraft Java network ingress and continuous local-world composition.

mod collision;
mod entry;
mod gateway;
mod settings;
mod tags;
mod world;

pub(crate) use gateway::{MinecraftGateway, MinecraftGatewayError};

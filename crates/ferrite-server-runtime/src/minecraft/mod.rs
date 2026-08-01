//! Minecraft Java network ingress and continuous local-world composition.

mod collision;
mod entry;
mod environment;
mod gateway;
mod settings;
mod tags;
mod world;

pub(crate) use gateway::{MinecraftGateway, MinecraftGatewayError};

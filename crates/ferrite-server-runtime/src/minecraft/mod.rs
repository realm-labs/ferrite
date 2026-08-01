//! Minecraft Java network ingress and continuous local-world composition.

mod collision;
mod entry;
mod environment;
mod gateway;
mod portal;
#[cfg(test)]
mod portal_continuity;
mod settings;
mod tags;
mod world;

pub(crate) use gateway::{MinecraftGateway, MinecraftGatewayError};

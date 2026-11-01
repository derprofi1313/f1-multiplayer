//! The Discord invite the help links point to. The News page shows F1 Multiplayer's own banner
//! ([`crate::hero`]) instead of a Discord banner.

pub const INVITE: &str = "https://discord.gg/STWsgyJKZS";

pub(crate) const LOGO: &[u8] = include_bytes!("../assets/discord.svg");
pub(crate) const BLURPLE: u32 = 0x5865f2;
pub(crate) const BLURPLE_HOVER: u32 = 0x4752c4;

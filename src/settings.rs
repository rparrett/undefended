use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Resource, Deref, DerefMut, Debug, Serialize, Deserialize, Clone)]
pub struct MusicSetting(u8);
impl Default for MusicSetting {
    fn default() -> Self {
        Self(100)
    }
}

#[derive(Resource, Deref, DerefMut, Debug, Serialize, Deserialize, Clone)]
pub struct SfxSetting(u8);
impl Default for SfxSetting {
    fn default() -> Self {
        Self(100)
    }
}

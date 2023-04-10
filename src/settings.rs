use std::fmt::Display;

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
#[derive(Resource, Debug, Default, Serialize, Deserialize, Clone)]
pub enum DifficultySetting {
    #[default]
    Normal,
    Hard,
    Extra,
}
impl DifficultySetting {
    pub fn next(&self) -> Self {
        match self {
            Self::Normal => Self::Hard,
            Self::Hard => Self::Extra,
            Self::Extra => Self::Normal,
        }
    }
}
impl Display for DifficultySetting {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Normal => "NORMAL",
                Self::Hard => "HARD",
                Self::Extra => "EXTRA",
            }
        )
    }
}

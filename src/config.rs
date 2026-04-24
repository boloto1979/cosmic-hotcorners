// SPDX-License-Identifier: MPL-2.0

use cosmic::cosmic_config::{self, CosmicConfigEntry, cosmic_config_derive::CosmicConfigEntry};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CornerAction {
    Disabled,
    ShowWorkspaces,
    OpenLauncher,
    RunCommand(String),
}

impl Default for CornerAction {
    fn default() -> Self {
        CornerAction::Disabled
    }
}

#[derive(Debug, Clone, CosmicConfigEntry, Eq, PartialEq)]
#[version = 1]
pub struct Config {
    pub enabled: bool,
    pub delay_ms: u64,
    pub top_left: CornerAction,
    pub top_right: CornerAction,
    pub bottom_left: CornerAction,
    pub bottom_right: CornerAction,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            enabled: true,
            delay_ms: 300,
            top_left: CornerAction::Disabled,
            top_right: CornerAction::Disabled,
            bottom_left: CornerAction::Disabled,
            bottom_right: CornerAction::Disabled,
        }
    }
}

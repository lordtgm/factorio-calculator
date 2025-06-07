use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Effects {
    pub consumption: Option<f32>,
    pub speed: Option<f32>,
    pub productivity: Option<f32>,
    pub quality: Option<f32>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct EffectReceiver {
    pub base_effect: Option<Effects>,
    pub uses_module_effects: bool,
    pub uses_beacon_effects: bool,
    pub uses_surface_effects: bool,
}

impl Default for EffectReceiver {
    fn default() -> Self {
        EffectReceiver {
            base_effect: None,
            uses_module_effects: true,
            uses_beacon_effects: true,
            uses_surface_effects: true,
        }
    }
}

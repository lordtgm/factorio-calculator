use crate::data::effects::{EffectReceiver, Effects};
use crate::data::materials::Material;

#[derive(Debug)]
pub enum EnergySource {
    Electric {
        drain: u32,
    },
    Burner {
        effectivity: f64,
        fuel_categories: Vec<String>,
    },
    Heat,
    Fluid {
        effectivity: f64,
        burns_fluid: bool,
        fluid_usage_per_tick: u32,
        scale_fluid_usage: bool,
    },
    Void
}

pub struct MiningDrillPrototype {
    pub name: String,
    pub energy_usage: u32,
    pub mining_speed: f64,
    pub energy_source: EnergySource,
    pub resource_categories: Vec<String>,
    pub effect_receiver: Option<EffectReceiver>,
    pub allowed_effects: Vec<String>,
    pub allowed_module_categories: Vec<String>,
    pub module_slots: u16,
    pub resource_drain_rate_percent: u8,
}

#[derive(Debug)]
pub struct CraftingMachinePrototype {
    pub name: String,
    pub energy_usage: u32,
    pub crafting_speed: f64,
    pub crafting_categories: Vec<String>,
    pub energy_source: EnergySource,
    pub effect_receiver: Option<EffectReceiver>,
    pub allowed_effects: Vec<String>,
    pub allowed_module_categories: Vec<String>,
    pub module_slots: u16,
}

pub struct RecipePrototype {
    pub name: String,
    pub category: String,
    pub ingredients: Vec<Material>,
    pub results: Vec<Material>,
    pub energy_required: f64,
    pub allowed_effects: Vec<String>,
}

pub struct ModulePrototype {
    pub name: String,
    pub category: String,
    pub effects: Effects,
}
#[derive(Debug)]
pub struct BeaconPrototype {
    pub name: String,
    pub energy_source: EnergySource,
    pub energy_usage: u32,
    pub efficiency: f64,
    pub efficiency_per_quality: f64,
    pub module_slots: u16,
    pub allowed_effects: Vec<String>,
    pub allowed_module_categories: Vec<String>,
}
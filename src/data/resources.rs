use crate::data::materials::{Fluid, Material};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Minable {
    pub mining_time: f64,
    pub results: Vec<Material>,
    pub input_fluid: Option<Fluid>,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct PlantPrototype {
    pub name: String,
    pub growth_ticks: u32,
    pub results: Minable,
    pub seeds: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ResourcePrototype {
    pub name: String,
    pub category: String,
    pub results: Minable,
}

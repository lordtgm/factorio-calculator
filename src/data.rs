use crate::data::machines::{BeaconPrototype, CraftingMachinePrototype, MiningDrillPrototype, ModulePrototype, RecipePrototype};
use crate::data::materials::{FluidPrototype, Item, ItemPrototype, Material};
use crate::data::resources::{PlantPrototype, ResourcePrototype};
use std::collections::HashMap;
use std::hash::Hash;
use std::rc::Weak;

pub mod data_loader;
pub mod effects;
pub mod machines;
pub mod materials;
pub mod resources;
pub mod types;

#[derive(Default)]
pub struct Registry {
    pub items: HashMap<String, ItemPrototype>,
    pub fluids: HashMap<String, FluidPrototype>,
    pub resources: HashMap<String, ResourcePrototype>,
    pub plants: HashMap<String, PlantPrototype>,
    pub mining_drills: HashMap<String, MiningDrillPrototype>,
    pub crafting_machines: HashMap<String, CraftingMachinePrototype>,
    pub recipes: HashMap<String, RecipePrototype>,
    pub modules: HashMap<String, ModulePrototype>,
    pub beacons: HashMap<String, BeaconPrototype>,
    // pub processes: Vec<Process>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ProcessType {
    Resource,
    Plant,
    Recipe,
}

impl Into<String> for &ProcessType {
    fn into(self) -> String {
        match self {
            ProcessType::Resource => "Resource",
            ProcessType::Plant => "Plant",
            ProcessType::Recipe => "Recipe",
        }
        .into()
    }
}

impl TryFrom<&String> for ProcessType {
    type Error = &'static str;
    fn try_from(value: &String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "Resource" => Ok(ProcessType::Resource),
            "Plant" => Ok(ProcessType::Plant),
            "Recipe" => Ok(ProcessType::Recipe),
            _ => Err("Unknown process type"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Process {
    pub process_type: ProcessType,
    pub registry: Weak<Registry>,
    pub name: String,
    pub productivity: f32,
}

impl Hash for Process {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl PartialEq for Process {
    fn eq(&self, other: &Self) -> bool {
        self.process_type == other.process_type && self.name == other.name
    }
}

impl Eq for Process {}

impl Process {
    pub fn get_ingredients(&self) -> Vec<Material> {
        match self.process_type {
            ProcessType::Resource => self
                .registry
                .upgrade()
                .unwrap()
                .resources
                .get(&self.name)
                .unwrap()
                .results
                .input_fluid
                .clone()
                .map(|fluid| vec![Material::Fluid(fluid)])
                .unwrap_or_else(|| vec![]),
            ProcessType::Plant => self
                .registry
                .upgrade()
                .unwrap()
                .plants
                .get(&self.name)
                .unwrap()
                .seeds
                .iter()
                .map(|seed| {
                    Material::Item(Item {
                        name: seed.clone(),
                        quality: None,
                        amount: Some(1),
                        amount_min: None,
                        amount_max: None,
                        probability: None,
                        ignored_by_productivity: None,
                        extra_count_fraction: None,
                    })
                })
                .collect(),
            ProcessType::Recipe => self
                .registry
                .upgrade()
                .unwrap()
                .recipes
                .get(&self.name)
                .unwrap()
                .ingredients
                .clone(),
        }
    }

    pub fn get_products(&self) -> Vec<Material> {
        match self.process_type {
            ProcessType::Resource => self
                .registry
                .upgrade()
                .unwrap()
                .resources
                .get(&self.name)
                .unwrap()
                .results
                .results
                .clone(),
            ProcessType::Plant => self
                .registry
                .upgrade()
                .unwrap()
                .plants
                .get(&self.name)
                .unwrap()
                .results
                .results
                .clone(),
            ProcessType::Recipe => self
                .registry
                .upgrade()
                .unwrap()
                .recipes
                .get(&self.name)
                .unwrap()
                .results
                .clone(),
        }
    }

}

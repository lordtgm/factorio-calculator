use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum Material {
    Item(Item),
    Fluid(Fluid),
}

#[derive(PartialEq, Eq, Hash, Clone, Debug, Serialize, Deserialize)]
pub enum MaterialPrototype {
    Item(String),
    Fluid(String),
}

impl MaterialPrototype {
    // pub fn get_name(&self) -> String {
    //     match self {
    //         MaterialPrototype::Item(item) => item,
    //         MaterialPrototype::Fluid(fluid) => fluid,
    //     }
    //     .clone()
    // }

    pub fn to_id(&self) -> String {
        match self {
            MaterialPrototype::Item(item) => format!("item:{}", item),
            MaterialPrototype::Fluid(fluid) => format!("fluid:{}", fluid),
        }
    }

    pub fn from_id(id: &str) -> Result<Self, String> {
        match id.split_once(':') {
            Some(("item", item)) => Ok(MaterialPrototype::Item(item.into())),
            Some(("fluid", fluid)) => Ok(MaterialPrototype::Fluid(fluid.into())),
            _ => Err(format!("Invalid material type '{}'", id)),
        }
    }
}

impl Material {
    pub fn get_prototype(&self) -> MaterialPrototype {
        match self {
            Material::Item(item) => MaterialPrototype::Item(item.name.clone()),
            Material::Fluid(fluid) => MaterialPrototype::Fluid(fluid.name.clone()),
        }
    }

    pub fn get_average_amount(&self, productivity: f64) -> f64 {
        match self {
            Material::Item(material) => {
                material.probability.unwrap_or(1.0)
                    * material
                        .amount
                        .map(|amount| amount as f64)
                        .unwrap_or_else(|| {
                            ((material.amount_max.unwrap() + material.amount_min.unwrap()) / 2)
                                as f64
                        })
                    * (1.0 + productivity)
                    - productivity * material.ignored_by_productivity.unwrap_or(0) as f64
                    + material.extra_count_fraction.unwrap_or(0.0) as f64
            }
            Material::Fluid(material) => {
                material.probability.unwrap_or(1.0)
                    * material.amount.unwrap_or_else(|| {
                        (material.amount_max.unwrap() + material.amount_min.unwrap()) / 2.0
                    })
                    * (1.0 + productivity)
                    - productivity * material.ignored_by_productivity.unwrap_or(0) as f64
            }
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ItemPrototype {
    pub name: String,
    pub stack_size: u32,
    pub fuel_category: Option<String>,
    pub fuel_value: Option<u32>,
    pub burnt_result: Option<String>,
    pub spoil_result: Option<String>,
    pub plant_result: Option<String>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Item {
    pub name: String,
    pub quality: Option<u8>,
    pub amount: Option<u16>,
    pub amount_min: Option<u16>,
    pub amount_max: Option<u16>,
    pub probability: Option<f64>,
    pub ignored_by_productivity: Option<u16>,
    pub extra_count_fraction: Option<f32>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct FluidPrototype {
    pub name: String,
    pub fuel_value: Option<u32>,
}
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Fluid {
    pub name: String,
    pub temperature: Option<f32>,
    pub amount: Option<f64>,
    pub amount_min: Option<f64>,
    pub amount_max: Option<f64>,
    pub probability: Option<f64>,
    pub ignored_by_productivity: Option<u16>,
}

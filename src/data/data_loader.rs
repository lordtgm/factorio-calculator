use crate::data::effects::{EffectReceiver, Effects};
use crate::data::machines::{
    BeaconPrototype, CraftingMachinePrototype, EnergySource, MiningDrillPrototype, ModulePrototype,
    RecipePrototype,
};
use crate::data::materials::{Fluid, FluidPrototype, Item, ItemPrototype, Material};
use crate::data::resources::{Minable, PlantPrototype, ResourcePrototype};
use crate::data::Registry;
use json::{Error, JsonValue};
use std::collections::HashMap;
use std::fs;
use std::string::String;

pub fn load_data(dump: String) -> Result<Registry, Error> {
    let mut registry: Registry = Registry::default();

    let data: String = fs::read_to_string(dump).unwrap();

    let parsed: JsonValue = json::parse(&data)?;

    for prototype in vec![
        "item",
        "ammo",
        "capsule",
        "gun",
        "module",
        "space-platform-starter-pack",
        "tool",
        "armor",
        "repair-tool",
    ] {
        for (name, value) in parsed[prototype].entries() {
            registry.items.insert(
                name.into(),
                ItemPrototype {
                    name: value["name"].as_str().unwrap().into(),
                    stack_size: value["stack_size"].as_u32().unwrap(),
                    fuel_category: value["fuel_category"].as_str().map(|s| s.into()),
                    fuel_value: value["fuel_value"].as_u32(),
                    burnt_result: value["burnt_result"].as_str().map(|s| s.into()),
                    spoil_result: value["spoil_result"].as_str().map(|s| s.into()),
                    plant_result: value["plant_result"].as_str().map(|s| s.into()),
                },
            );
        }
    }

    for (name, value) in parsed["fluid"].entries() {
        registry.fluids.insert(
            name.into(),
            FluidPrototype {
                name: value["name"].as_str().unwrap().into(),
                fuel_value: value["fuel_value"].as_u32(),
            },
        );
    }

    for (name, value) in parsed["resource"].entries() {
        registry.resources.insert(
            name.into(),
            ResourcePrototype {
                name: value["name"].as_str().unwrap().into(),
                category: value["category"]
                    .as_str()
                    .or(Some("basic-solid"))
                    .unwrap()
                    .into(),
                results: get_minable(&value["minable"]),
            },
        );
        // let process = Process::Resource {
        //     name: name.into(),
        //     productivity: 0.0,
        // };
        // registry.processes.push(process);
    }

    for (_name, value) in parsed["tile"].entries() {
        if value["fluid"].is_string() {
            registry.resources.insert(
                value["fluid"].as_str().unwrap().to_string() + " *tile",
                ResourcePrototype {
                    name: value["name"].as_str().unwrap().into(),
                    category: "calculator internal tile".into(),
                    results: Minable {
                        mining_time: 1.0,
                        results: vec![Material::Fluid(Fluid {
                            name: value["fluid"].as_str().unwrap().into(),
                            temperature: None,
                            amount: Some(1.0),
                            amount_min: None,
                            amount_max: None,
                            probability: None,
                            ignored_by_productivity: None,
                        })],
                        input_fluid: None,
                    },
                },
            );
        }
    }

    for (name, value) in parsed["plant"].entries() {
        registry.plants.insert(
            name.into(),
            PlantPrototype {
                name: value["name"].as_str().unwrap().into(),
                growth_ticks: value["growth_ticks"].as_u32().unwrap(),
                results: get_minable(&value["minable"]),
                seeds: vec![],
            },
        );
        // let process = Process::Plant {
        //     name: name.into(),
        //     productivity: 0.0,
        // };
        // registry.processes.push(process);
    }

    let mut plant_seeds: HashMap<String, Vec<String>> = HashMap::new();

    for (name, value) in registry.items.iter() {
        if !value
            .plant_result
            .as_ref()
            .is_some_and(|string: &String| !string.is_empty())
        {
            continue;
        }
        let plant_name = value.plant_result.as_ref().unwrap();
        if !plant_seeds.contains_key(plant_name.as_str()) {
            plant_seeds.insert(plant_name.clone(), vec![]);
        }
        plant_seeds
            .get_mut(plant_name.as_str())
            .as_mut()
            .unwrap()
            .push(name.clone());
    }

    for (plant_name, seeds) in plant_seeds {
        let plant = registry.plants.remove(&plant_name).unwrap();
        registry.plants.insert(
            plant_name.clone(),
            PlantPrototype {
                name: plant.name,
                growth_ticks: plant.growth_ticks,
                results: plant.results,
                seeds,
            },
        );
    }

    for (name, value) in parsed["mining-drill"].entries() {
        registry.mining_drills.insert(
            name.into(),
            MiningDrillPrototype {
                name: value["name"].as_str().unwrap().into(),
                energy_usage: get_energy(value["energy_usage"].as_str().unwrap().into()),
                mining_speed: value["mining_speed"].as_f64().unwrap(),
                energy_source: get_energy_source(&value["energy_source"]),
                resource_categories: value["resource_categories"]
                    .members()
                    .map(|string| string.as_str().unwrap().into())
                    .collect(),
                effect_receiver: get_effect_receiver(&value["effect_receiver"]),
                allowed_effects: value["allowed_effects"]
                    .members()
                    .map(|string| string.as_str().unwrap().into())
                    .collect(),
                allowed_module_categories: value["allowed_module_categories"]
                    .members()
                    .map(|string| string.as_str().unwrap().into())
                    .collect(),
                module_slots: value["module_slots"].as_u16().or(Some(0)).unwrap(),
                resource_drain_rate_percent: value["resource_drain_rate_percent"]
                    .as_u8()
                    .or(Some(100))
                    .unwrap(),
            },
        );
    }

    for (name, value) in parsed["offshore-pump"].entries() {
        registry.mining_drills.insert(
            name.into(),
            MiningDrillPrototype {
                name: value["name"].as_str().unwrap().into(),
                energy_usage: get_energy(value["energy_usage"].as_str().unwrap().into()),
                mining_speed: value["pumping_speed"].as_f64().unwrap(),
                energy_source: get_energy_source(&value["energy_source"]),
                resource_categories: vec![
                    "calculator internal tile".into(),
                ],
                effect_receiver: None,
                allowed_effects: vec![],
                allowed_module_categories: vec!["".into()],
                module_slots: 0,
                resource_drain_rate_percent: 0,
            }
        );
    }

    for (name, value) in parsed["assembling-machine"]
        .entries()
        .chain(parsed["furnace"].entries())
    {
        registry.crafting_machines.insert(
            name.into(),
            CraftingMachinePrototype {
                name: value["name"].as_str().unwrap().into(),
                energy_usage: get_energy(value["energy_usage"].as_str().unwrap().into()),
                crafting_speed: value["crafting_speed"].as_f64().unwrap(),
                crafting_categories: value["crafting_categories"]
                    .members()
                    .map(|string| string.as_str().unwrap().into())
                    .collect(),
                energy_source: get_energy_source(&value["energy_source"]),
                effect_receiver: get_effect_receiver(&value["effect_receiver"]),
                allowed_effects: value["allowed_effects"]
                    .members()
                    .map(|string| string.as_str().unwrap().into())
                    .collect(),
                allowed_module_categories: value["allowed_module_categories"]
                    .members()
                    .map(|string| string.as_str().unwrap().into())
                    .collect(),
                module_slots: value["module_slots"].as_u16().or(Some(0)).unwrap(),
            },
        );
    }

    for (name, value) in parsed["recipe"].entries() {
        registry.recipes.insert(
            name.into(),
            RecipePrototype {
                name: value["name"].as_str().unwrap().into(),
                category: value["category"]
                    .as_str()
                    .or(Some("crafting"))
                    .unwrap()
                    .into(),
                ingredients: get_materials(&value["ingredients"]),
                results: get_materials(&value["results"]),
                energy_required: value["energy_required"].as_f64().or(Some(0.5)).unwrap(),
                allowed_effects: {
                    let mut effects: Vec<String> = vec![];
                    if value["allow_consumption"]
                        .as_bool()
                        .or(Some(false))
                        .unwrap()
                    {
                        effects.push("consumption".into());
                    }
                    if value["allow_speed"].as_bool().or(Some(false)).unwrap() {
                        effects.push("speed".into());
                    }
                    if value["allow_productivity"]
                        .as_bool()
                        .or(Some(false))
                        .unwrap()
                    {
                        effects.push("productivity".into());
                    }
                    if value["allow_quality"].as_bool().or(Some(false)).unwrap() {
                        effects.push("quality".into());
                    }
                    effects
                },
            },
        );

        // let process = Process::Recipe {
        //     name: name.into(),
        //     productivity: 0.0,
        // };
        // registry.processes.push(process);
    }

    for (name, value) in parsed["module"].entries() {
        registry.modules.insert(
            name.into(),
            ModulePrototype {
                name: value["name"].as_str().unwrap().into(),
                category: value["category"].as_str().unwrap().into(),
                effects: get_effects(&value["effect"]),
            },
        );
    }

    for (name, value) in parsed["beacon"].entries() {
        registry.beacons.insert(
            name.into(),
            BeaconPrototype {
                name: value["name"].as_str().unwrap().into(),
                energy_source: get_energy_source(&value["energy_source"]),
                energy_usage: get_energy(value["energy_usage"].as_str().unwrap().into()),
                efficiency: value["distribution_effectivity"].as_f64().unwrap(),
                efficiency_per_quality: value["distribution_effectivity_bonus_per_quality_level"]
                    .as_f64()
                    .unwrap_or(0.0),
                module_slots: value["module_slots"].as_u16().unwrap(),
                allowed_effects: value["allowed_effects"]
                    .members()
                    .map(|string| string.as_str().unwrap().into())
                    .collect(),
                allowed_module_categories: value["allowed_module_categories"]
                    .members()
                    .map(|string| string.as_str().unwrap().into())
                    .collect(),
                profile: if value["profile"].is_array() {
                    Some(
                        value["profile"]
                            .members()
                            .map(|value| value.as_f64().unwrap())
                            .collect(),
                    )
                } else {
                    None
                },
                beacon_counter: value["beacon_counter"].as_str().map(|string| string.into()),
            },
        );
    }

    Ok(registry)
}

fn get_minable(value: &JsonValue) -> Minable {
    Minable {
        mining_time: value["mining_time"].as_f64().unwrap(),
        results: value["result"].as_str().map_or_else(
            || get_materials(&value["results"]),
            |result| {
                vec![Material::Item(Item {
                    name: result.into(),
                    quality: Some(0),
                    amount: value["count"].as_u16().or(1.into()),
                    amount_min: None,
                    amount_max: None,
                    probability: None,
                    ignored_by_productivity: None,
                    extra_count_fraction: None,
                })]
            },
        ),
        input_fluid: if !value["required_fluid"].is_empty() {
            Some(Fluid {
                name: value["required_fluid"].as_str().unwrap().into(),
                temperature: None,
                amount: value["fluid_amount"].as_f64(),
                amount_min: None,
                amount_max: None,
                probability: None,
                ignored_by_productivity: None,
            })
        } else {
            None
        },
    }
}
fn get_energy_source(value: &JsonValue) -> EnergySource {
    match value["type"].as_str().unwrap() {
        "electric" => EnergySource::Electric {
            drain: get_energy(value["drain"].as_str().or(Some("0J")).unwrap().into()),
        },
        "burner" => EnergySource::Burner {
            effectivity: value["effectivity"].as_f64().unwrap(),
            fuel_categories: value["fuel_categories"]
                .members()
                .map(|string| string.as_str().unwrap().into())
                .collect(),
        },
        "heat" => EnergySource::Heat,
        "fluid" => EnergySource::Fluid {
            effectivity: value["effectivity"].as_f64().unwrap(),
            burns_fluid: value["burns_fluid"].as_bool().unwrap(),
            fluid_usage_per_tick: value["fluid_usage_per_tick"].as_u32().unwrap(),
            scale_fluid_usage: value["scale_fluid_usage"].as_bool().unwrap(),
        },
        "void" => EnergySource::Void,
        &_ => {
            panic!("Unknown type: {}", value["type"].as_str().unwrap());
        }
    }
}
fn get_energy(value: String) -> u32 {
    let mut multiplier: u32 = 1;
    match &value[value.len() - 1..] {
        "J" => {
            multiplier *= 60;
        }
        "W" => {}
        _ => panic!(),
    }

    match &value[value.len() - 2..value.len() - 1] {
        "k" => multiplier *= 10 ^ 3,
        "M" => multiplier *= 10 ^ 6,
        "G" => multiplier *= 10 ^ 9,
        "T" => multiplier *= 10 ^ 12,
        _ => return multiplier * value[..value.len() - 1].parse::<u32>().unwrap(),
    }
    (multiplier as f64 * value[..value.len() - 2].parse::<f64>().unwrap()) as u32
}
fn get_effect_receiver(value: &JsonValue) -> Option<EffectReceiver> {
    if value.is_null() {
        return None;
    }
    Some(EffectReceiver {
        base_effect: if value.has_key("base_effect") {
            Some(get_effects(&value["base_effect"]))
        } else {
            None
        },
        uses_module_effects: value["uses_module_effects"]
            .as_bool()
            .or(Some(false))
            .unwrap(),
        uses_beacon_effects: value["uses_beacon_effects"]
            .as_bool()
            .or(Some(false))
            .unwrap(),
        uses_surface_effects: value["uses_surface_effects"]
            .as_bool()
            .or(Some(false))
            .unwrap(),
    })
}

fn get_effects(value: &JsonValue) -> Effects {
    Effects {
        consumption: value["consumption"].as_f32(),
        speed: value["speed"].as_f32(),
        productivity: value["productivity"].as_f32(),
        quality: value["quality"].as_f32(),
    }
}
fn get_materials(value: &JsonValue) -> Vec<Material> {
    value
        .members()
        .map(|result| match result["type"].as_str().unwrap() {
            "item" => {
                let mut item = Item {
                    name: result["name"].as_str().unwrap().into(),
                    quality: Some(0),
                    amount: result["amount"].as_u16(),
                    amount_min: result["amount_min"].as_u16(),
                    amount_max: result["amount_max"].as_u16(),
                    probability: result["probability"].as_f64(),
                    ignored_by_productivity: result["ignored_by_productivity"].as_u16(),
                    extra_count_fraction: result["extra_count_fraction"].as_f32(),
                };
                if item.amount.is_none() && item.amount_min.is_none() {
                    item.amount = 1.into();
                }
                Material::Item(item)
            }
            "fluid" => Material::Fluid(Fluid {
                name: result["name"].as_str().unwrap().into(),
                temperature: result["extra_count_fraction"].as_f32(),
                amount: result["amount"].as_f64(),
                amount_min: result["amount_min"].as_f64(),
                amount_max: result["amount_max"].as_f64(),
                probability: result["probability"].as_f64(),
                ignored_by_productivity: result["ignored_by_productivity"].as_u16(),
            }),
            &_ => panic!("Unknown type: {}", result["type"].as_str().unwrap()),
        })
        .collect()
}
#[cfg(test)]
mod tests {
    use crate::data::data_loader::load_data;

    #[test]
    fn test() {
        let registry =
            load_data("E:/Games/Factorio/script-output/data-raw-dump.json".into()).unwrap();
        println!(
            "{:#?}",
            registry
                .crafting_machines
                .iter()
                .map(|(_name, prototype)| { format!("{:#?}", prototype) })
                .collect::<Vec<_>>()
        );
    }
}

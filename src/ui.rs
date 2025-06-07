use crate::data::data_loader::load_data;
use crate::data::effects::EffectReceiver;
use crate::data::materials::MaterialPrototype;
use crate::data::{get_registry, set_registry, Process, ProcessType, Registry};
use crate::model::{Model, ModelResult};
use native_dialog::DialogBuilder;
use ratatui::crossterm::event::KeyModifiers;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{prelude::*, widgets::*};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;
use std::{fs, iter, panic};

#[derive(Serialize, Deserialize)]
struct SaveData {
    registry: Registry,
    model: Model,
    process_data: HashMap<(ProcessType, String), ProcessData>,
}

#[derive(Serialize, Deserialize, Clone)]
struct Beacon {
    prototype: String,
    count: u16,
    modules: Vec<String>,
}
#[derive(Clone, Serialize, Deserialize)]
enum ProcessData {
    Resource {
        mining_drill: Option<String>,
        modules: Vec<String>,
        beacons: HashMap<String, Beacon>,
    },
    Recipe {
        crafting_machine: Option<String>,
        modules: Vec<String>,
        beacons: HashMap<String, Beacon>,
    },
}

struct Menu {
    title: String,
    items: Vec<String>,
    handle_click: fn(&mut App, String),
    update_menu: Option<fn(&mut App) -> Menu>,
}

struct App {
    menu_stack: Vec<(Menu, usize)>,
    message: Option<String>,
    should_quit: bool,
    save_path: Option<PathBuf>,
    scroll_key: Option<KeyCode>,
    scroll_start_time: Option<std::time::Instant>,
    last_scroll_time: Option<std::time::Instant>,
    model: Model,
    process_data: HashMap<(ProcessType, String), ProcessData>,
    selected_process: Option<String>,
    fill_modules: bool,
    selected_beacon: Option<String>,
    selected_material: Option<(MaterialPrototype, f64)>,
    number_input: Option<(String, String)>,
    search_query: Option<String>,
    message_scroll: usize,
}

impl App {
    fn new() -> Self {
        let main_menu = Menu {
            title: "Main Menu".to_string(),
            items: vec![
                "Select data file".into(),
                "Load Project".into(),
                "Exit".into(),
            ],
            handle_click: |app: &mut App, name: String| match name.as_str() {
                "Select data file" => {
                    match DialogBuilder::file()
                        .add_filter("Json file", &["json"])
                        .open_single_file()
                        .show()
                        .unwrap()
                    {
                        Some(file) => match load_data(file.to_str().unwrap().to_string()) {
                            Ok(registry) => {
                                set_registry(registry);
                                app.set_message("Loaded data file.");
                                let menu = app.get_project_menu();
                                app.menu_stack.push((menu, 0));
                            }
                            Err(_) => {
                                app.set_message("Failed to load file!");
                            }
                        },
                        None => {
                            app.set_message("No file selected!");
                        }
                    }
                }
                "Load Project" => {
                    app.load_project();
                }
                "Exit" => {
                    app.should_quit = true;
                }
                &_ => {}
            },
            update_menu: None,
        };
        App {
            menu_stack: vec![(main_menu, 0)],
            message: None,
            should_quit: false,
            save_path: None,
            scroll_key: None,
            scroll_start_time: None,
            last_scroll_time: None,
            model: Model::default(),
            selected_process: None,
            process_data: Default::default(),
            fill_modules: false,
            selected_beacon: None,
            selected_material: None,
            number_input: None,
            search_query: None,
            message_scroll: 0,
        }
    }
    fn get_project_menu(&mut self) -> Menu {
        Menu {
            title: "Project".into(),
            items: vec![
                "Processes".into(),
                "Add Process".into(),
                "Outputs".into(),
                "Inputs".into(),
                // "Auto Inputs".into(),
                "Solve Model".into(),
                "Save Project".into(),
            ],
            handle_click: |app: &mut App, name: String| match name.as_str() {
                "Processes" => {
                    let menu = app.get_processes_menu();
                    app.menu_stack.push((menu, 0));
                }
                "Add Process" => {
                    let menu = app.get_new_process_menu(None);
                    app.menu_stack.push((menu, 0));
                }
                "Outputs" => {
                    let menu = app.get_outputs_menu();
                    app.menu_stack.push((menu, 0));
                }
                "Inputs" => {
                    let menu = app.get_inputs_menu();
                    app.menu_stack.push((menu, 0));
                }
                "Solve Model" => {
                    app.solve_model();
                }
                "Save Project" => {
                    app.save_project(true);
                }
                _ => panic!(),
            },
            update_menu: Some(|app: &mut App| app.get_project_menu()),
        }
    }
    //noinspection DuplicatedCode
    fn get_new_process_menu(&mut self, process_type: Option<ProcessType>) -> Menu {
        match process_type {
            Some(process_type) => {
                let registry = get_registry();
                match process_type {
                    ProcessType::Resource => Menu {
                        title: "Add Resource".into(),
                        items: registry
                            .resources
                            .keys()
                            .cloned()
                            .filter(|name| self.get_process_from_name(name).is_none())
                            .collect(),
                        handle_click: |app: &mut App, name: String| {
                            app.model.processes.push(Process {
                                process_type: ProcessType::Resource,
                                name: name.clone(),
                                productivity: 0.0,
                            });
                            app.process_data.insert(
                                (ProcessType::Resource, name),
                                ProcessData::Resource {
                                    mining_drill: None,
                                    modules: vec![],
                                    beacons: Default::default(),
                                },
                            );
                            app.menu_stack.pop();
                            app.menu_stack.pop();
                        },
                        update_menu: Some(|app: &mut App| {
                            app.get_new_process_menu(Some(ProcessType::Resource))
                        }),
                    },
                    ProcessType::Plant => Menu {
                        title: "Add Plant".into(),
                        items: registry
                            .plants
                            .keys()
                            .cloned()
                            .filter(|name| self.get_process_from_name(name).is_none())
                            .collect(),
                        handle_click: |app: &mut App, name: String| {
                            app.model.processes.push(Process {
                                process_type: ProcessType::Plant,
                                name,
                                productivity: 0.0,
                            });
                            app.menu_stack.pop();
                            app.menu_stack.pop();
                        },
                        update_menu: Some(|app: &mut App| {
                            app.get_new_process_menu(Some(ProcessType::Plant))
                        }),
                    },
                    ProcessType::Recipe => Menu {
                        title: "Add Recipe".into(),
                        items: registry
                            .recipes
                            .keys()
                            .cloned()
                            .filter(|name| self.get_process_from_name(name).is_none())
                            .collect(),
                        handle_click: |app: &mut App, name: String| {
                            app.model.processes.push(Process {
                                process_type: ProcessType::Recipe,
                                name: name.clone(),
                                productivity: 0.0,
                            });
                            app.process_data.insert(
                                (ProcessType::Recipe, name),
                                ProcessData::Recipe {
                                    crafting_machine: None,
                                    modules: vec![],
                                    beacons: Default::default(),
                                },
                            );
                            app.menu_stack.pop();
                            app.menu_stack.pop();
                        },
                        update_menu: Some(|app: &mut App| {
                            app.get_new_process_menu(Some(ProcessType::Recipe))
                        }),
                    },
                }
            }
            None => Menu {
                title: "Add Process".into(),
                items: vec!["Resource".into(), "Plant".into(), "Recipe".into()],
                handle_click: |app: &mut App, name: String| {
                    let menu = app.get_new_process_menu(Some(match name.as_str() {
                        "Resource" => ProcessType::Resource,
                        "Plant" => ProcessType::Plant,
                        "Recipe" => ProcessType::Recipe,
                        _ => panic!(),
                    }));
                    app.menu_stack.push((menu, 0));
                },
                update_menu: Some(|app: &mut App| app.get_new_process_menu(None)),
            },
        }
    }
    fn get_processes_menu(&mut self) -> Menu {
        Menu {
            title: "Processes".into(),
            items: self
                .model
                .processes
                .iter()
                .map(|process| Into::<String>::into(&process.process_type) + " :" + &*process.name)
                .collect(),
            handle_click: |app: &mut App, name: String| {
                app.selected_process = name.into();
                let menu = app.get_process_menu();
                app.menu_stack.push((menu, 0));
            },
            update_menu: Some(|app: &mut App| app.get_processes_menu()),
        }
    }
    fn get_process_from_name(&mut self, name: &String) -> Option<&mut Process> {
        self.model.processes.iter_mut().find(|process| {
            (Into::<String>::into(&process.process_type) + " :" + &*process.name).as_str()
                == name.as_str()
        })
    }
    fn get_selected_process(&mut self) -> &mut Process {
        self.get_process_from_name(&self.selected_process.as_ref().unwrap().clone())
            .unwrap()
    }
    fn get_selected_process_data(&mut self) -> &mut ProcessData {
        let process_type = self.get_selected_process().process_type;
        let name = self.get_selected_process().name.clone();
        self.process_data.get_mut(&(process_type, name)).unwrap()
    }
    fn get_process_menu(&mut self) -> Menu {
        if match self.get_selected_process_data() {
            ProcessData::Resource {
                mining_drill,
                modules: _modules,
                beacons: _beacons,
            } => mining_drill.is_some(),
            ProcessData::Recipe {
                crafting_machine,
                modules: _modules,
                beacons: _beacons,
            } => crafting_machine.is_some(),
        } {
            Menu {
                title: self.selected_process.as_ref().unwrap().clone(),
                items: if self.get_selected_process().process_type != ProcessType::Plant {
                    vec![
                        "Machine".into(),
                        "Modules".into(),
                        "Beacons".into(),
                        "Remove".into(),
                    ]
                } else {
                    vec!["Remove".into()]
                },
                handle_click: move |app: &mut App, name: String| match name.as_ref() {
                    "Machine" => {
                        let menu = app.get_machine_menu();
                        app.menu_stack.push((menu, 0));
                    }
                    "Modules" => {
                        let menu = app.get_modules_menu();
                        app.menu_stack.push((menu, 0));
                    }
                    "Beacons" => {
                        let menu = app.get_beacons_menu();
                        app.menu_stack.push((menu, 0));
                    }
                    "Remove" => {
                        app.menu_stack.pop();
                        app.model.processes.remove(
                            app.model
                                .processes
                                .iter()
                                .position(|process| {
                                    (Into::<String>::into(&process.process_type)
                                        + " :"
                                        + &*process.name)
                                        == *app.selected_process.as_ref().unwrap()
                                })
                                .unwrap(),
                        );
                    }
                    _ => panic!(),
                },
                update_menu: Some(|app: &mut App| app.get_process_menu()),
            }
        } else {
            self.get_machine_menu()
        }
    }
    fn get_machine_menu(&mut self) -> Menu {
        Menu {
            title: "Select Machine".into(),
            items: {
                let registry = get_registry();
                let process = self.get_selected_process();
                match process.process_type {
                    ProcessType::Resource => registry
                        .mining_drills
                        .iter()
                        .filter_map(|(drill_name, drill)| {
                            if drill
                                .resource_categories
                                .contains(&registry.resources.get(&process.name).unwrap().category)
                            {
                                drill_name.clone().into()
                            } else {
                                None
                            }
                        })
                        .collect(),
                    ProcessType::Plant => panic!(),
                    ProcessType::Recipe => registry
                        .crafting_machines
                        .iter()
                        .filter_map(|(machine_name, machine)| {
                            if machine
                                .crafting_categories
                                .contains(&registry.recipes.get(&process.name).unwrap().category)
                            {
                                machine_name.clone().into()
                            } else {
                                None
                            }
                        })
                        .collect(),
                }
            },
            handle_click: |app: &mut App, name: String| {
                let process_name = app.get_selected_process().name.clone();
                match app.get_selected_process().process_type {
                    ProcessType::Resource => {
                        let ProcessData::Resource {
                            mining_drill,
                            modules: _modules,
                            beacons: _beacons,
                        } = app
                            .process_data
                            .get_mut(&(ProcessType::Resource, process_name))
                            .unwrap()
                        else {
                            panic!()
                        };
                        *mining_drill = Option::from(name);
                    }
                    ProcessType::Plant => {
                        panic!()
                    }
                    ProcessType::Recipe => {
                        let ProcessData::Recipe {
                            crafting_machine,
                            modules: _modules,
                            beacons: _beacons,
                        } = app
                            .process_data
                            .get_mut(&(ProcessType::Recipe, process_name))
                            .unwrap()
                        else {
                            panic!()
                        };
                        *crafting_machine = Option::from(name);
                    }
                }
                app.menu_stack.pop();
            },
            update_menu: Some(|app: &mut App| app.get_machine_menu()),
        }
    }
    fn get_modules_menu(&mut self) -> Menu {
        Menu {
            title: "Modules (select to remove)".into(),
            items: {
                let registry = get_registry();
                match self.selected_beacon.clone() {
                    Some(beacon_name) => {
                        let modules = &self.get_selected_beacon().modules;
                        if modules.len()
                            < registry.beacons.get(&beacon_name).unwrap().module_slots as usize
                        {
                            vec!["Add Module".into(), "Fill Modules".into()]
                        } else {
                            Vec::<String>::new()
                        }
                        .into_iter()
                        .chain(modules.clone())
                        .collect()
                    }
                    None => match self.get_selected_process_data() {
                        ProcessData::Resource {
                            mining_drill,
                            modules,
                            beacons: _beacons,
                        } => if modules.len()
                            < registry
                                .mining_drills
                                .get(mining_drill.as_ref().unwrap())
                                .unwrap()
                                .module_slots as usize
                        {
                            vec!["Add Module".into(), "Fill Modules".into()]
                        } else {
                            Vec::<String>::new()
                        }
                        .into_iter()
                        .chain(modules.clone())
                        .collect(),
                        ProcessData::Recipe {
                            crafting_machine,
                            modules,
                            beacons: _beacons,
                        } => if modules.len()
                            < registry
                                .crafting_machines
                                .get(crafting_machine.as_ref().unwrap())
                                .unwrap()
                                .module_slots as usize
                        {
                            vec!["Add Module".into(), "Fill Modules".into()]
                        } else {
                            Vec::<String>::new()
                        }
                        .into_iter()
                        .chain(modules.clone())
                        .collect(),
                    },
                }
            },
            handle_click: |app: &mut App, name: String| match name.as_str() {
                "Add Module" => {
                    app.fill_modules = false;
                    let menu = app.get_add_module_menu();
                    app.menu_stack.push((menu, 0));
                }
                "Fill Modules" => {
                    app.fill_modules = true;
                    let menu = app.get_add_module_menu();
                    app.menu_stack.push((menu, 0));
                }
                _ => match app.selected_beacon.clone() {
                    Some(_) => {
                        let modules = &mut app.get_selected_beacon().modules;
                        modules
                            .iter()
                            .position(|string| *string == name)
                            .map(|index| modules.remove(index));
                    }
                    None => match app.get_selected_process_data() {
                        ProcessData::Resource {
                            mining_drill: _mining_drill,
                            modules,
                            beacons: _beacons,
                        } => {
                            modules
                                .iter()
                                .position(|string| *string == name)
                                .map(|index| modules.remove(index));
                        }
                        ProcessData::Recipe {
                            crafting_machine: _crafting_machine,
                            modules,
                            beacons: _beacons,
                        } => {
                            modules
                                .iter()
                                .position(|string| *string == name)
                                .map(|index| modules.remove(index));
                        }
                    },
                },
            },
            update_menu: Some(|app: &mut App| app.get_modules_menu()),
        }
    }
    fn get_add_module_menu(&mut self) -> Menu {
        Menu {
            title: "Add Module".into(),
            items: {
                let registry = get_registry();
                match self.selected_beacon.as_ref() {
                    Some(beacon_name) => registry
                        .modules
                        .iter()
                        .filter_map(|(name, module)| {
                            let categories = &registry
                                .beacons
                                .get(beacon_name)
                                .unwrap()
                                .allowed_module_categories;
                            if categories.is_empty() || categories.contains(&module.category) {
                                Some(name.clone())
                            } else {
                                None
                            }
                        })
                        .collect(),
                    None => {
                        let (process_name, process_type) = {
                            let process = self.get_selected_process();
                            (process.name.clone(), process.process_type)
                        };
                        match process_type {
                            ProcessType::Resource => {
                                let ProcessData::Resource {
                                    mining_drill,
                                    modules: _modules,
                                    beacons: _beacons,
                                } = self
                                    .process_data
                                    .get_mut(&(ProcessType::Resource, process_name))
                                    .unwrap()
                                else {
                                    panic!()
                                };
                                registry
                                    .modules
                                    .iter()
                                    .filter_map(|(name, module)| {
                                        let categories = &registry
                                            .mining_drills
                                            .get(mining_drill.as_ref().unwrap())
                                            .unwrap()
                                            .allowed_module_categories;
                                        if categories.is_empty()
                                            || categories.contains(&module.category)
                                        {
                                            Some(name.clone())
                                        } else {
                                            None
                                        }
                                    })
                                    .collect()
                            }
                            ProcessType::Plant => panic!(),
                            ProcessType::Recipe => {
                                let ProcessData::Recipe {
                                    ref crafting_machine,
                                    modules: _,
                                    beacons: _,
                                } = self
                                    .process_data
                                    .get_mut(&(ProcessType::Recipe, process_name))
                                    .unwrap()
                                else {
                                    panic!()
                                };
                                registry
                                    .modules
                                    .iter()
                                    .filter_map(|(name, module)| {
                                        let categories = &registry
                                            .crafting_machines
                                            .get(crafting_machine.as_ref().unwrap())
                                            .unwrap()
                                            .allowed_module_categories;
                                        if categories.is_empty()
                                            || categories.contains(&module.category)
                                        {
                                            Some(name.clone())
                                        } else {
                                            None
                                        }
                                    })
                                    .collect()
                            }
                        }
                    }
                }
            },
            handle_click: |app: &mut App, name: String| {
                let fill = app.fill_modules;
                let max_modules: u16;
                let registry = get_registry();
                let modules = match app.selected_beacon.as_ref() {
                    Some(beacon_name) => {
                        max_modules = registry.beacons.get(beacon_name).unwrap().module_slots;
                        &mut app.get_selected_beacon().modules
                    }
                    None => match app.get_selected_process_data() {
                        ProcessData::Resource {
                            mining_drill,
                            modules,
                            beacons: _beacons,
                        } => {
                            max_modules = registry
                                .mining_drills
                                .get(mining_drill.as_ref().unwrap())
                                .unwrap()
                                .module_slots;
                            modules
                        }
                        ProcessData::Recipe {
                            crafting_machine,
                            modules,
                            beacons: _beacons,
                        } => {
                            max_modules = registry
                                .crafting_machines
                                .get(crafting_machine.as_ref().unwrap())
                                .unwrap()
                                .module_slots;
                            modules
                        }
                    },
                };
                for _ in 0..(if fill {
                    max_modules - modules.len() as u16
                } else {
                    1
                }) {
                    modules.push(name.clone());
                }
                app.menu_stack.pop();
            },
            update_menu: Some(|app: &mut App| app.get_add_module_menu()),
        }
    }
    fn get_selected_beacon(&mut self) -> &mut Beacon {
        let beacon_name = self.selected_beacon.as_ref().unwrap().clone();
        match self.get_selected_process_data() {
            ProcessData::Resource {
                mining_drill: _mining_drill,
                modules: _modules,
                beacons,
            } => beacons,
            ProcessData::Recipe {
                crafting_machine: _crafting_machine,
                modules: _modules,
                beacons,
            } => beacons,
        }
        .get_mut(&beacon_name)
        .unwrap()
    }
    fn get_beacons_menu(&mut self) -> Menu {
        Menu {
            title: "Beacons".into(),
            items: vec!["Add Beacon".into()]
                .into_iter()
                .chain(match self.get_selected_process_data() {
                    ProcessData::Resource {
                        mining_drill: _mining_drill,
                        modules: _modules,
                        beacons,
                    } => beacons
                        .iter()
                        .map(|(_name, beacon)| beacon.prototype.clone())
                        .collect::<Vec<String>>(),
                    ProcessData::Recipe {
                        crafting_machine: _crafting_machine,
                        modules: _modules,
                        beacons,
                    } => beacons
                        .iter()
                        .map(|(_name, beacon)| beacon.prototype.clone())
                        .collect::<Vec<String>>(),
                })
                .collect(),
            handle_click: |app: &mut App, name: String| {
                if name == "Add Beacon" {
                    let menu = app.get_add_beacon_menu();
                    app.menu_stack.push((menu, 0))
                } else {
                    app.selected_beacon = Some(name);
                    let menu = app.get_beacon_menu();
                    app.menu_stack.push((menu, 0));
                }
            },
            update_menu: Some(|app: &mut App| app.get_beacons_menu()),
        }
    }
    fn get_beacon_menu(&mut self) -> Menu {
        Menu {
            title: "Beacon".into(),
            items: vec!["Modules".into(), "Count".into(), "Remove".into()],
            handle_click: |app: &mut App, name: String| match name.as_str() {
                "Modules" => {
                    let menu = app.get_modules_menu();
                    app.menu_stack.push((menu, 0));
                }
                "Count" => {
                    let current_count = app.get_selected_beacon().count.to_string();
                    app.number_input = Some(("beacon_count".into(), current_count));
                }
                "Remove" => {
                    let beacon = app.get_selected_beacon().prototype.clone();
                    match app.get_selected_process_data() {
                        ProcessData::Resource {
                            mining_drill: _mining_drill,
                            modules: _modules,
                            beacons,
                        } => beacons,
                        ProcessData::Recipe {
                            crafting_machine: _crafting_machine,
                            modules: _modules,
                            beacons,
                        } => beacons,
                    }
                    .remove(&beacon);
                    app.selected_beacon = None;
                    app.menu_stack.pop();
                }
                _ => {
                    panic!()
                }
            },
            // ...
            update_menu: Some(|app| app.get_beacon_menu()),
        }
    }
    fn get_add_beacon_menu(&mut self) -> Menu {
        Menu {
            title: "Add Beacon".into(),
            items: {
                let registry = get_registry();
                registry
                    .beacons
                    .iter()
                    .filter_map(|(name, _beacon)| {
                        if match self.get_selected_process_data() {
                            ProcessData::Resource {
                                mining_drill: _mining_drill,
                                modules: _modules,
                                beacons,
                            } => beacons.contains_key(name),
                            ProcessData::Recipe {
                                crafting_machine: _crafting_machine,
                                modules: _modules,
                                beacons,
                            } => beacons.contains_key(name),
                        } {
                            None
                        } else {
                            Some(name.clone())
                        }
                    })
                    .collect()
            },
            handle_click: |app: &mut App, name: String| {
                match app.get_selected_process_data() {
                    ProcessData::Resource {
                        mining_drill: _mining_drill,
                        modules: _modules,
                        beacons,
                    } => beacons,
                    ProcessData::Recipe {
                        crafting_machine: _crafting_machine,
                        modules: _modules,
                        beacons,
                    } => beacons,
                }
                .insert(
                    name.clone(),
                    Beacon {
                        prototype: name,
                        count: 1,
                        modules: vec![],
                    },
                );
                app.menu_stack.pop();
            },
            update_menu: Some(|app: &mut App| app.get_add_beacon_menu()),
        }
    }
    fn get_outputs_menu(&mut self) -> Menu {
        Menu {
            title: "Outputs".into(),
            items: vec!["Add Output".into()]
                .into_iter()
                .chain(self.model.outputs.iter().map(|(prototype, amount)| {
                    prototype.to_id().clone() + ": " + &*amount.to_string()
                }))
                .collect(),
            handle_click: |app: &mut App, name: String| {
                if name == "Add Output" {
                    let menu = app.get_new_output_menu();
                    app.menu_stack.push((menu, 0));
                } else {
                    let (material, amount) = name.split_once(": ").unwrap();
                    let amount = amount.parse::<f64>().unwrap();
                    app.selected_material =
                        Some((app.model.get_output(material).unwrap().0, amount));
                    let menu = app.get_output_menu();
                    app.menu_stack.push((menu, 0));
                }
            },
            update_menu: Some(|app: &mut App| app.get_outputs_menu()),
        }
    }
    fn get_inputs_menu(&mut self) -> Menu {
        Menu {
            title: "Inputs".into(),
            items: vec!["Add Input".into()]
                .into_iter()
                .chain(self.model.inputs.iter().map(|(prototype, amount)| {
                    prototype.to_id().clone() + ": " + &*amount.to_string()
                }))
                .collect(),
            handle_click: |app: &mut App, name: String| {
                if name == "Add Input" {
                    let menu = app.get_new_input_menu();
                    app.menu_stack.push((menu, 0));
                } else {
                    let (material, amount) = name.split_once(": ").unwrap();
                    let amount = amount.parse::<f64>().unwrap();
                    app.selected_material =
                        Some((app.model.get_input(material).unwrap().0, amount));
                    let menu = app.get_input_menu();
                    app.menu_stack.push((menu, 0));
                }
            },
            update_menu: Some(|app: &mut App| app.get_inputs_menu()),
        }
    }
    fn get_output_menu(&mut self) -> Menu {
        let (material, _current_amount) = self.selected_material.as_ref().unwrap();
        Menu {
            title: material.to_id(),
            items: vec!["Edit Amount".into(), "Remove".into()],
            handle_click: |app: &mut App, name: String| {
                let (material, current_amount) = app.selected_material.as_ref().unwrap();
                let current_amount = current_amount.to_string();
                match name.as_str() {
                    "Edit Amount" => {
                        app.number_input = Some(("output_amount".into(), current_amount.clone()));
                    }
                    "Remove" => {
                        app.model.outputs.remove(material);
                        app.menu_stack.pop();
                    }
                    _ => {}
                }
            },
            update_menu: Some(|app: &mut App| app.get_output_menu()),
        }
    }
    fn get_input_menu(&mut self) -> Menu {
        let (material, _current_amount) = self.selected_material.as_ref().unwrap();
        Menu {
            title: material.to_id(),
            items: vec!["Edit Amount".into(), "Remove".into()],
            handle_click: |app: &mut App, name: String| {
                let (material, current_amount) = app.selected_material.as_ref().unwrap();
                let current_amount = current_amount.to_string();
                match name.as_str() {
                    "Edit Amount" => {
                        app.number_input = Some(("input_amount".into(), current_amount.clone()));
                    }
                    "Remove" => {
                        app.model.inputs.remove(material);
                        app.menu_stack.pop();
                    }
                    _ => {}
                }
            },
            update_menu: Some(|app: &mut App| app.get_input_menu()),
        }
    }
    //noinspection DuplicatedCode
    fn get_new_output_menu(&mut self) -> Menu {
        Menu {
            title: "New Output".into(),
            items: {
                let registry = get_registry();
                iter::empty()
                    .chain(
                        registry
                            .items
                            .iter()
                            .map(|(name, _prototype)| MaterialPrototype::Item(name.clone())),
                    )
                    .chain(
                        registry
                            .fluids
                            .iter()
                            .map(|(name, _prototype)| MaterialPrototype::Fluid(name.clone())),
                    )
                    .filter_map(|prototype| match self.model.outputs.get(&prototype) {
                        Some(_) => None,
                        None => Some(prototype.to_id()),
                    })
                    .collect()
            },

            handle_click: |app: &mut App, name: String| {
                app.model
                    .outputs
                    .insert(MaterialPrototype::from_id(&name).unwrap(), 0.0);
                app.menu_stack.pop();
            },
            update_menu: Some(|app: &mut App| app.get_new_output_menu()),
        }
    }
    //noinspection DuplicatedCode
    fn get_new_input_menu(&mut self) -> Menu {
        Menu {
            title: "New Input".into(),
            items: {
                let registry = get_registry();
                iter::empty()
                    .chain(
                        registry
                            .items
                            .iter()
                            .map(|(name, _prototype)| MaterialPrototype::Item(name.clone())),
                    )
                    .chain(
                        registry
                            .fluids
                            .iter()
                            .map(|(name, _prototype)| MaterialPrototype::Fluid(name.clone())),
                    )
                    .filter_map(|prototype| match self.model.inputs.get(&prototype) {
                        Some(_) => None,
                        None => Some(prototype.to_id()),
                    })
                    .collect()
            },

            handle_click: |app: &mut App, name: String| {
                app.model
                    .inputs
                    .insert(MaterialPrototype::from_id(&name).unwrap(), 0.0);
                app.menu_stack.pop();
            },
            update_menu: Some(|app: &mut App| app.get_new_input_menu()),
        }
    }

    fn get_effect_receiver(
        process_data: &ProcessData,
    ) -> (EffectReceiver, &Vec<String>, &HashMap<String, Beacon>) {
        let (effect_receiver, modules, beacons) = match process_data {
            ProcessData::Resource {
                mining_drill,
                modules,
                beacons,
            } => (
                get_registry()
                    .mining_drills
                    .get(mining_drill.as_ref().unwrap())
                    .unwrap()
                    .effect_receiver
                    .unwrap_or_default(),
                modules,
                beacons,
            ),
            ProcessData::Recipe {
                crafting_machine,
                modules,
                beacons,
            } => (
                get_registry()
                    .crafting_machines
                    .get(crafting_machine.as_ref().unwrap())
                    .unwrap()
                    .effect_receiver
                    .unwrap_or_default(),
                modules,
                beacons,
            ),
        };
        (effect_receiver, modules, beacons)
    }

    fn solve_model(&mut self) {
        let registry = get_registry();
        // update productivity values
        for process in self.model.processes.iter_mut() {
            let mut productivity = 0.0;
            let (effect_receiver, modules, beacons) = App::get_effect_receiver(
                self.process_data
                    .get(&(process.process_type, process.name.clone()))
                    .unwrap(),
            );
            if let Some(base_effect) = effect_receiver.base_effect {
                productivity += base_effect.productivity.unwrap_or(0.0);
            }
            if effect_receiver.uses_module_effects {
                for module in modules.iter() {
                    productivity += registry
                        .modules
                        .get(module)
                        .unwrap()
                        .effects
                        .productivity
                        .unwrap_or(0.0);
                }
            }
            if effect_receiver.uses_beacon_effects {
                for beacon in beacons.values() {
                    for module in beacon.modules.iter() {
                        productivity += registry
                            .modules
                            .get(module)
                            .unwrap()
                            .effects
                            .productivity
                            .unwrap_or(0.0);
                    }
                }
            }
            // if effect_receiver.uses_surface_effects {
            //     todo!()
            // }
            process.productivity = productivity;
        }
        let message: String = match self.model.solve(false) {
            ModelResult::NoSolution => "No Solution!".into(),
            ModelResult::OneSolution(solution) => {
                let registry = get_registry().clone();
                vec!["Solution:".to_string()]
                    .into_iter()
                    .chain(solution.iter().map(|(process, &amount)| {
                        let time: f64 = match process.process_type {
                            ProcessType::Resource => {
                                registry
                                    .resources
                                    .get(&process.name)
                                    .unwrap()
                                    .results
                                    .mining_time
                            }
                            ProcessType::Plant => {
                                let plant = registry.plants.get(&process.name).unwrap();
                                (plant.growth_ticks / 60) as f64 + plant.results.mining_time
                            }
                            ProcessType::Recipe => {
                                registry.recipes.get(&process.name).unwrap().energy_required
                            }
                        };
                        let speed: f64 = match process.process_type {
                            ProcessType::Resource => {
                                let ProcessData::Resource {
                                    mining_drill,
                                    modules: _modules,
                                    beacons: _beacons,
                                } = self
                                    .process_data
                                    .get(&(process.process_type, process.name.clone()))
                                    .unwrap()
                                else {
                                    panic!()
                                };
                                Some(
                                    registry
                                        .mining_drills
                                        .get(mining_drill.as_ref().unwrap())
                                        .unwrap()
                                        .mining_speed,
                                )
                            }
                            ProcessType::Plant => None,
                            ProcessType::Recipe => {
                                let ProcessData::Recipe {
                                    crafting_machine,
                                    modules: _modules,
                                    beacons: _beacons,
                                } = self
                                    .process_data
                                    .get(&(process.process_type, process.name.clone()))
                                    .unwrap()
                                else {
                                    panic!()
                                };
                                Some(
                                    registry
                                        .crafting_machines
                                        .get(crafting_machine.as_ref().unwrap())
                                        .unwrap()
                                        .crafting_speed,
                                )
                            }
                        }
                        .map_or(0.0, |mut speed| {
                            let (effect_receiver, modules, beacons) = App::get_effect_receiver(
                                self.process_data
                                    .get(&(process.process_type, process.name.clone()))
                                    .unwrap(),
                            );
                            if let Some(base_effect) = effect_receiver.base_effect {
                                speed += base_effect.speed.unwrap_or(0.0) as f64;
                            }
                            if effect_receiver.uses_module_effects {
                                for module in modules.iter() {
                                    speed += registry
                                        .modules
                                        .get(module)
                                        .unwrap()
                                        .effects
                                        .speed
                                        .unwrap_or(0.0)
                                        as f64
                                }
                            }
                            if effect_receiver.uses_beacon_effects {
                                for beacon in beacons.values() {
                                    for module in beacon.modules.iter() {
                                        speed += registry
                                            .modules
                                            .get(module)
                                            .unwrap()
                                            .effects
                                            .speed
                                            .unwrap_or(0.0)
                                            as f64
                                    }
                                }
                            }
                            speed
                        });
                        format!("{} : {}", process.name, amount * time / speed)
                    }))
                    .collect::<Vec<String>>()
                    .join("\n")
            }
            ModelResult::MultipleSolutions {
                lower_bounds,
                higher_bounds,
            } => vec!["Solution:".to_string()]
                .into_iter()
                .chain(lower_bounds.iter().map(|(material, amount)| {
                    format!("{} can be reduced to {}", material.to_id(), amount)
                }))
                .chain(higher_bounds.iter().map(|(material, amount)| {
                    format!("{} can be increased to {}", material.to_id(), amount)
                }))
                .collect::<Vec<String>>()
                .join("\n"),
        };
        self.set_message(message);
    }
    fn save_project(&mut self, force_prompt: bool) {
        let file = if !force_prompt && let Some(file) = self.save_path.clone() {
            file
        } else {
            let Some(file) = DialogBuilder::file()
                .add_filter("Project files", &["cfpr"])
                .save_single_file()
                .show()
                .unwrap()
            else {
                self.set_message("No file selected");
                return;
            };
            file
        };
        self.save_path = Some(file.clone());
        let save_data = SaveData {
            registry: (*get_registry()).clone(),
            model: self.model.clone(),
            process_data: self
                .process_data
                .iter()
                .map(|((process_type, name), value)| {
                    ((process_type.clone(), name.clone()), value.clone())
                })
                .collect(),
        };
        if let Err(e) = fs::write(file, rmp_serde::to_vec(&save_data).unwrap()) {
            self.set_message(format!("Failed to save data! Error: {}", e));
        } else {
            self.set_message("Successfully saved project file");
        }
    }
    fn load_project(&mut self) {
        match DialogBuilder::file()
            .add_filter("Project files", &["cfpr"])
            .open_single_file()
            .show()
            .unwrap()
        {
            Some(file) => match fs::read(file) {
                Ok(data) => match rmp_serde::from_slice::<SaveData>(&data) {
                    Ok(SaveData {
                        model,
                        registry,
                        process_data,
                    }) => {
                        set_registry(registry);
                        self.model = model;
                        self.process_data = process_data;
                        let menu = self.get_project_menu();
                        self.menu_stack.push((menu, 0));
                    }
                    Err(e) => {
                        self.set_message(format!("Failed to load data! Error: {}", e));
                    }
                },
                Err(e) => {
                    self.set_message(format!("Failed to load data! Error: {}", e));
                }
            },
            None => {
                self.set_message("No file selected");
            }
        }
    }

    fn set_message<S: Into<String>>(&mut self, message: S) {
        self.message = Some(message.into());
        self.message_scroll = 0;
    }
    fn current_menu(&self) -> Option<&Menu> {
        self.menu_stack.last().map(|(menu, _pos)| menu)
    }
    fn current_selected(&self) -> usize {
        self.menu_stack.last().unwrap().1
    }
    fn set_selected(&mut self, index: usize) {
        self.menu_stack.last_mut().unwrap().1 = index
    }
    fn handle_input(&mut self) -> Result<(), Box<dyn Error>> {
        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    // Check if in beacon count input mode
                    match key.code {
                        KeyCode::Char(c) if let Some(query) = self.search_query.as_mut() => {
                            query.push(c);
                        }
                        KeyCode::Char(c) if c.is_ascii_digit() || c == '.' => {
                            if let Some((_context, ref mut input)) = self.number_input.as_mut() {
                                input.push(c);
                            }
                        }
                        KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            self.save_project(false);
                        }
                        KeyCode::Backspace => {
                            if let Some((_context, ref mut input)) = self.number_input.as_mut() {
                                input.pop();
                            }
                            if let Some(query) = self.search_query.as_mut() {
                                query.pop();
                            }
                        }
                        KeyCode::Esc => {
                            if self.search_query.is_some() || self.number_input.is_some() {
                                self.search_query = None;
                                self.number_input = None;
                            } else {
                                self.menu_stack.pop();
                            }
                        }
                        KeyCode::Up | KeyCode::Char('w') => {
                            if self.message.is_none() {
                                let selected = self.current_selected();
                                if selected > 0 {
                                    self.set_selected(selected - 1)
                                }
                            } else {
                                self.message_scroll = self.message_scroll.saturating_sub(1);
                            }
                        }
                        KeyCode::Down | KeyCode::Char('s') => {
                            if self.message.is_none() {
                                let selected = self.current_selected();
                                let menu = self.current_menu().unwrap();
                                let items = get_displayed_list(menu, self);
                                if items.len() > 0 && selected < items.len() - 1 {
                                    self.set_selected(selected + 1)
                                }
                            } else {
                                self.message_scroll = self.message_scroll.saturating_add(1);
                            }
                        }
                        KeyCode::Enter | KeyCode::Char(' ') => {
                            if let Some((context, input)) = self.number_input.take() {
                                match context.as_str() {
                                    "beacon_count" => {
                                        if let Ok(count) = input.parse::<u16>() {
                                            self.get_selected_beacon().count = count;
                                        }
                                    }
                                    "output_amount" => {
                                        if let Ok(amount) = input.parse::<f64>() {
                                            let (material, _) =
                                                self.selected_material.as_ref().unwrap().clone();
                                            self.model.outputs.insert(material, amount);
                                        }
                                    }
                                    "input_amount" => {
                                        if let Ok(amount) = input.parse::<f64>() {
                                            let (material, _) =
                                                self.selected_material.as_ref().unwrap().clone();
                                            self.model.inputs.insert(material, amount);
                                        }
                                    }
                                    _ => {}
                                }
                                self.number_input = None;
                                return Ok(());
                            }
                            if self.message.is_some() {
                                self.message = None;
                                self.scroll_key = None;
                            } else {
                                let (current_menu, selected) = self.menu_stack.last().unwrap();
                                let function = current_menu.handle_click.clone();
                                let items_to_display: Vec<&String> =
                                    get_displayed_list(current_menu, self);
                                if items_to_display.is_empty() {
                                    return Ok(());
                                }
                                if let Some(_query) = &self.search_query {
                                    self.search_query = None;
                                }
                                function(self, items_to_display[*selected].clone());
                            }
                        }
                        KeyCode::Char('f') | KeyCode::Char('F') => {
                            self.search_query = Some(String::new());
                        }
                        _ => {}
                    }
                    let Some(current_menu) = self.current_menu() else {
                        self.should_quit = true;
                        return Ok(());
                    };
                    if let Some(update_menu) = current_menu.update_menu.clone() {
                        let Some((_, pos)) = self.menu_stack.pop() else {
                            panic!()
                        };
                        let new_menu = update_menu(self);
                        self.menu_stack.push((new_menu, pos));
                    }
                    let mut length = get_displayed_list(self.current_menu().unwrap(), self).len();
                    if length == 0 {
                        length = 1;
                    }
                    let (_, ref mut pos) = self.menu_stack.last_mut().unwrap();
                    if *pos >= length {
                        *pos = length - 1
                    }
                    return Ok(());
                }

                if let Some(_) = &self.message {
                    match key.code {
                        KeyCode::Up | KeyCode::Down | KeyCode::PageUp | KeyCode::PageDown => {
                            // Start tracking key press
                            self.scroll_key = Some(key.code);
                            self.scroll_start_time = Some(std::time::Instant::now());
                            self.last_scroll_time = Some(std::time::Instant::now());

                            // Handle initial scroll
                            self.handle_scroll_key(key.code);
                        }
                        _ => {
                            // Reset scroll tracking for non-scroll keys
                            self.scroll_key = None;
                        }
                    }
                } else if key.kind == KeyEventKind::Release {
                    // Clear scroll tracking when key is released
                    if self.scroll_key == Some(key.code) {
                        self.scroll_key = None;
                    }
                }
            }
        }
        // Handle auto-repeat for held keys
        if let (Some(key), Some(start), Some(last)) = (
            self.scroll_key,
            self.scroll_start_time,
            self.last_scroll_time,
        ) {
            let now = std::time::Instant::now();
            let since_start = now.duration_since(start);
            let since_last = now.duration_since(last);

            if since_start > std::time::Duration::from_millis(500)
                && since_last > std::time::Duration::from_millis(50)
            {
                self.handle_scroll_key(key);
                self.last_scroll_time = Some(now);
            }
        }

        Ok(())
    }
    fn handle_scroll_key(&mut self, key: KeyCode) {
        if let Some(_message) = &self.message {
            match key {
                KeyCode::Up => self.message_scroll = self.message_scroll.saturating_sub(1),
                KeyCode::Down => self.message_scroll = self.message_scroll.saturating_add(1),
                _ => {}
            }
        }
    }
}

pub fn main() -> Result<(), Box<dyn Error>> {
    let default_hook = panic::take_hook();

    // Set a custom panic hook
    panic::set_hook(Box::new(move |panic_info| {
        // Custom code to run on panic (e.g., logging, cleanup)
        restore_terminal().expect("TODO: panic message");

        // Call the default hook to print the panic message
        default_hook(panic_info);
    }));

    let mut terminal = setup_terminal()?;
    let mut app = App::new();

    while !app.should_quit {
        terminal.draw(|f| ui(f, &app))?;
        app.handle_input()?;
    }

    restore_terminal()?;
    Ok(())
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<std::io::Stdout>>, Box<dyn Error>> {
    crossterm::terminal::enable_raw_mode()?;

    crossterm::execute!(std::io::stdout(), crossterm::terminal::EnterAlternateScreen)?;
    Ok(Terminal::new(CrosstermBackend::new(std::io::stdout()))?)
    // Ok(ratatui::init())
}

fn restore_terminal() -> Result<(), Box<dyn Error>> {
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(std::io::stdout(), crossterm::terminal::LeaveAlternateScreen)?;
    Ok(())
}
fn get_displayed_list<'a>(current_menu: &'a Menu, app: &App) -> Vec<&'a String> {
    if let Some(ref query) = app.search_query {
        current_menu
            .items
            .iter()
            .filter(|item| item.contains(query))
            .collect()
    } else {
        current_menu.items.iter().collect()
    }
}

fn ui(frame: &mut Frame, app: &App) {
    let (current_menu, selected_index) = app.menu_stack.last().unwrap();

    // Filter items based on search query
    let items_to_display: Vec<&String> = get_displayed_list(current_menu, app);

    let total_items = items_to_display.len();

    // Calculate visible range and arrows
    let (start_idx, end_idx, show_up, show_down) = {
        let mut start = selected_index.saturating_sub(9);
        let mut end = (start + 10).min(total_items);
        let show_up = start > 0;
        let mut show_down = end < total_items;

        // Adjust for arrows
        if show_up {
            start += 1;
            end = (start + 9).min(total_items);
            show_down = end < total_items;
        }
        (start, end, show_up, show_down)
    };

    // Build visible items with arrows
    let mut visible_items = Vec::new();
    if show_up {
        visible_items.push(ListItem::new("↑ More items above ↑"));
    }
    visible_items.extend(
        items_to_display[start_idx..end_idx]
            .iter()
            .map(|&string| ListItem::new(string.as_str())),
    );
    if show_down {
        visible_items.push(ListItem::new("↓ More items below ↓"));
    }

    let title = if let Some(ref query) = app.search_query {
        format!("{} [Search: {}]", current_menu.title, query)
    } else if let Some((ref _context, ref input)) = app.number_input {
        format!("{} [Value: {}]", current_menu.title, input)
    } else {
        current_menu.title.clone()
    };

    // Create list widget
    let list = List::new(visible_items)
        .block(Block::default().title(title).borders(Borders::ALL))
        .highlight_style(Style::default().fg(Color::Yellow))
        .highlight_symbol(">");

    // Calculate visible selection index
    let visible_selection = if *selected_index < start_idx {
        0
    } else {
        selected_index - start_idx + show_up as usize
    };

    let mut list_state = ListState::default().with_selected(Some(visible_selection));

    frame.render_stateful_widget(list, frame.area(), &mut list_state);
    if let Some(message) = &app.message {
        // Split message into lines
        let lines: Vec<String> = message.lines().map(|s| s.to_string()).collect();

        // Calculate display parameters
        let max_height = 20;
        let height = max_height.min(lines.len());
        let scroll = app.message_scroll.min(lines.len().saturating_sub(height));

        // Create layout
        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(height as u16 + 2),
                Constraint::Min(0),
            ])
            .split(frame.area());

        let horizontal_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25),
                Constraint::Percentage(50),
                Constraint::Percentage(25),
            ])
            .split(vertical_chunks[1]);

        let area = horizontal_chunks[1];

        // Create scrollable block
        let block = Block::default()
            .title(" Message ")
            .title_bottom("▲/▼ Scroll • Space/Enter: Close")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL);

        // Create inner scrolling area
        let inner_area = block.inner(area);
        frame.render_widget(Clear, area);
        frame.render_widget(block, area);

        // Create line chunks
        let line_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(1); height])
            .split(inner_area);

        // Render visible lines
        for (i, line) in lines.iter().skip(scroll).take(height).enumerate() {
            let text = Paragraph::new(line.as_str()).style(Style::default().fg(Color::White));
            frame.render_widget(text, line_chunks[i]);
        }

        // Add scroll indicator if needed
        if lines.len() > height {
            let scroll_text = format!("{}/{}", scroll + 1, lines.len());
            let indicator = Paragraph::new(scroll_text)
                .alignment(Alignment::Right)
                .style(Style::default().fg(Color::DarkGray));
            frame.render_widget(indicator, inner_area);
        }
    }
}

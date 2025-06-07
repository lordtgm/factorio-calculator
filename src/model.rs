use crate::data::materials::MaterialPrototype;
use crate::data::Process;
use good_lp::SolutionStatus::Optimal;
use good_lp::{
    microlp, variable, Constraint, Expression, IntoAffineExpression, ProblemVariables, Solution,
    SolverModel, Variable,
};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Default, Clone)]
#[derive(Serialize, Deserialize)]
pub struct Model {
    pub processes: Vec<Process>,
    pub inputs: HashMap<MaterialPrototype, f64>,
    pub outputs: HashMap<MaterialPrototype, f64>,
}

#[derive(Debug)]
pub enum ModelResult {
    NoSolution,
    OneSolution(HashMap<Process, f64>),
    MultipleSolutions {
        lower_bounds: HashMap<MaterialPrototype, f64>,
        higher_bounds: HashMap<MaterialPrototype, f64>,
    },
}

impl Model {
    //noinspection DuplicatedCode
    pub fn get_input(&self, name: &str) -> Option<(MaterialPrototype, f64)> {
        self.inputs
            .get_key_value(&MaterialPrototype::from_id(name).unwrap())
            .map(|(k, v)| (k.clone(), *v))
    }
    //noinspection DuplicatedCode
    pub fn get_output(&self, name: &str) -> Option<(MaterialPrototype, f64)> {
        self.outputs
            .get_key_value(&MaterialPrototype::from_id(name).unwrap())
            .map(|(k, v)| (k.clone(), *v))
    }

    //noinspection DuplicatedCode
    pub fn solve(&mut self, generate_inputs: bool) -> ModelResult {
        let mut variables = ProblemVariables::new();
        let mut processes: HashMap<&Process, Variable> = HashMap::new();
        let mut materials: HashMap<MaterialPrototype, Vec<&Process>> = HashMap::new();
        let mut constraints: Vec<Box<dyn Fn() -> Constraint>> = Vec::new();
        let mut input_variables: HashMap<MaterialPrototype, Variable> = HashMap::new();
        let mut output_variables: HashMap<MaterialPrototype, Variable> = HashMap::new();

        for process in self.processes.iter() {
            processes.insert(process, variables.add(variable().min(0)));
            for material in process
                .get_ingredients()
                .into_iter()
                .chain(process.get_products())
            {
                if !materials.contains_key(&material.get_prototype()) {
                    materials.insert(material.get_prototype(), Vec::new());
                }
                materials
                    .get_mut(&material.get_prototype())
                    .unwrap()
                    .push(&process);
            }
            #[cfg(all(debug_assertions, feature = "debug_model"))]
            println!(
                "{:#?} : {:#?}",
                process.name.clone(),
                processes.get(process).unwrap()
            );
        }

        for (prototype, &amount) in self.inputs.iter() {
            input_variables.insert(
                prototype.clone(),
                variables.add(variable().min(0).max(amount)),
            );
            #[cfg(all(debug_assertions, feature = "debug_model"))]
            println!(
                "input {:#?} : {:#?}",
                prototype,
                input_variables.get(prototype).unwrap()
            );
        }

        for (prototype, &amount) in self.outputs.iter() {
            output_variables.insert(prototype.clone(), variables.add(variable().min(amount)));
            #[cfg(all(debug_assertions, feature = "debug_model"))]
            println!(
                "output {:#?} : {:#?}",
                prototype,
                output_variables.get(prototype).unwrap()
            );
        }

        for (material_prototype, material_processes) in materials.iter() {
            #[cfg(all(debug_assertions, feature = "debug_model"))]
            println!("{:#?}", &material_prototype);
            let mut expression = Expression::from(0);
            let mut generate_input: bool = true;
            for &process in material_processes {
                #[cfg(all(debug_assertions, feature = "debug_model"))]
                println!("{:#?}", &process.name.clone());
                for ingredient in process.get_ingredients() {
                    if ingredient.get_prototype() == *material_prototype {
                        expression -= *processes.get(process).unwrap();
                        #[cfg(all(debug_assertions, feature = "debug_model"))]
                        println!(
                            "{:#?}",
                            ingredient.get_average_amount(process.productivity as f64)
                        );
                    }
                }
                for product in process.get_products() {
                    if product.get_prototype() == *material_prototype {
                        generate_input = false;
                        expression += *processes.get(process).unwrap()
                            * product.get_average_amount(process.productivity as f64);
                        #[cfg(all(debug_assertions, feature = "debug_model"))]
                        println!(
                            "{:#?}",
                            product.get_average_amount(process.productivity as f64)
                        );
                    }
                }
            }
            if generate_inputs
                && generate_input
                && !self.inputs.contains_key(&material_prototype)
                && !self.outputs.contains_key(&material_prototype)
            {
                input_variables
                    .insert(material_prototype.clone(), variables.add(variable().min(0)));
                #[cfg(all(debug_assertions, feature = "debug_model"))]
                println!(
                    "input {:#?} : {:#?}",
                    material_prototype,
                    input_variables.get(material_prototype).unwrap()
                );
            }
            #[cfg(all(debug_assertions, feature = "debug_model"))]
            println!("{:#?}", &expression);
            let limit: Expression = match (
                output_variables.get(&material_prototype),
                input_variables.get(&material_prototype),
            ) {
                (Some(output), Some(input)) => {
                    if -self.inputs.get(&material_prototype).unwrap()
                        < *self.outputs.get(&material_prototype).unwrap()
                    {
                        -input.into_expression()
                    } else {
                        output.into_expression()
                    }
                }
                (Some(output), None) => output.into_expression(),
                (None, Some(input)) => -input.into_expression(),
                (None, None) => 0.into_expression(),
            };
            #[cfg(all(debug_assertions, feature = "debug_model"))]
            println!("{:#?}", limit);
            constraints.push(Box::new(move || expression.clone().geq(limit.clone())));
        }

        let results = copy_variables(&variables)
            .minimise(
                processes
                    .iter()
                    .fold(Expression::from(0), |acc, process| acc + process.1),
            )
            .using(microlp)
            .with_all(constraints.iter().map(|function| function()))
            .solve();

        if results.is_err()
            || match results.as_ref().unwrap().status() {
                Optimal => false,
                _ => true,
            }
        {
            return ModelResult::NoSolution;
        }

        let mut multiple_solutions: bool = false;

        let mut input_optimums: HashMap<MaterialPrototype, f64> = HashMap::new();
        let mut output_optimums: HashMap<MaterialPrototype, f64> = HashMap::new();

        for (material, variable) in input_variables.iter() {
            let (min, max) = solve_for(&variables, &constraints, variable, !generate_inputs);
            if generate_inputs {
                self.inputs.insert(material.clone(), min);
            }
            if (max - min).abs() > 1e-6 {
                multiple_solutions = true;
                input_optimums.insert(material.clone(), min);
            }
        }

        for (material, variable) in output_variables.iter() {
            let (min, max) = solve_for(&variables, &constraints, variable, !generate_inputs);
            if (max - min).abs() > 1e-6 {
                multiple_solutions = true;
                output_optimums.insert(material.clone(), max);
            }
        }

        if !multiple_solutions {
            return ModelResult::OneSolution(
                processes
                    .iter()
                    .map(|(&process, variable)| {
                        (process.clone(), results.as_ref().unwrap().value(*variable))
                    })
                    .collect(),
            );
        }

        ModelResult::MultipleSolutions {
            lower_bounds: input_optimums,
            higher_bounds: output_optimums,
        }
    }
}

fn copy_variables(variables: &ProblemVariables) -> ProblemVariables {
    let mut new_variables = ProblemVariables::new();
    for (_, variable_definition) in variables.iter_variables_with_def() {
        new_variables.add(variable_definition.clone());
    }
    new_variables
}

fn solve_for(
    variables: &ProblemVariables,
    constraints: &Vec<Box<dyn Fn() -> Constraint>>,
    variable: &Variable,
    maximize: bool,
) -> (f64, f64) {
    let min = copy_variables(&variables)
        .minimise(variable)
        .using(microlp)
        .with_all(constraints.iter().map(|function| function()))
        .solve()
        .unwrap()
        .value(*variable);
    let max = if maximize {
        copy_variables(&variables)
            .maximise(variable)
            .using(microlp)
            .with_all(constraints.iter().map(|function| function()))
            .solve()
            .unwrap()
            .value(*variable)
    } else {
        min
    };
    (min, max)
}

#[cfg(test)]
mod tests {
    use crate::data::data_loader::load_data;
    use crate::data::materials::MaterialPrototype;
    use crate::model::Model;
    use std::collections::HashMap;

    #[test]
    fn test() {
        let _registry =
            load_data("E:/Games/Factorio/script-output/data-raw-dump.json".to_string()).unwrap();
        let mut model: Model = Model {
            processes: vec![],
            inputs: HashMap::new(),
            outputs: HashMap::from([(MaterialPrototype::Item("rocket-part".to_string()), 200.0)]),
        };
        println!("{:?}", model.solve(true));
        println!("{:#?}", model.inputs);
    }
}

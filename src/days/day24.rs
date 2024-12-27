use crate::libs::{
    cli::{new_cli_problem, CliProblem, Freeze},
    parse::{parse_alphanumeric, parse_lines, StringParse},
    problem::{Problem, ProblemResult},
};
use ahash::{AHashMap, AHashSet};
use chumsky::{
    error::Rich,
    extra,
    prelude::{choice, just},
    text, IterParser, Parser,
};
use clap::{Args, ValueEnum};
use itertools::Itertools;
use std::{collections::VecDeque, sync::LazyLock};

pub static DAY_24: LazyLock<CliProblem<Input, CommandLineArguments, Day24, Freeze>> = LazyLock::new(
    || {
        new_cli_problem(
            "day24",
            "Finds information about a circuit",
            "Newline delimited list of the intial circuit values, followed by a blank line, followed by a newline delimited list of gates.",
        )
        .with_part(
            "Simluates the gates and returns the output.",
            CommandLineArguments {
                wire_task: WireTask::Simulate,
            },
            vec![
                ("sample.txt", 4_usize.into()),
                ("sample2.txt", 2024_usize.into()),
            ],
        )
        .with_part(
            "Verifies the gates are a full adder and returns the output gates which need to be swapped to do so.",
            CommandLineArguments {
                wire_task: WireTask::FixAdder,
            },
            vec![],
        )
        .freeze()
    },
);

#[derive(Debug)]
pub struct Input {
    gate_values: Vec<(String, bool)>,
    gates: Vec<Gate>,
}

impl StringParse for Input {
    fn parse<'a>() -> impl Parser<'a, &'a str, Self, extra::Err<Rich<'a, char>>> {
        let bool = just("1").to(true).or(just("0").to(false));
        let gate_value = parse_alphanumeric()
            .map(|s: &'a str| s.to_string())
            .then_ignore(just(": "))
            .then(bool);
        let gate_values = gate_value
            .separated_by(text::newline())
            .at_least(1)
            .collect::<Vec<_>>();

        let and = just("AND").to(GateType::And);
        let or = just("OR").to(GateType::Or);
        let xor = just("XOR").to(GateType::Xor);
        let gate_type = choice((and, or, xor));

        let gate = parse_alphanumeric()
            .map(|s: &'a str| s.to_string())
            .then_ignore(just(" "))
            .then(gate_type)
            .then_ignore(just(" "))
            .then(parse_alphanumeric().map(|s: &'a str| s.to_string()))
            .then_ignore(just(" -> "))
            .then(parse_alphanumeric().map(|s: &'a str| s.to_string()))
            .map(|(((operand1, gate_type), operand2), result)| Gate {
                operand1,
                operand2,
                result,
                gate_type,
            });
        let gates = parse_lines(gate);

        gate_values
            .then_ignore(text::newline().repeated().at_least(1))
            .then(gates)
            .map(|(gate_values, gates)| Input { gate_values, gates })
    }
}

#[derive(Debug, Clone)]
struct Gate {
    operand1: String,
    operand2: String,
    result: String,
    gate_type: GateType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum GateType {
    And,
    Or,
    Xor,
}

#[derive(ValueEnum, Clone)]
enum WireTask {
    Simulate,
    FixAdder,
}

#[derive(Args)]
pub struct CommandLineArguments {
    #[arg(short, long, help = "What to do with the wires")]
    wire_task: WireTask,
}

pub struct Day24 {}

impl Problem<Input, CommandLineArguments> for Day24 {
    type Output = ProblemResult;

    fn run(input: Input, arguments: &CommandLineArguments) -> Self::Output {
        let gates = input.gates;

        match arguments.wire_task {
            WireTask::Simulate => {
                let mut gate_values = input.gate_values.into_iter().collect::<AHashMap<_, _>>();
                simulate_gates(&mut gate_values, &gates);
                extract_output_gates(&gate_values).into()
            }
            WireTask::FixAdder => {
                let mut carry_in = None;
                let mut swapped_gates = Vec::new();

                for i in 0.. {
                    let result = find_addition_carry_and_swaps(i, carry_in, &gates);
                    carry_in = result.0;
                    result.1.into_iter().for_each(|(gate1, gate2)| {
                        swapped_gates.push(gate1);
                        swapped_gates.push(gate2);
                    });

                    if carry_in.is_none() {
                        break;
                    }
                }

                swapped_gates.into_iter().sorted().join(",").into()
            }
        }
    }
}

fn find_addition_carry_and_swaps(
    i: usize,
    carry_in_gate: Option<String>,
    gates: &[Gate],
) -> (Option<String>, Option<(String, String)>) {
    let x_gate = format!("x{:0>2}", i);
    let y_gate = format!("y{:0>2}", i);
    let mut swapped_gates = None;

    let add_gate = gates
        .iter()
        .find(|gate| {
            (gate.operand1 == x_gate || gate.operand1 == y_gate)
                && (gate.operand2 == x_gate || gate.operand2 == y_gate)
                && gate.gate_type == GateType::Xor
        })
        .map(|gate| gate.result.clone());

    let carry_gate = gates
        .iter()
        .find(|gate| {
            (gate.operand1 == x_gate || gate.operand1 == y_gate)
                && (gate.operand2 == x_gate || gate.operand2 == y_gate)
                && gate.gate_type == GateType::And
        })
        .map(|gate| gate.result.clone());

    if carry_in_gate.is_none() {
        return (
            match (add_gate, carry_gate) {
                (Some(add), Some(carry)) => {
                    if !add.starts_with("z") {
                        // Add result detected as wrong, must be swapped with carry
                        swapped_gates = Some((add.clone(), carry.clone()));
                        Some(add)
                    } else {
                        Some(carry)
                    }
                }
                _ => None,
            },
            swapped_gates,
        );
    }

    if add_gate.is_none() || carry_gate.is_none() {
        // Must be the final output
        return (None, swapped_gates);
    }

    let mut add_gate = add_gate.expect("Exists");
    let mut carry_gate = carry_gate.expect("Exists");
    let carry_in_gate = carry_in_gate.expect("Exists");

    let final_add = gates
        .iter()
        .find(|gate| {
            (gate.operand1 == add_gate || gate.operand1 == carry_in_gate)
                && (gate.operand2 == add_gate || gate.operand2 == carry_in_gate)
                && gate.gate_type == GateType::Xor
        })
        .map(|gate| gate.result.clone());

    let final_add = if let Some(final_add) = final_add {
        final_add
    } else {
        // Final addition detected as wrong, carry and add must be swapped
        swapped_gates = Some((add_gate.clone(), carry_gate.clone()));
        (add_gate, carry_gate) = (carry_gate, add_gate);
        gates
            .iter()
            .find(|gate| {
                (gate.operand1 == add_gate || gate.operand1 == carry_in_gate)
                    && (gate.operand2 == add_gate || gate.operand2 == carry_in_gate)
                    && gate.gate_type == GateType::Xor
            })
            .map(|gate| gate.result.clone())
            .expect("Exists after swap")
    };

    let mut carry_in_add = gates
        .iter()
        .find(|gate| {
            (gate.operand1 == add_gate || gate.operand1 == carry_in_gate)
                && (gate.operand2 == add_gate || gate.operand2 == carry_in_gate)
                && gate.gate_type == GateType::And
        })
        .map(|gate| gate.result.clone())
        .expect("Should always exist at this point.");

    let final_carry = gates
        .iter()
        .find(|gate| {
            (gate.operand1 == carry_gate || gate.operand1 == carry_in_add)
                && (gate.operand2 == carry_gate || gate.operand2 == carry_in_add)
                && gate.gate_type == GateType::Or
        })
        .map(|gate| gate.result.clone());

    (
        if final_carry.is_none() {
            // Final carry detected as bad

            if !final_add.starts_with("z") {
                if carry_in_add.starts_with("z") {
                    // carry_in_add, final_add swapped
                    swapped_gates = Some((final_add.clone(), carry_in_add.clone()));
                    carry_in_add = final_add;
                } else if carry_gate.starts_with("z") {
                    // carry_gate, final_add swapped
                    swapped_gates = Some((final_add.clone(), carry_gate.clone()));
                    carry_gate = final_add;
                }
            }

            gates
                .iter()
                .find(|gate| {
                    (gate.operand1 == carry_gate || gate.operand1 == carry_in_add)
                        || (gate.operand2 == carry_gate || gate.operand2 == carry_in_add)
                            && gate.gate_type == GateType::Or
                })
                .map(|gate| gate.result.clone())
        } else {
            let final_carry = final_carry.expect("Exists");
            if final_carry.starts_with("z") {
                let next_z_gate = format!("z{:0>2}", i + 1);
                if final_carry != next_z_gate {
                    // final carry detected as wrong, final_add and final_carry must be swapped
                    swapped_gates = Some((final_add.clone(), final_carry.clone()));
                    return (Some(final_add), swapped_gates);
                }

                Some(final_carry)
            } else {
                Some(final_carry)
            }
        },
        swapped_gates,
    )
}

fn extract_output_gates(gate_values: &AHashMap<String, bool>) -> usize {
    let mut result = 0;
    gate_values
        .iter()
        .filter(|(key, _)| key.starts_with("z"))
        .map(|(key, value)| {
            let (_, key) = key.split_at(1);
            let shift = key.parse::<usize>().expect("valid");
            (*value as usize) << shift
        })
        .for_each(|value| {
            result |= value;
        });

    result
}

fn simulate_gates(gate_values: &mut AHashMap<String, bool>, gates: &[Gate]) {
    let mut visited = AHashSet::new();
    let mut gates_to_process = VecDeque::from_iter(gates.iter());

    while let Some(gate) = gates_to_process.pop_front() {
        let operand1 = gate_values.get(&gate.operand1);
        let operand2 = gate_values.get(&gate.operand2);
        if visited.contains(&gate.result) {
            continue;
        }

        match (operand1, operand2) {
            (Some(operand1), Some(operand2)) => {
                let result = match gate.gate_type {
                    GateType::And => operand1 & operand2,
                    GateType::Or => operand1 | operand2,
                    GateType::Xor => operand1 ^ operand2,
                };

                gate_values.insert(gate.result.clone(), result);
                visited.insert(gate.result.clone());
            }
            _ => gates_to_process.push_back(gate),
        }
    }
}

use crate::libs::{
    cli::{new_cli_problem, CliProblem, Freeze},
    parse::{parse_usize, StringParse},
    problem::{Problem, ProblemResult},
};
use adventofcode_macro::{problem_day, problem_parse};
use ahash::AHashMap;
use chumsky::{
    error::Rich,
    extra,
    prelude::{choice, end, just},
    text, IterParser, Parser,
};
use clap::{Args, ValueEnum};
use itertools::Itertools;
use std::{collections::VecDeque, iter::once, sync::LazyLock};
use tap::Tap;

pub static DAY_17: LazyLock<CliProblem<Day17, CommandLineArguments, Freeze>> =
    LazyLock::new(|| {
        new_cli_problem(
            "day17",
            "Runs a program",
            "The program initial registers followed by the program itself.",
        )
        .with_part(
            "Computes the output of the program",
            CommandLineArguments {
                program_execution: ProgramExecution::Run,
            },
            vec![("sample.txt", "4,6,3,5,6,3,5,2,1,0".to_string().into())],
        )
        .with_part(
            "Computes the register a value that would have the program produce itself.",
            CommandLineArguments {
                program_execution: ProgramExecution::FindQuine,
            },
            vec![("sample2.txt", 117440_usize.into())],
        )
        .freeze()
    });

#[derive(Clone, Debug)]
pub struct Day17 {
    a: usize,
    b: usize,
    c: usize,
    program: Vec<(Instruction, usize)>,
}

#[problem_parse]
fn parse<'a>() -> impl Parser<'a, &'a str, Day17, extra::Err<Rich<'a, char>>> {
    let program = parse_instruction()
        .separated_by(just(","))
        .at_least(1)
        .collect::<Vec<_>>();

    just("Register A: ")
        .ignore_then(parse_usize())
        .then_ignore(text::newline())
        .then_ignore(just("Register B: "))
        .then(parse_usize())
        .then_ignore(text::newline())
        .then_ignore(just("Register C: "))
        .then(parse_usize())
        .then_ignore(text::newline().repeated().at_least(1))
        .then_ignore(just("Program: "))
        .then(program)
        .then_ignore(text::newline())
        .then_ignore(end())
        .map(|(((a, b), c), program)| Day17 { a, b, c, program })
}

fn parse_instruction<'a>(
) -> impl Parser<'a, &'a str, (Instruction, usize), extra::Err<Rich<'a, char>>> {
    let adv = just("0").to(Instruction::Adv);
    let bxl = just("1").to(Instruction::Bxl);
    let bst = just("2").to(Instruction::Bst);
    let jnz = just("3").to(Instruction::Jnz);
    let bxc = just("4").to(Instruction::Bxc);
    let out = just("5").to(Instruction::Out);
    let bdv = just("6").to(Instruction::Bdv);
    let cdv = just("7").to(Instruction::Cdv);
    let opcode = choice((adv, bxl, bst, jnz, bxc, out, bdv, cdv));
    opcode.then_ignore(just(",")).then(parse_usize())
}

#[derive(Debug, Clone)]
enum Instruction {
    Adv,
    Bxl,
    Bst,
    Jnz,
    Bxc,
    Out,
    Bdv,
    Cdv,
}

impl Instruction {
    fn get_numeral(&self) -> usize {
        match self {
            Instruction::Adv => 0,
            Instruction::Bxl => 1,
            Instruction::Bst => 2,
            Instruction::Jnz => 3,
            Instruction::Bxc => 4,
            Instruction::Out => 5,
            Instruction::Bdv => 6,
            Instruction::Cdv => 7,
        }
    }
}

#[derive(ValueEnum, Clone)]
enum ProgramExecution {
    Run,
    FindQuine,
}

#[derive(Args)]
pub struct CommandLineArguments {
    #[arg(short, long, help = "What to do with the input program.")]
    program_execution: ProgramExecution,
}

#[problem_day]
fn run(mut input: Day17, arguments: &CommandLineArguments) -> ProblemResult {
    match arguments.program_execution {
        ProgramExecution::Run => run_program(&mut input).into(),
        ProgramExecution::FindQuine => {
            let valid_bit_patterns: AHashMap<usize, Vec<usize>> = (0..1024)
                .map(|i| {
                    input.a = i;
                    input.b = 0;
                    input.c = 0;

                    (run_program_with_first_out(&mut input), i)
                })
                .fold(AHashMap::new(), |mut acc, (key, pattern)| {
                    let patterns = acc.entry(key).or_default();
                    patterns.push(pattern);
                    acc
                });

            let shift = 3;
            let mask = 0b_0000_0111_1111;

            let mut to_find: VecDeque<usize> = input
                .program
                .iter()
                .flat_map(|(operator, operand)| once(operator.get_numeral()).chain(once(*operand)))
                .collect::<VecDeque<_>>();

            let target_string = to_find.iter().map(|value| value.to_string()).join(",");

            let mut previous_patterns = valid_bit_patterns
                .get(&to_find.pop_front().expect("exists"))
                .expect("exists")
                .clone();

            let mut i = 1;

            while let Some(next) = to_find.pop_front() {
                let patterns = valid_bit_patterns.get(&next).expect("exists");
                previous_patterns = previous_patterns
                    .iter()
                    .flat_map(|previous_pattern| {
                        let shifted = previous_pattern >> (shift * i);

                        patterns
                            .iter()
                            .filter(|pattern| (**pattern & mask) == shifted)
                            .map(|pattern| (pattern << (shift * i)) | previous_pattern)
                            .collect::<Vec<_>>()
                    })
                    .collect();

                i += 1;
            }

            (*previous_patterns
                .tap_mut(|patterns| patterns.sort())
                .iter()
                .find(|a_value| {
                    input.a = **a_value;
                    input.b = 0;
                    input.c = 0;
                    run_program(&mut input) == target_string
                })
                .expect("Exists"))
            .into()
        }
    }
}

fn run_program_with_first_out(input: &mut Day17) -> usize {
    let mut pc = 0;

    while pc < input.program.len() * 2 {
        let (opcode, operand) = input.program.get(pc / 2).expect("Exists");
        let combo_value = get_value(input, *operand);

        match opcode {
            Instruction::Adv => {
                input.a >>= combo_value;
            }
            Instruction::Bxl => input.b ^= operand,
            Instruction::Bst => input.b = combo_value & 0b111,
            Instruction::Jnz => {
                if input.a != 0 {
                    pc = *operand;
                    continue;
                }
            }
            Instruction::Bxc => input.b ^= input.c,
            Instruction::Out => return combo_value & 0b111,
            Instruction::Bdv => {
                input.b = input.a >> combo_value;
            }
            Instruction::Cdv => {
                input.c = input.a >> combo_value;
            }
        }

        pc += 2;
    }

    panic!("No output!")
}

fn run_program(input: &mut Day17) -> String {
    let mut pc = 0;
    let mut out = Vec::new();

    while pc < input.program.len() * 2 {
        let (opcode, operand) = input.program.get(pc / 2).expect("Exists");
        let combo_value = get_value(input, *operand);

        match opcode {
            Instruction::Adv => {
                input.a >>= combo_value;
            }
            Instruction::Bxl => input.b ^= operand,
            Instruction::Bst => input.b = combo_value & 0b111,
            Instruction::Jnz => {
                if input.a != 0 {
                    pc = *operand;
                    continue;
                }
            }
            Instruction::Bxc => input.b ^= input.c,
            Instruction::Out => out.push(combo_value & 0b111),
            Instruction::Bdv => {
                input.b = input.a >> combo_value;
            }
            Instruction::Cdv => {
                input.c = input.a >> combo_value;
            }
        }

        pc += 2;
    }

    out.into_iter().map(|value| value.to_string()).join(",")
}

fn get_value(register: &Day17, operand: usize) -> usize {
    match operand {
        x @ 0..=3 => x,
        4 => register.a,
        5 => register.b,
        6 => register.c,
        _ => unreachable!(),
    }
}

use crate::libs::{
    cli::{new_cli_problem, CliProblem, Freeze},
    parse::{parse_usize, StringParse},
    problem::Problem,
};
use adventofcode_macro::{problem_day, problem_parse};
use chumsky::{
    error::Rich,
    extra,
    prelude::{any, choice, end, just},
    IterParser, Parser,
};
use clap::Args;
use std::sync::LazyLock;

pub static DAY_03: LazyLock<CliProblem<Day03, CommandLineArguments, Freeze>> = LazyLock::new(
    || {
        new_cli_problem(
            "day03",
            "Interprets instructions from corrupted memory",
            "String with potiential instructions inside of it",
        )
        .with_part(
            "Computes the sum of all the mul instructions in the data",
            CommandLineArguments { full_instruction_set: false },
            vec![("sample.txt", 161)],
        )
        .with_part(
            "Computes the sum of all the mul instructions in the data not gaurded by a don't instruction",
            CommandLineArguments { full_instruction_set: true },
            vec![("sample2.txt", 48)],
        )
        .freeze()
    },
);

#[derive(Args)]
pub struct CommandLineArguments {
    #[arg(short, long, help = "Use the full instruction set or not")]
    full_instruction_set: bool,
}

pub struct Day03(Vec<Instruction>);

#[derive(Debug, Clone)]
enum Instruction {
    Multiply(usize, usize),
    Do,
    Dont,
    Garbage,
}

#[problem_parse]
fn parse<'a>() -> impl Parser<'a, &'a str, Day03, extra::Err<Rich<'a, char>>> {
    let do_ = just("do()").to(Instruction::Do);
    let dont = just("don't()").to(Instruction::Dont);
    let garbage = any()
        .and_is(choice((parse_mul(), do_.clone(), dont.clone())).not())
        .repeated()
        .at_least(1)
        .to(Instruction::Garbage);

    choice((parse_mul(), do_, dont, garbage))
        .repeated()
        .at_least(1)
        .collect::<Vec<_>>()
        .then_ignore(end())
        .map(Day03)
}

fn parse_mul<'a>() -> impl Parser<'a, &'a str, Instruction, extra::Err<Rich<'a, char>>> {
    just("mul(")
        .ignore_then(parse_usize())
        .then_ignore(just(","))
        .then(parse_usize())
        .then_ignore(just(")"))
        .map(|(a, b)| Instruction::Multiply(a, b))
}

#[problem_day]
fn run(Day03(input): Day03, arguments: &CommandLineArguments) -> usize {
    let mut do_ = true;

    input
        .into_iter()
        .map(|i| match i {
            Instruction::Multiply(x, y) => {
                if do_ {
                    x * y
                } else {
                    0
                }
            }
            Instruction::Garbage => 0,
            Instruction::Do => {
                do_ = true;
                0
            }
            Instruction::Dont => {
                if arguments.full_instruction_set {
                    do_ = false;
                }
                0
            }
        })
        .sum()
}

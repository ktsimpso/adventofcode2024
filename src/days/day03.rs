use crate::libs::{
    cli::{new_cli_problem, Command},
    parse::{parse_usize, StringParse},
    problem::Problem,
};
use chumsky::{
    error::Rich,
    extra,
    prelude::{any, choice, end, just},
    IterParser, Parser,
};
use clap::Args;
use std::sync::LazyLock;

pub static DAY_03: LazyLock<Box<dyn Command + Send + Sync>> = LazyLock::new(|| {
    Box::new(
        new_cli_problem::<Input, CommandLineArguments, Day03>(
            "day03",
            "Interprets instructions from corrupted memory",
            "String with potiential instructions inside of it",
        )
        .with_part(
            "Computes the sum of all the mul instructions in the data",
            CommandLineArguments { full_instruction_set: false },
        )
        .with_part(
            "Computes the sum of all the mul instructions in the data not gaurded by a don't instruction",
            CommandLineArguments { full_instruction_set: true },
        )
        .freeze(),
    )
});

struct Input(Vec<Instruction>);

impl StringParse for Input {
    fn parse<'a>() -> impl Parser<'a, &'a str, Self, extra::Err<Rich<'a, char>>> {
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
            .map(Input)
    }
}

fn parse_mul<'a>() -> impl Parser<'a, &'a str, Instruction, extra::Err<Rich<'a, char>>> {
    just("mul(")
        .ignore_then(parse_usize())
        .then_ignore(just(","))
        .then(parse_usize())
        .then_ignore(just(")"))
        .map(|(a, b)| Instruction::Multiply(a, b))
}

#[derive(Debug, Clone)]
enum Instruction {
    Multiply(usize, usize),
    Do,
    Dont,
    Garbage,
}

#[derive(Args)]
struct CommandLineArguments {
    #[arg(short, long, help = "Use the full instruction set or not")]
    full_instruction_set: bool,
}

struct Day03 {}

impl Problem<Input, CommandLineArguments> for Day03 {
    type Output = usize;

    fn run(input: Input, arguments: &CommandLineArguments) -> Self::Output {
        let mut do_ = true;

        input
            .0
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
}

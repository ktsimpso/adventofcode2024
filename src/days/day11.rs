use crate::libs::{
    cli::{new_cli_problem, CliProblem, Freeze},
    parse::{parse_usize, StringParse},
    problem::Problem,
};
use ahash::AHashMap;
use chumsky::{
    error::Rich,
    extra,
    prelude::{end, just},
    text, IterParser, Parser,
};
use clap::Args;
use std::sync::LazyLock;

pub static DAY_11: LazyLock<CliProblem<Input, CommandLineArguments, Day11, Freeze>> =
    LazyLock::new(|| {
        new_cli_problem(
            "day11",
            "Counts the number of stones after a certain number of blinks",
            "Space delimited list of the intial stone numbers",
        )
        .with_part(
            "Computes the number of stones after 25 iterations.",
            CommandLineArguments { n: 25 },
            vec![("sample2.txt", 55312)],
        )
        .with_part(
            "Computes the number of stones after 75 iterations",
            CommandLineArguments { n: 75 },
            vec![],
        )
        .freeze()
    });

pub struct Input(Vec<usize>);

impl StringParse for Input {
    fn parse<'a>() -> impl Parser<'a, &'a str, Self, extra::Err<Rich<'a, char>>> {
        parse_usize()
            .separated_by(just(" "))
            .at_least(1)
            .collect()
            .then_ignore(text::newline())
            .then_ignore(end())
            .map(Input)
    }
}

#[derive(Args)]
pub struct CommandLineArguments {
    #[arg(short, long, help = "Number of times to blink")]
    n: usize,
}

pub struct Day11 {}

impl Problem<Input, CommandLineArguments> for Day11 {
    type Output = usize;

    fn run(input: Input, arguments: &CommandLineArguments) -> Self::Output {
        let mut stones = input.0.into_iter().fold(AHashMap::new(), |mut acc, stone| {
            *acc.entry(stone).or_insert(0) += 1;
            acc
        });

        for _ in 0..arguments.n {
            stones = stones
                .into_iter()
                .flat_map(|(stone, count)| {
                    if stone == 0 {
                        vec![(1, count)]
                    } else if let digits = stone.ilog10() + 1
                        && digits % 2 == 0
                    {
                        let divisor = 10_usize.pow(digits / 2);
                        vec![(stone / divisor, count), (stone % divisor, count)]
                    } else {
                        vec![(stone * 2024, count)]
                    }
                })
                .fold(AHashMap::new(), |mut acc, (stone, count)| {
                    *acc.entry(stone).or_insert(0) += count;
                    acc
                });
        }

        stones.values().sum()
    }
}

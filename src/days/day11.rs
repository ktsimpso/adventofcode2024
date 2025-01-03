use crate::libs::{
    cli::{new_cli_problem, CliProblem, Freeze},
    parse::{parse_usize, ParserExt, StringParse},
    problem::Problem,
};
use adventofcode_macro::{problem_day, problem_parse};
use ahash::AHashMap;
use chumsky::{error::Rich, extra, prelude::just, IterParser, Parser};
use clap::Args;
use std::sync::LazyLock;

pub static DAY_11: LazyLock<CliProblem<Day11, CommandLineArguments, Freeze>> =
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

#[derive(Args)]
pub struct CommandLineArguments {
    #[arg(short, long, help = "Number of times to blink")]
    n: usize,
}

pub struct Day11(Vec<usize>);

#[problem_parse]
fn parse<'a>() -> impl Parser<'a, &'a str, Day11, extra::Err<Rich<'a, char>>> {
    parse_usize()
        .separated_by(just(" "))
        .at_least(1)
        .collect()
        .map(Day11)
        .end()
}

#[problem_day]
fn run(Day11(input): Day11, arguments: &CommandLineArguments) -> usize {
    let mut stones = input.into_iter().fold(AHashMap::new(), |mut acc, stone| {
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

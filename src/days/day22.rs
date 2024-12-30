use crate::libs::{
    cli::{new_cli_problem, CliProblem, Freeze},
    parse::{parse_lines, parse_usize, StringParse},
    problem::Problem,
};
use adventofcode_macro::problem_day;
use chumsky::{error::Rich, extra, Parser};
use clap::{Args, ValueEnum};
use itertools::{iterate, Itertools};
use std::sync::LazyLock;

pub static DAY_22: LazyLock<CliProblem<Input, CommandLineArguments, Day22, Freeze>> =
    LazyLock::new(|| {
        new_cli_problem(
            "day22",
            "Finds out how to corner the banana market.",
            "Newline delimited lists of numbers",
        )
        .with_part(
            "Computes the sum of the 2000th secret number for all the monkeys",
            CommandLineArguments {
                banana_market_information: BananaMarketInformation::LastSecret,
            },
            vec![("sample.txt", 37327623)],
        )
        .with_part(
            "Computes the maximum number of purchasable bananas given the best prefix value",
            CommandLineArguments {
                banana_market_information: BananaMarketInformation::MostBananas,
            },
            vec![("sample2.txt", 23)],
        )
        .freeze()
    });

pub struct Input(Vec<usize>);

impl StringParse for Input {
    fn parse<'a>() -> impl Parser<'a, &'a str, Self, extra::Err<Rich<'a, char>>> {
        parse_lines(parse_usize()).map(Input)
    }
}

#[derive(ValueEnum, Clone)]
enum BananaMarketInformation {
    LastSecret,
    MostBananas,
}

#[derive(Args)]
pub struct CommandLineArguments {
    #[arg(
        short,
        long,
        help = "The infomration about the banana market you want."
    )]
    banana_market_information: BananaMarketInformation,
}

#[problem_day(Day22)]
fn run(input: Input, arguments: &CommandLineArguments) -> usize {
    match arguments.banana_market_information {
        BananaMarketInformation::LastSecret => input
            .0
            .into_iter()
            .flat_map(|number| iterate(number, |number| next_secret(*number)).nth(2000))
            .sum(),
        BananaMarketInformation::MostBananas => input
            .0
            .into_iter()
            .enumerate()
            .fold(
                (vec![0_u16; 1_048_576], vec![0_u16; 1_048_576]),
                |mut acc, (index, number)| {
                    price_by_last_four_deltas(number, (index + 1) as u16, &mut acc.0, &mut acc.1);
                    acc
                },
            )
            .0
            .into_iter()
            .max()
            .unwrap_or(0) as usize,
    }
}

fn price_by_last_four_deltas(
    number: usize,
    seen_generation: u16,
    prices: &mut [u16],
    seen: &mut [u16],
) {
    let mut numbers = iterate(number, |number| next_secret(*number))
        .take(2000)
        .map(|number| number % 10)
        .tuple_windows();

    let mut key = 0;

    numbers.by_ref().take(3).for_each(|(previous, current)| {
        let difference = current.wrapping_sub(previous) + 9;
        key = ((key << 5) | difference) & 1_048_575;
    });

    numbers.for_each(|(previous, current)| {
        let difference = current.wrapping_sub(previous) + 9;
        key = ((key << 5) | difference) & 1_048_575;

        if seen[key] != seen_generation {
            seen[key] = seen_generation;
            prices[key] += current as u16;
        }
    });
}

fn next_secret(mut number: usize) -> usize {
    number ^= number << 6;
    number &= 16777215;

    number ^= number >> 5;

    number ^= number << 11;
    number & 16777215
}

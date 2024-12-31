use crate::libs::{
    cli::{new_cli_problem, CliProblem, Freeze},
    parse::{parse_lines, StringParse},
    problem::Problem,
};
use adventofcode_macro::{problem_day, problem_parse};
use ahash::AHashMap;
use chumsky::{
    error::Rich,
    extra,
    prelude::{choice, just},
    text, IterParser, Parser,
};
use clap::{Args, ValueEnum};
use std::sync::LazyLock;

pub static DAY_19: LazyLock<CliProblem<Day19, CommandLineArguments, Freeze>> = LazyLock::new(
    || {
        new_cli_problem(
            "day19",
            "Finds if you can combine base towels to make target towels",
            "Comma delimited list of towels, followed by a blank line, then a new line delimited list of target towels.",
        )
        .with_part(
            "Computes the number of target towels that are possible to make.",
            CommandLineArguments {
                towel_options: TowelOptions::TowelPossible,
            },
            vec![("sample.txt", 6)],
        )
        .with_part(
            "Computes the number of ways you can make the target towels.",
            CommandLineArguments {
                towel_options: TowelOptions::NumberOfWaysPossible,
            },
            vec![("sample.txt", 16)],
        )
        .freeze()
    },
);

#[derive(ValueEnum, Clone)]
enum TowelOptions {
    TowelPossible,
    NumberOfWaysPossible,
}

#[derive(Args)]
pub struct CommandLineArguments {
    #[arg(
        short,
        long,
        help = "Whether to count only if the configuration is possible, or all valid configurations."
    )]
    towel_options: TowelOptions,
}

#[derive(Debug)]
pub struct Day19 {
    available_towels: Vec<Vec<Color>>,
    target_towels: Vec<Vec<Color>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Color {
    White,
    Blue,
    Black,
    Red,
    Green,
}

#[problem_parse]
fn parse<'a>() -> impl Parser<'a, &'a str, Day19, extra::Err<Rich<'a, char>>> {
    let available_towels = parse_towel_color()
        .separated_by(just(", "))
        .at_least(1)
        .collect();

    available_towels
        .then_ignore(text::newline().repeated().at_least(1))
        .then(parse_lines(parse_towel_color()))
        .map(|(available_towels, target_towels)| Day19 {
            available_towels,
            target_towels,
        })
}

fn parse_towel_color<'a>() -> impl Parser<'a, &'a str, Vec<Color>, extra::Err<Rich<'a, char>>> {
    let white = just("w").to(Color::White);
    let blue = just("u").to(Color::Blue);
    let black = just("b").to(Color::Black);
    let red = just("r").to(Color::Red);
    let green = just("g").to(Color::Green);
    let colors = choice((white, blue, black, red, green));
    colors.repeated().at_least(1).collect::<Vec<_>>()
}

#[problem_day]
fn run(input: Day19, arguments: &CommandLineArguments) -> usize {
    let prefix_map = build_prefix_map(&input.available_towels);

    match arguments.towel_options {
        TowelOptions::TowelPossible => input
            .target_towels
            .into_iter()
            .filter(|target| is_possible(target, &prefix_map))
            .count(),
        TowelOptions::NumberOfWaysPossible => input
            .target_towels
            .into_iter()
            .map(|target| count_possible(&target, &prefix_map, &mut vec![None; target.len()]))
            .sum(),
    }
}

fn count_possible(
    target: &[Color],
    prefix_map: &AHashMap<Color, Vec<Vec<Color>>>,
    cache: &mut Vec<Option<usize>>,
) -> usize {
    if target.is_empty() {
        return 1;
    }

    if let Some(count) = cache.get(target.len() - 1).expect("Exists") {
        return *count;
    }

    let first = target.first().expect("Exists");
    let result = prefix_map
        .get(first)
        .map(|matches| {
            matches
                .iter()
                .filter(|match_| match_.len() <= target.len())
                .filter(|match_| {
                    match_
                        .iter()
                        .zip(target.iter())
                        .all(|(left, right)| left == right)
                })
                .map(|full_match| {
                    count_possible(target.split_at(full_match.len()).1, prefix_map, cache)
                })
                .sum()
        })
        .unwrap_or(0);
    *cache.get_mut(target.len() - 1).expect("Exists") = Some(result);

    result
}

fn is_possible(target: &[Color], prefix_map: &AHashMap<Color, Vec<Vec<Color>>>) -> bool {
    if target.is_empty() {
        return true;
    }

    let first = target.first().expect("Exists");
    prefix_map.get(first).is_some_and(|matches| {
        matches
            .iter()
            .filter(|match_| match_.len() <= target.len())
            .filter(|match_| {
                match_
                    .iter()
                    .zip(target.iter())
                    .all(|(left, right)| left == right)
            })
            .any(|full_match| is_possible(target.split_at(full_match.len()).1, prefix_map))
    })
}

fn build_prefix_map(available_towels: &[Vec<Color>]) -> AHashMap<Color, Vec<Vec<Color>>> {
    available_towels
        .iter()
        .filter_map(|towel| towel.first().map(|first| (first, towel.clone())))
        .fold(AHashMap::new(), |mut acc, (key, value)| {
            let entry = acc.entry(key.clone()).or_default();
            entry.push(value);
            acc
        })
}

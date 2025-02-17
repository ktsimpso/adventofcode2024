use crate::libs::{
    cli::{new_cli_problem, CliProblem, Freeze},
    graph::{Direction, PlanarCoordinate},
    parse::{parse_table2, ParserExt, StringParse},
    problem::Problem,
};
use adventofcode_macro::{problem_day, problem_parse};
use ahash::AHashMap;
use chumsky::{
    error::Rich,
    extra,
    prelude::{any, just},
    text, Parser,
};
use clap::{Args, ValueEnum};
use itertools::Itertools;
use ndarray::Array2;
use std::{iter::once, sync::LazyLock};

pub static DAY_08: LazyLock<CliProblem<Day08, CommandLineArguments, Freeze>> =
    LazyLock::new(|| {
        new_cli_problem(
            "day08",
            "Calculates the number of areas on a dish which have antinodes",
            "Dish grid with anteni and their frequencies",
        )
        .with_part(
            "The number of antinodes with discrete resonance",
            CommandLineArguments {
                resonance: Resonance::Discrete,
            },
            vec![("sample.txt", 14)],
        )
        .with_part(
            "The number of antinodes with harmonic resonance",
            CommandLineArguments {
                resonance: Resonance::Harmonic,
            },
            vec![("sample.txt", 34)],
        )
        .freeze()
    });

#[derive(ValueEnum, Clone)]
enum Resonance {
    Discrete,
    Harmonic,
}

#[derive(Args)]
pub struct CommandLineArguments {
    #[arg(
        short,
        long,
        help = "The type of resonance used to calculate antinodes"
    )]
    resonance: Resonance,
}

#[derive(Debug, Clone)]
enum Dish {
    Empty,
    Antena(char),
}

pub struct Day08(Array2<Dish>);

#[problem_parse]
fn parse<'a>() -> impl Parser<'a, &'a str, Day08, extra::Err<Rich<'a, char>>> {
    let empty = just(".").to(Dish::Empty);
    let antena = any()
        .and_is(just(".").not())
        .and_is(text::newline().not())
        .map(Dish::Antena);
    parse_table2(empty.or(antena)).map(Day08).end()
}

#[problem_day]
fn run(Day08(input): Day08, arguments: &CommandLineArguments) -> usize {
    input
        .indexed_iter()
        .filter(|(_, location)| matches!(location, Dish::Antena(_)))
        .fold(AHashMap::new(), |mut acc, (index, item)| {
            match item {
                Dish::Antena(key) => {
                    let items: &mut Vec<(usize, usize)> = acc.entry(key).or_default();
                    items.push(index);
                }
                _ => unreachable!(),
            }

            acc
        })
        .values()
        .flat_map(|antenas| {
            antenas
                .iter()
                .tuple_combinations()
                .flat_map(|(a, b)| antinodes_from_points(a, b, &arguments.resonance, &input.dim()))
        })
        .unique()
        .count()
}

fn antinodes_from_points(
    a: &(usize, usize),
    b: &(usize, usize),
    resonance: &Resonance,
    (y_size, x_size): &(usize, usize),
) -> Vec<(usize, usize)> {
    let x_diff = a.1.abs_diff(b.1);
    let y_diff = a.0.abs_diff(b.0);
    let (x_dir, y_dir) = a.relative_position_to(b);

    match resonance {
        Resonance::Discrete => once(a.jump_to(x_diff, x_dir, y_diff, y_dir))
            .chain(once(b.jump_to(
                x_diff,
                x_dir.get_opposite(),
                y_diff,
                y_dir.get_opposite(),
            )))
            .flatten()
            .filter(|(y, x)| y < y_size && x < x_size)
            .collect(),
        Resonance::Harmonic => a
            .into_iter_jumping(x_diff, x_dir, y_diff, y_dir)
            .take_while(|(y, x)| y < y_size && x < x_size)
            .chain(
                a.into_iter_jumping(x_diff, x_dir.get_opposite(), y_diff, y_dir.get_opposite())
                    .take_while(|(y, x)| y < y_size && x < x_size),
            )
            .chain(once(*a))
            .collect(),
    }
}

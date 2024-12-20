use crate::libs::{
    cli::{new_cli_problem, CliProblem, Freeze},
    graph::{BoundedPoint, Direction},
    parse::{parse_table2, StringParse},
    problem::Problem,
};
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

pub static DAY_08: LazyLock<CliProblem<Input, CommandLineArguments, Day08, Freeze>> =
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

pub struct Input(Array2<Dish>);

impl StringParse for Input {
    fn parse<'a>() -> impl Parser<'a, &'a str, Self, extra::Err<Rich<'a, char>>> {
        let empty = just(".").to(Dish::Empty);
        let antena = any()
            .and_is(just(".").not())
            .and_is(text::newline().not())
            .map(Dish::Antena);
        parse_table2(empty.or(antena)).map(Input)
    }
}

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

pub struct Day08 {}

impl Problem<Input, CommandLineArguments> for Day08 {
    type Output = usize;

    fn run(input: Input, arguments: &CommandLineArguments) -> Self::Output {
        let (max_x, max_y) = BoundedPoint::maxes_from_table(&input.0);

        input
            .0
            .indexed_iter()
            .filter(|(_, location)| matches!(location, Dish::Antena(_)))
            .fold(AHashMap::new(), |mut acc, (index, item)| {
                match item {
                    Dish::Antena(key) => {
                        let items: &mut Vec<BoundedPoint> = acc.entry(key).or_default();
                        items.push(BoundedPoint::from_table_index(index, max_x, max_y));
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
                    .flat_map(|(a, b)| (antinodes_from_points(a, b, &arguments.resonance)))
            })
            .unique()
            .count()
    }
}

fn antinodes_from_points(
    a: &BoundedPoint,
    b: &BoundedPoint,
    resonance: &Resonance,
) -> Vec<BoundedPoint> {
    let x_diff = a.x.abs_diff(b.x);
    let y_diff = a.y.abs_diff(b.y);
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
            .collect(),
        Resonance::Harmonic => a
            .into_iter_jumping(x_diff, x_dir, y_diff, y_dir)
            .chain(a.into_iter_jumping(x_diff, x_dir.get_opposite(), y_diff, y_dir.get_opposite()))
            .chain(once(*a))
            .collect(),
    }
}

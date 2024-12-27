use crate::libs::{
    cli::{new_cli_problem, CliProblem, Freeze},
    graph::BoundedPoint,
    parse::{parse_between_blank_lines, StringParse},
    problem::Problem,
};
use chumsky::{error::Rich, extra, prelude::just, text, IterParser, Parser};
use clap::Args;
use itertools::Itertools;
use ndarray::Array2;
use std::sync::LazyLock;

pub static DAY_25: LazyLock<CliProblem<Input, CommandLineArguments, Day25, Freeze>> =
    LazyLock::new(|| {
        new_cli_problem(
            "day25",
            "Finds how many keys can fit in all the locks",
            "Key and lock configurations separated by blank lines.",
        )
        .with_part(
            "Computes the sum of how many lock each key fits",
            CommandLineArguments {},
            vec![("sample.txt", 3)],
        )
        .freeze()
    });

pub struct Input(Vec<Array2<KeyHole>>);

impl StringParse for Input {
    fn parse<'a>() -> impl Parser<'a, &'a str, Self, extra::Err<Rich<'a, char>>> {
        let blocked = just("#").to(KeyHole::Blocked);
        let open = just(".").to(KeyHole::Open);
        let key_hole = blocked.or(open);

        let lock_key = key_hole
            .repeated()
            .at_least(1)
            .collect::<Vec<_>>()
            .separated_by(text::newline())
            .collect::<Vec<_>>()
            .try_map(|items, span| {
                let columns = items.first().map_or(0, |row| row.len());
                let rows = items.len();

                Array2::from_shape_vec(
                    (rows, columns),
                    items
                        .into_iter()
                        .fold(Vec::with_capacity(rows * columns), |mut acc, row| {
                            acc.extend(row);
                            acc
                        }),
                )
                .map_err(|op| Rich::custom(span, op))
            });

        parse_between_blank_lines(lock_key).map(Input)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum KeyHole {
    Blocked,
    Open,
}

#[derive(Args)]
pub struct CommandLineArguments {}

pub struct Day25 {}

impl Problem<Input, CommandLineArguments> for Day25 {
    type Output = usize;

    fn run(input: Input, _arguments: &CommandLineArguments) -> Self::Output {
        let (_, max_y) = BoundedPoint::maxes_from_table(input.0.first().expect("At least 1"));
        let (keys, locks): (Vec<_>, Vec<_>) = input
            .0
            .into_iter()
            .map(|lock_key| {
                let is_key = lock_key
                    .first()
                    .is_some_and(|slot| matches!(slot, KeyHole::Open));
                let counts = lock_key
                    .columns()
                    .into_iter()
                    .flat_map(|column| {
                        let column_height = column.len();
                        column
                            .into_iter()
                            .chunk_by(|key| (*key).clone())
                            .into_iter()
                            .next()
                            .map(|(_, chunk)| {
                                if is_key {
                                    column_height - chunk.count() - 1
                                } else {
                                    chunk.count() - 1
                                }
                            })
                    })
                    .collect::<Vec<_>>();

                (is_key, counts)
            })
            .partition(|(is_key, _)| *is_key);

        keys.into_iter()
            .map(|(_, key)| key)
            .cartesian_product(locks.into_iter().map(|(_, lock)| lock))
            .filter(|(key, lock)| {
                key.iter()
                    .zip(lock)
                    .map(|(k, l)| k + l)
                    .all(|size| size < max_y)
            })
            .count()
    }
}

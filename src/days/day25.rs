use crate::libs::{
    cli::{new_cli_problem, CliProblem, Freeze},
    graph::BoundedPoint,
    parse::{parse_between_blank_lines, parse_table2, StringParse},
    problem::Problem,
};
use adventofcode_macro::{problem_day, problem_parse};
use chumsky::{error::Rich, extra, prelude::just, Parser};
use clap::Args;
use itertools::Itertools;
use ndarray::Array2;
use std::sync::LazyLock;

pub static DAY_25: LazyLock<CliProblem<Day25, CommandLineArguments, Freeze>> =
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

#[derive(Args)]
pub struct CommandLineArguments {}

#[derive(Debug, Clone, PartialEq, Eq)]
enum KeyHole {
    Blocked,
    Open,
}

pub struct Day25(Vec<Array2<KeyHole>>);

#[problem_parse]
fn parse<'a>() -> impl Parser<'a, &'a str, Day25, extra::Err<Rich<'a, char>>> {
    let blocked = just("#").to(KeyHole::Blocked);
    let open = just(".").to(KeyHole::Open);
    let key_hole = blocked.or(open);

    parse_between_blank_lines(parse_table2(key_hole)).map(Day25)
}

#[problem_day]
fn run(Day25(input): Day25, _arguments: &CommandLineArguments) -> usize {
    let (_, max_y) = BoundedPoint::maxes_from_table(input.first().expect("At least 1"));
    let (keys, locks): (Vec<_>, Vec<_>) = input
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

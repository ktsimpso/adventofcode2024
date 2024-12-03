use crate::libs::{
    cli::{new_cli_problem, Command},
    math::absolute_difference,
    parse::{parse_lines, parse_usize, StringParse},
    problem::Problem,
};
use chumsky::{error::Rich, extra, prelude::just, IterParser, Parser};
use clap::Args;
use itertools::Itertools;
use std::{collections::HashSet, sync::LazyLock};

pub static DAY_02: LazyLock<Box<dyn Command + Send + Sync>> = LazyLock::new(|| {
    Box::new(
        new_cli_problem::<Input, CommandLineArguments, Day02>(
            "day02",
            "Determines the safety of reactor reports",
            "newline delimited lists of numbers. Within a line delimited by a space",
        )
        .with_part(
            "Computes the sum of the safe reports",
            CommandLineArguments {
                error_correction: false,
            },
        )
        .with_part(
            "Computes the sum of the safe reports once error correction is applied",
            CommandLineArguments {
                error_correction: true,
            },
        )
        .freeze(),
    )
});

struct Input(Vec<Vec<usize>>);

impl StringParse for Input {
    fn parse<'a>() -> impl Parser<'a, &'a str, Self, extra::Err<Rich<'a, char>>> {
        parse_lines(parse_usize().separated_by(just(" ")).at_least(1).collect()).map(Input)
    }
}

#[derive(Args)]
struct CommandLineArguments {
    #[arg(short, long, help = "Whether to apply error correction to the report")]
    error_correction: bool,
}

struct Day02 {}

impl Problem<Input, CommandLineArguments> for Day02 {
    type Output = usize;

    fn run(input: Input, arguments: &CommandLineArguments) -> Self::Output {
        let (valid, potentially_invalid): (Vec<_>, Vec<_>) = input
            .0
            .into_iter()
            .partition(|report| validate_report(report));

        if arguments.error_correction {
            valid.len()
                + potentially_invalid
                    .into_iter()
                    .filter(|report| validate_report_with_error(report))
                    .count()
        } else {
            valid.len()
        }
    }
}

fn validate_report_with_error(report: &[usize]) -> bool {
    let mut error_indices = HashSet::new();
    report
        .iter()
        .map_windows(|[a, b]| a > b)
        .find_position(|r| !r)
        .map(|r| r.0)
        .into_iter()
        .for_each(|error_index| {
            error_indices.insert(error_index);
            error_indices.insert(error_index + 1);
        });

    report
        .iter()
        .map_windows(|[a, b]| a < b)
        .find_position(|r| !r)
        .map(|r| r.0)
        .into_iter()
        .for_each(|error_index| {
            error_indices.insert(error_index);
            error_indices.insert(error_index + 1);
        });

    report
        .iter()
        .map_windows(|[a, b]| absolute_difference(**a, **b) <= 3)
        .find_position(|r| !r)
        .map(|r| r.0)
        .into_iter()
        .for_each(|error_index| {
            error_indices.insert(error_index);
            error_indices.insert(error_index + 1);
        });

    error_indices
        .into_iter()
        .any(|index| validate_report_with_skip(report, index))
}

fn validate_report_with_skip(report: &[usize], skip: usize) -> bool {
    report
        .iter()
        .skip_index(skip)
        .tuple_windows()
        .all_or(|(a, b)| a > b, |(a, b)| b > a)
        && report
            .iter()
            .skip_index(skip)
            .map_windows(|[a, b]| absolute_difference(**a, **b) <= 3)
            .all(|r| r)
}

fn validate_report(report: &[usize]) -> bool {
    report
        .iter()
        .tuple_windows()
        .all_or(|(a, b)| a > b, |(a, b)| b > a)
        && report
            .iter()
            .map_windows(|[a, b]| absolute_difference(**a, **b) <= 3)
            .all(|r| r)
}

struct IndexSkipper<I: Iterator> {
    base: I,
    index: usize,
    current: usize,
}

impl<I> Iterator for IndexSkipper<I>
where
    I: Iterator,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(v) = self.base.next() {
            let result = if self.index == self.current {
                self.base.next()
            } else {
                Some(v)
            };
            self.current += 1;
            result
        } else {
            None
        }
    }
}

trait IteratorExtension: Iterator {
    fn skip_index(self, index: usize) -> IndexSkipper<Self>
    where
        Self: Sized,
    {
        IndexSkipper {
            base: self,
            index,
            current: 0,
        }
    }

    fn all_or<F, G>(&mut self, mut predicate1: F, mut predicate2: G) -> bool
    where
        Self: Sized,
        F: FnMut(&Self::Item) -> bool,
        G: FnMut(&Self::Item) -> bool,
    {
        let mut all_one = true;
        let mut all_two = true;

        for item in self {
            if all_one && !predicate1(&item) {
                all_one = false;
            }

            if all_two && !predicate2(&item) {
                all_two = false;
            }

            if !all_one && !all_two {
                break;
            }
        }

        all_one || all_two
    }
}

impl<I: Iterator> IteratorExtension for I {}

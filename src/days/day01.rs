use crate::libs::{
    cli::{new_cli_problem, CliProblem, Freeze},
    parse::{parse_lines, parse_usize, StringParse},
    problem::Problem,
};
use adventofcode_macro::problem_day;
use ahash::AHashMap;
use chumsky::{error::Rich, extra, prelude::just, Parser};
use clap::{Args, ValueEnum};
use std::sync::LazyLock;

pub static DAY_01: LazyLock<CliProblem<Input, CommandLineArguments, Day01, Freeze>> =
    LazyLock::new(|| {
        new_cli_problem(
            "day01",
            "Interprets different lists of ids",
            "newline delimited lists of numbers with 2 numbers per line one for each list.",
        )
        .with_part(
            "Computes the sum of the difference between the like indexed ordered lists",
            CommandLineArguments {
                interpretation: ListInterpretation::Difference,
            },
            vec![("sample.txt", 11)],
        )
        .with_part(
            "Computes a score based on the frequency the first list item occurs in the second list",
            CommandLineArguments {
                interpretation: ListInterpretation::Similarity,
            },
            vec![("sample.txt", 31)],
        )
        .freeze()
    });

pub struct Input(Vec<(usize, usize)>);

impl StringParse for Input {
    fn parse<'a>() -> impl Parser<'a, &'a str, Self, extra::Err<Rich<'a, char>>> {
        parse_lines(parse_usize().then_ignore(just("   ")).then(parse_usize())).map(Input)
    }
}

#[derive(ValueEnum, Clone)]
enum ListInterpretation {
    Difference,
    Similarity,
}

#[derive(Args)]
pub struct CommandLineArguments {
    #[arg(short, long, help = "The interpretation of the lists")]
    interpretation: ListInterpretation,
}

#[problem_day(Day01)]
fn run(input: Input, arguments: &CommandLineArguments) -> usize {
    let (mut left, mut right) = input.0.into_iter().fold(
        (Vec::new(), Vec::new()),
        |(mut left_list, mut right_list), (left, right)| {
            left_list.push(left);
            right_list.push(right);
            (left_list, right_list)
        },
    );

    match arguments.interpretation {
        ListInterpretation::Difference => {
            left.sort();
            right.sort();
            compare_relative_values(left, right)
        }
        ListInterpretation::Similarity => similarity_score(left, right),
    }
}

fn compare_relative_values(left: Vec<usize>, right: Vec<usize>) -> usize {
    left.into_iter()
        .zip(right)
        .map(|(left, right)| left.abs_diff(right))
        .sum()
}

fn similarity_score(left: Vec<usize>, right: Vec<usize>) -> usize {
    let right = right.into_iter().fold(AHashMap::new(), |mut acc, number| {
        *acc.entry(number).or_insert(0) += 1;
        acc
    });

    left.into_iter()
        .map(|number| number * right.get(&number).unwrap_or(&0))
        .sum()
}

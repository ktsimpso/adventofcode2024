use crate::libs::{
    cli::{new_cli_problem, Command},
    math::absolute_difference,
    parse::{parse_lines, parse_usize, StringParse},
    problem::Problem,
};
use chumsky::{error::Rich, extra, prelude::just, Parser};
use clap::{Args, ValueEnum};
use std::{collections::HashMap, sync::LazyLock};

pub static DAY_01: LazyLock<Box<dyn Command + Send + Sync>> = LazyLock::new(|| {
    Box::new(
        new_cli_problem::<Input, CommandLineArguments, Day01>(
            "day01",
            "Interprets different lists of ids",
            "newline delimited lists of numbers with 2 numbers per line one for each list.",
        )
        .with_part(
            "Computes the sum of the difference between the like indexed ordered lists",
            CommandLineArguments {
                interpretation: ListInterpretation::Difference,
            },
        )
        .with_part(
            "Computes a score based on the frequency the first list item occurs in the second list",
            CommandLineArguments {
                interpretation: ListInterpretation::Similarity,
            },
        )
        .freeze(),
    )
});

struct Input(Vec<(usize, usize)>);

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
struct CommandLineArguments {
    #[arg(short, long, help = "The interpretation of the lists")]
    interpretation: ListInterpretation,
}

struct Day01 {}

impl Problem<Input, CommandLineArguments> for Day01 {
    type Output = usize;

    fn run(input: Input, arguments: &CommandLineArguments) -> Self::Output {
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
}

fn compare_relative_values(left: Vec<usize>, right: Vec<usize>) -> usize {
    left.into_iter()
        .zip(right)
        .map(|(left, right)| absolute_difference(left, right))
        .sum()
}

fn similarity_score(left: Vec<usize>, right: Vec<usize>) -> usize {
    let right = right.into_iter().fold(HashMap::new(), |mut acc, number| {
        *acc.entry(number).or_insert(0) += 1;
        acc
    });

    left.into_iter()
        .map(|number| number * right.get(&number).unwrap_or(&0))
        .sum()
}

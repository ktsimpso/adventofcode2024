use crate::libs::{
    cli::{new_cli_problem, CliProblem, Freeze},
    parse::{parse_lines, parse_usize, StringParse},
    problem::Problem,
};
use chumsky::{error::Rich, extra, prelude::just, IterParser, Parser};
use clap::{Args, ValueEnum};
use std::sync::LazyLock;

pub static DAY_07: LazyLock<CliProblem<Input, CommandLineArguments, Day07, Freeze>> = LazyLock::new(
    || {
        new_cli_problem(
            "day07",
            "Interprets different lists of ids",
            "Finds the number of results that can be satisfied by the test values with the given operators",
        )
        .with_part(
            "The sum of the valid values given Add and Multiply operators",
            CommandLineArguments {
                operators: vec![Operator::Add, Operator::Multiply],
            },
            vec![("sample.txt", 3749)],
        )
        .with_part(
            "The sum of the valid values given Add, Multiply, and Concat operators",
            CommandLineArguments {
                operators: vec![Operator::Add, Operator::Multiply, Operator::Concat],
            },
            vec![("sample.txt", 11387)],
        )
        .freeze()
    },
);

struct TestInput {
    result: usize,
    test_values: Vec<usize>,
}

pub struct Input(Vec<TestInput>);

impl StringParse for Input {
    fn parse<'a>() -> impl Parser<'a, &'a str, Self, extra::Err<Rich<'a, char>>> {
        let test_values = parse_usize().separated_by(just(" ")).at_least(1).collect();
        let test_input =
            parse_usize()
                .then_ignore(just(": "))
                .then(test_values)
                .map(|(result, test_values)| TestInput {
                    result,
                    test_values,
                });
        parse_lines(test_input).map(Input)
    }
}

#[derive(Args)]
pub struct CommandLineArguments {
    #[arg(short, long, num_args = 1..3, value_delimiter = ' ', required = true, help = "The list of operators to test")]
    operators: Vec<Operator>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, ValueEnum)]
enum Operator {
    Add,
    Multiply,
    Concat,
}

pub struct Day07 {}

impl Problem<Input, CommandLineArguments> for Day07 {
    type Output = usize;

    fn run(input: Input, arguments: &CommandLineArguments) -> Self::Output {
        input
            .0
            .into_iter()
            .filter(|test_input| {
                can_satisfy(
                    &test_input.test_values,
                    test_input.result,
                    &arguments.operators,
                )
            })
            .map(|test_input| test_input.result)
            .sum()
    }
}

fn can_satisfy(test_values: &[usize], target: usize, operators: &[Operator]) -> bool {
    test_values.split_last().is_some_and(|(last_value, rest)| {
        if rest.is_empty() {
            return *last_value == target;
        }

        operators.iter().any(|operator| match operator {
            Operator::Add => {
                if *last_value > target {
                    false
                } else {
                    can_satisfy(rest, target - last_value, operators)
                }
            }
            Operator::Multiply => {
                if target % *last_value != 0 {
                    false
                } else {
                    can_satisfy(rest, target / last_value, operators)
                }
            }
            Operator::Concat => {
                let base = last_value.ilog10();
                let value_digits = 10usize.pow(base + 1);
                if target % value_digits != *last_value {
                    false
                } else {
                    can_satisfy(rest, target / value_digits, operators)
                }
            }
        })
    })
}

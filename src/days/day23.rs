use crate::libs::{
    cli::{new_cli_problem, CliProblem, Freeze},
    parse::{parse_lines, StringParse},
    problem::{Problem, ProblemResult},
};
use adventofcode_macro::problem_day;
use ahash::{AHashMap, AHashSet};
use chumsky::{
    error::Rich,
    extra,
    prelude::{just, one_of},
    Parser,
};
use clap::{Args, ValueEnum};
use itertools::Itertools;
use std::sync::LazyLock;

pub static DAY_23: LazyLock<CliProblem<Input, CommandLineArguments, Day23, Freeze>> =
    LazyLock::new(|| {
        new_cli_problem(
            "day23",
            "Finds information about sets of connected computers",
            "Newline delimited lists of computer ids whom are connected.",
        )
        .with_part(
            "Counts all unique 3 fully connected networks where at least 1 computer starts with t",
            CommandLineArguments {
                connection_information: ConnectionInformation::MutualTruplesWithT,
            },
            vec![("sample.txt", 7_usize.into())],
        )
        .with_part(
            "Finds the max fully connected sub-network of computers",
            CommandLineArguments {
                connection_information: ConnectionInformation::MostMutualConnections,
            },
            vec![("sample.txt", "co,de,ka,ta".to_string().into())],
        )
        .freeze()
    });

pub struct Input(Vec<(String, String)>);

impl StringParse for Input {
    fn parse<'a>() -> impl Parser<'a, &'a str, Self, extra::Err<Rich<'a, char>>> {
        let alpha = one_of('a'..='z');
        let computer = alpha
            .clone()
            .then(alpha)
            .map(|(a, b)| format!("{}{}", a, b));

        parse_lines(computer.clone().then_ignore(just("-")).then(computer)).map(Input)
    }
}

#[derive(ValueEnum, Clone)]
enum ConnectionInformation {
    MutualTruplesWithT,
    MostMutualConnections,
}

#[derive(Args)]
pub struct CommandLineArguments {
    #[arg(short, long, help = "The type of connection information desired.")]
    connection_information: ConnectionInformation,
}

#[problem_day(Day23)]
fn run(input: Input, arguments: &CommandLineArguments) -> ProblemResult {
    let computer_to_connections = input.0.into_iter().fold(
        AHashMap::new(),
        |mut acc: AHashMap<String, AHashSet<String>>, (a, b)| {
            let a_entry = acc.entry(a.clone()).or_default();
            a_entry.insert(b.clone());
            let b_entry = acc.entry(b).or_default();
            b_entry.insert(a);
            acc
        },
    );

    match arguments.connection_information {
        ConnectionInformation::MutualTruplesWithT => {
            let mut visited = AHashSet::new();

            computer_to_connections
                .keys()
                .filter(|computer| computer.starts_with("t"))
                .map(|key| {
                    let connections = computer_to_connections.get(key).expect("Exists");
                    let mut local_visit = AHashSet::new();
                    let result = connections
                        .iter()
                        .filter(|connection| !visited.contains(connection))
                        .map(|connection| {
                            let connections_connections =
                                computer_to_connections.get(connection).expect("Exists");
                            let mutual_connections = connections
                                .intersection(connections_connections)
                                .filter(|connection| !visited.contains(connection))
                                .filter(|connection| !local_visit.contains(connection))
                                .count();

                            local_visit.insert(connection);

                            mutual_connections
                        })
                        .sum::<usize>();
                    visited.insert(key);
                    result
                })
                .sum::<usize>()
                .into()
        }
        ConnectionInformation::MostMutualConnections => get_most_mutual_connections(
            AHashSet::new(),
            computer_to_connections.keys().map(|s| s.as_str()).collect(),
            AHashSet::new(),
            0,
            &computer_to_connections,
        )
        .into_iter()
        .sorted()
        .join(",")
        .into(),
    }
}

fn get_most_mutual_connections<'a>(
    in_set: AHashSet<&'a str>,
    candidates: AHashSet<&'a str>,
    mut visited: AHashSet<&'a str>,
    best_found: usize,
    graph: &'a AHashMap<String, AHashSet<String>>,
) -> AHashSet<&'a str> {
    if in_set.len() + candidates.len() <= best_found {
        return AHashSet::new();
    }

    if candidates.is_empty() {
        return in_set;
    }

    let mut max = best_found;
    let mut max_set = AHashSet::new();

    candidates.iter().for_each(|computer| {
        visited.insert(computer);
        let best_connection = get_most_mutual_connections(
            in_set
                .union(&AHashSet::from([*computer]))
                .copied()
                .collect(),
            candidates
                .intersection(
                    &graph
                        .get(*computer)
                        .expect("Exists")
                        .iter()
                        .map(|s| s.as_str())
                        .collect::<AHashSet<&str>>(),
                )
                .filter(|computer| !visited.contains(**computer))
                .copied()
                .collect(),
            visited.clone(),
            max,
            graph,
        );

        if best_connection.len() > max {
            max = best_connection.len();
            max_set = best_connection;
        }
    });

    max_set
}

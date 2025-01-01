use crate::libs::{
    cli::{flag_arg, new_cli_problem, single_arg, CliArgs, CliProblem, Freeze},
    parse::{parse_isize, parse_lines, ParserExt, StringParse},
    problem::Problem,
};
use adventofcode_macro::{problem_day, problem_parse};
use ahash::AHashMap;
use chumsky::{error::Rich, extra, prelude::just, Parser};
use clap::value_parser;
use core::f64;
use itertools::Itertools;
use num_integer::Integer;
use std::{cmp::max, collections::HashSet, sync::LazyLock};

pub static DAY_14: LazyLock<CliProblem<Day14, CommandLineArguments, Freeze>> =
    LazyLock::new(|| {
        new_cli_problem(
            "day14",
            "Finds stats about robots in the bathroom",
            "Newline delimited list of robots with thier position and velocity.",
        )
        .with_part(
            "Computes the safety factor after a certain amount of time.",
            CommandLineArguments {
                x_size: 101,
                y_size: 103,
                robot_stat: RobotStat::SafetyFactor(100),
            },
            vec![],
        )
        .with_part(
            "Finds the time when the robots form a special image.",
            CommandLineArguments {
                x_size: 101,
                y_size: 103,
                robot_stat: RobotStat::FindTree(false),
            },
            vec![],
        )
        .freeze()
    });

enum RobotStat {
    SafetyFactor(usize),
    FindTree(bool),
}

pub struct CommandLineArguments {
    x_size: usize,
    y_size: usize,
    robot_stat: RobotStat,
}

impl CliArgs for CommandLineArguments {
    fn get_args() -> Vec<clap::Arg> {
        let x_size =
            single_arg("x_size", 'x', "The size of the x axis").value_parser(value_parser!(usize));
        let y_size =
            single_arg("y_size", 'y', "The size of the y axis").value_parser(value_parser!(usize));
        let safety_factory_after = single_arg(
            "safety_factory_after",
            's',
            "Output the safety factor after n iterations",
        )
        .value_parser(value_parser!(usize))
        .group("robot_stat")
        .conflicts_with_all(["find_tree", "print_tree"]);
        let find_tree = flag_arg("find_tree", 't', "Finds the secrete tree").group("robot_stat");
        let print_tree = flag_arg("print_tree", 'p', "Prints the tree after it has been found");
        vec![x_size, y_size, safety_factory_after, find_tree, print_tree]
    }

    fn parse_output(args: &clap::ArgMatches) -> Self {
        let x_size = *args.get_one::<usize>("x_size").expect("Required argument");
        let y_size = *args.get_one::<usize>("y_size").expect("Required argument");

        let safety_factor_after = args
            .get_one::<usize>("safety_factory_after")
            .map(|t| RobotStat::SafetyFactor(*t));

        let print_tree = args.get_flag("print_tree");

        match safety_factor_after {
            Some(robot_stat) => CommandLineArguments {
                x_size,
                y_size,
                robot_stat,
            },
            None => CommandLineArguments {
                x_size,
                y_size,
                robot_stat: RobotStat::FindTree(print_tree),
            },
        }
    }
}

#[derive(Debug)]
struct Robot {
    position: (isize, isize),
    horiztonal_velocity: isize,
    vertical_velocity: isize,
}

pub struct Day14(Vec<Robot>);

#[problem_parse]
fn parse<'a>() -> impl Parser<'a, &'a str, Day14, extra::Err<Rich<'a, char>>> {
    let robot = just("p=")
        .ignore_then(parse_isize())
        .then_ignore(just(","))
        .then(parse_isize())
        .then_ignore(just(" v="))
        .then(parse_isize())
        .then_ignore(just(","))
        .then(parse_isize())
        .map(|(((x, y), dx), dy)| Robot {
            position: (x, y),
            horiztonal_velocity: dx,
            vertical_velocity: dy,
        });
    parse_lines(robot).map(Day14).end()
}

#[derive(Debug, PartialEq, Eq, Hash)]
enum Quandrant {
    NorthWest,
    NorthEast,
    SouthEast,
    SouthWest,
}

#[problem_day]
fn run(Day14(input): Day14, arguments: &CommandLineArguments) -> isize {
    let x_size = arguments.x_size as isize;
    let y_size = arguments.y_size as isize;
    match arguments.robot_stat {
        RobotStat::SafetyFactor(t) => input
            .into_iter()
            .map(|robot| calculate_position_after(&robot, x_size, y_size, t as isize))
            .flat_map(|(x_pos, y_pos)| find_quadrant(x_pos, y_pos, x_size, y_size))
            .fold(AHashMap::new(), |mut acc, quandrant| {
                *acc.entry(quandrant).or_insert(0) += 1;
                acc
            })
            .values()
            .product(),
        RobotStat::FindTree(should_print_tree) => {
            let t = max(x_size, y_size);

            let mut min_y_index = 0;
            let mut min_y = f64::MAX;
            let mut min_x_index = 0;
            let mut min_x = f64::MAX;

            for i in 0..t {
                let new_positions = input
                    .iter()
                    .map(|robot| calculate_position_after(robot, x_size, y_size, i))
                    .collect::<Vec<_>>();

                if i < x_size {
                    let mean = new_positions.iter().map(|(x, _)| *x).sum::<isize>() as f64
                        / input.len() as f64;
                    let variance = new_positions
                        .iter()
                        .map(|(x, _)| *x)
                        .map(|x| x as f64 - mean)
                        .map(|x| x * x)
                        .sum::<f64>();

                    if variance < min_x {
                        min_x = variance;
                        min_x_index = i;
                    }
                }

                if i < y_size {
                    let mean = new_positions.iter().map(|(_, y)| *y).sum::<isize>() as f64
                        / input.len() as f64;
                    let variance = new_positions
                        .iter()
                        .map(|(_, y)| *y)
                        .map(|y| y as f64 - mean)
                        .map(|y| y * y)
                        .sum::<f64>();

                    if variance < min_y {
                        min_y = variance;
                        min_y_index = i;
                    }
                }
            }

            let result = find_alignment(min_x_index, x_size, min_y_index, y_size).expect("Exists");

            if should_print_tree {
                let tree = input
                    .iter()
                    .map(|robot| Robot {
                        position: calculate_position_after(robot, x_size, y_size, result),
                        ..*robot
                    })
                    .collect::<Vec<_>>();

                print_tree(&tree, x_size, y_size);
            }
            result
        }
    }
}

fn print_tree(robots: &[Robot], x_size: isize, y_size: isize) {
    let positions: HashSet<_> = robots.iter().map(|robot| robot.position).collect();
    let drones = (0..y_size)
        .map(|y| {
            (0..x_size)
                .map(|x| {
                    if positions.contains(&(x, y)) {
                        "O"
                    } else {
                        "."
                    }
                })
                .join("")
        })
        .join("\n");

    println!("{}\n", drones);
}

fn calculate_position_after(
    robot: &Robot,
    x_size: isize,
    y_size: isize,
    t: isize,
) -> (isize, isize) {
    let final_x_position = (robot.position.0 + robot.horiztonal_velocity * t).rem_euclid(x_size);
    let final_y_position = (robot.position.1 + robot.vertical_velocity * t).rem_euclid(y_size);

    (final_x_position, final_y_position)
}

fn find_quadrant(x_pos: isize, y_pos: isize, x_size: isize, y_size: isize) -> Option<Quandrant> {
    let mid_x = x_size / 2;
    let mid_y = y_size / 2;

    if x_pos < mid_x && y_pos < mid_y {
        Some(Quandrant::NorthWest)
    } else if x_pos > mid_x && y_pos < mid_y {
        Some(Quandrant::NorthEast)
    } else if x_pos < mid_x && y_pos > mid_y {
        Some(Quandrant::SouthWest)
    } else if x_pos > mid_x && y_pos > mid_y {
        Some(Quandrant::SouthEast)
    } else {
        None
    }
}

fn find_alignment(x: isize, x_period: isize, y: isize, y_period: isize) -> Option<isize> {
    let gcd = x_period.extended_gcd(&y_period);
    let phase_difference = x - y;

    if phase_difference % gcd.gcd != 0 {
        return None;
    }

    let combined_period = (x_period / gcd.gcd) * y_period;
    let combined_phase =
        (x - gcd.x * (phase_difference / gcd.gcd) * x_period).rem_euclid(combined_period);

    Some(combined_phase)
}

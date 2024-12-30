use crate::libs::{
    cli::{flag_arg, new_cli_problem, single_arg, CliArgs, CliProblem, Freeze},
    graph::BoundedPoint,
    parse::{parse_lines, parse_usize, StringParse},
    problem::{Problem, ProblemResult},
};
use adventofcode_macro::problem_day;
use chumsky::{error::Rich, extra, prelude::just, Parser};
use clap::value_parser;
use ndarray::Array2;
use std::{collections::VecDeque, sync::LazyLock};

pub static DAY_18: LazyLock<CliProblem<Input, CommandLineArguments, Day18, Freeze>> =
    LazyLock::new(|| {
        new_cli_problem(
            "day18",
            "Finds ways through corrupted memory.",
            "Newline delimited pairs of x,y coordinates of corruption.",
        )
        .with_part(
            "Computes the length of the shortest path after 1024 corruptions.",
            CommandLineArguments {
                x_size: 70,
                y_size: 70,
                path_stat: PathStat::ShortestPath(1024),
            },
            vec![],
        )
        .with_part(
            "Computes the first corruption that blocks the path to the exit.",
            CommandLineArguments {
                x_size: 70,
                y_size: 70,
                path_stat: PathStat::FirstBlockage,
            },
            vec![],
        )
        .freeze()
    });

pub struct Input(Vec<(usize, usize)>);

impl StringParse for Input {
    fn parse<'a>() -> impl Parser<'a, &'a str, Self, extra::Err<Rich<'a, char>>> {
        parse_lines(parse_usize().then_ignore(just(",")).then(parse_usize())).map(Input)
    }
}

enum PathStat {
    ShortestPath(usize),
    FirstBlockage,
}

pub struct CommandLineArguments {
    x_size: usize,
    y_size: usize,
    path_stat: PathStat,
}

impl CliArgs for CommandLineArguments {
    fn get_args() -> Vec<clap::Arg> {
        let x_size =
            single_arg("x_size", 'x', "The size of the x axis").value_parser(value_parser!(usize));
        let y_size =
            single_arg("y_size", 'y', "The size of the y axis").value_parser(value_parser!(usize));
        let shortest_path = single_arg(
            "shortest_path",
            's',
            "Output length of the shortest path after n corruptions",
        )
        .value_parser(value_parser!(usize))
        .group("path_stat")
        .conflicts_with("blockage");
        let blockage = flag_arg(
            "blockage",
            'b',
            "Find when the first corruption makes the path impossible",
        )
        .group("path_stat");
        vec![x_size, y_size, shortest_path, blockage]
    }

    fn parse_output(args: &clap::ArgMatches) -> Self {
        let x_size = *args.get_one::<usize>("x_size").expect("Required argument");
        let y_size = *args.get_one::<usize>("y_size").expect("Required argument");

        let shortest_path = args
            .get_one::<usize>("shortest_path")
            .map(|n| PathStat::ShortestPath(*n));

        let blockage = args.get_flag("blockage");

        match (shortest_path, blockage) {
            (None, true) => CommandLineArguments {
                x_size,
                y_size,
                path_stat: PathStat::FirstBlockage,
            },
            (Some(path_stat), false) => CommandLineArguments {
                x_size,
                y_size,
                path_stat,
            },
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone)]
enum Memory {
    Corrupted,
    Safe,
}

#[problem_day(Day18)]
fn run(input: Input, arguments: &CommandLineArguments) -> ProblemResult {
    match arguments.path_stat {
        PathStat::ShortestPath(n) => {
            let mut data =
                Array2::from_elem((arguments.x_size + 1, arguments.y_size + 1), Memory::Safe);

            input
                .0
                .into_iter()
                .take(n)
                .map(|(x, y)| BoundedPoint {
                    x,
                    y,
                    max_x: arguments.x_size,
                    max_y: arguments.y_size,
                })
                .for_each(|point| {
                    *data.get_mut((point.y, point.x)).expect("exists") = Memory::Corrupted;
                });

            let start = BoundedPoint {
                x: 0,
                y: 0,
                max_x: arguments.x_size,
                max_y: arguments.y_size,
            };

            let end = BoundedPoint {
                x: arguments.x_size,
                y: arguments.y_size,
                max_x: arguments.x_size,
                max_y: arguments.y_size,
            };

            shortest_path(&start, &end, &data).expect("Exists").into()
        }
        PathStat::FirstBlockage => {
            find_first_blockage(&input.0, arguments.x_size, arguments.y_size)
                .map(|(x, y)| format!("{},{}", x, y))
                .expect("Exists")
                .into()
        }
    }
}

#[derive(Debug, Clone)]
enum Color {
    Red,
    Blue,
    White,
    Blank,
}

fn find_first_blockage(
    blockages: &[(usize, usize)],
    max_x: usize,
    max_y: usize,
) -> Option<&(usize, usize)> {
    let mut data = Array2::from_elem((max_x + 1, max_y + 1), Color::Blank);

    blockages.iter().find(|(x, y)| {
        let point = BoundedPoint {
            x: *x,
            y: *y,
            max_x,
            max_y,
        };
        let is_blue = *x == max_x
            || *y == 0
            || point
                .into_iter_radial_adjacent()
                .flat_map(|point| point.get_from_table(&data))
                .any(|color| matches!(color, Color::Blue));
        let is_red = *x == 0
            || *y == max_y
            || point
                .into_iter_radial_adjacent()
                .flat_map(|point| point.get_from_table(&data))
                .any(|color| matches!(color, Color::Red));

        match (is_blue, is_red) {
            (true, true) => true,
            (true, false) => {
                *data.get_mut((point.y, point.x)).expect("Exists") = Color::Blue;
                color_neighbors(&point, Color::Blue, &mut data);
                false
            }
            (false, true) => {
                *data.get_mut((point.y, point.x)).expect("Exists") = Color::Red;
                color_neighbors(&point, Color::Red, &mut data);
                false
            }
            (false, false) => {
                *data.get_mut((point.y, point.x)).expect("Exists") = Color::White;
                false
            }
        }
    })
}

fn color_neighbors(point: &BoundedPoint, color: Color, data: &mut Array2<Color>) {
    point.into_iter_radial_adjacent().for_each(|adjacent| {
        let current_color = data.get_mut((adjacent.y, adjacent.x)).expect("Exists");
        if !matches!(current_color, Color::White) {
            return;
        }

        *current_color = color.clone();

        color_neighbors(&adjacent, color.clone(), data);
    });
}

fn shortest_path(start: &BoundedPoint, end: &BoundedPoint, data: &Array2<Memory>) -> Option<usize> {
    let mut queue = VecDeque::new();
    queue.push_back((*start, 0));

    let mut visited = Array2::from_elem((71, 71), false);
    let mut result = None;

    while let Some((point, distance)) = queue.pop_front() {
        if point == *end {
            result = Some(distance);
            break;
        }

        let visit = point.get_mut_from_table(&mut visited).expect("Exists");
        if *visit {
            continue;
        }
        *visit = true;

        get_adjacent(&point, data)
            .filter(|adjacent| !adjacent.get_from_table(&visited).expect("exists"))
            .for_each(|adjacent| {
                queue.push_back((adjacent, distance + 1));
            });
    }

    result
}

fn get_adjacent<'a>(
    point: &BoundedPoint,
    data: &'a Array2<Memory>,
) -> impl Iterator<Item = BoundedPoint> + 'a {
    point
        .into_iter_cardinal_adjacent()
        .filter(|adjacent| matches!(adjacent.get_from_table(data).expect("Exists"), Memory::Safe))
}

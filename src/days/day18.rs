use crate::libs::{
    cli::{flag_arg, new_cli_problem, single_arg, CliArgs, CliProblem, Freeze},
    graph::{breadth_first_search, BreadthFirstSearchLifecycle, PlanarCoordinate},
    parse::{parse_lines, parse_usize, ParserExt, StringParse},
    problem::{Problem, ProblemResult},
};
use adventofcode_macro::{problem_day, problem_parse};
use chumsky::{error::Rich, extra, prelude::just, Parser};
use clap::value_parser;
use ndarray::Array2;
use std::{collections::VecDeque, sync::LazyLock};

pub static DAY_18: LazyLock<CliProblem<Day18, CommandLineArguments, Freeze>> =
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

pub struct Day18(Vec<(usize, usize)>);

#[problem_parse]
fn parse<'a>() -> impl Parser<'a, &'a str, Day18, extra::Err<Rich<'a, char>>> {
    parse_lines(parse_usize().then_ignore(just(",")).then(parse_usize()))
        .map(Day18)
        .end()
}

#[derive(Debug, Clone)]
enum Memory {
    Corrupted,
    Safe,
}

#[problem_day]
fn run(Day18(input): Day18, arguments: &CommandLineArguments) -> ProblemResult {
    match arguments.path_stat {
        PathStat::ShortestPath(n) => {
            let mut data =
                Array2::from_elem((arguments.y_size + 1, arguments.x_size + 1), Memory::Safe);

            input
                .into_iter()
                .take(n)
                .map(|(x, y)| (y, x))
                .for_each(|point| *data.get_mut(point).expect("Exists") = Memory::Corrupted);

            shortest_path(&(0, 0), &(arguments.y_size, arguments.x_size), &data)
                .expect("Exists")
                .into()
        }
        PathStat::FirstBlockage => find_first_blockage(&input, arguments.x_size, arguments.y_size)
            .map(|(x, y)| format!("{},{}", x, y))
            .expect("Exists")
            .into(),
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
        let point = (*y, *x);
        let is_blue = *x == max_x
            || *y == 0
            || point
                .into_iter_radial_adjacent()
                .flat_map(|point| data.get(point))
                .any(|color| matches!(color, Color::Blue));
        let is_red = *x == 0
            || *y == max_y
            || point
                .into_iter_radial_adjacent()
                .flat_map(|point| data.get(point))
                .any(|color| matches!(color, Color::Red));

        match (is_blue, is_red) {
            (true, true) => true,
            (true, false) => {
                *data.get_mut(point).expect("Exists") = Color::Blue;
                color_neighbors(&point, Color::Blue, &mut data);
                false
            }
            (false, true) => {
                *data.get_mut(point).expect("Exists") = Color::Red;
                color_neighbors(&point, Color::Red, &mut data);
                false
            }
            (false, false) => {
                *data.get_mut(point).expect("Exists") = Color::White;
                false
            }
        }
    })
}

fn color_neighbors(point: &(usize, usize), color: Color, data: &mut Array2<Color>) {
    point.into_iter_radial_adjacent().for_each(|adjacent| {
        if data
            .get_mut(adjacent)
            .filter(|current_color| matches!(current_color, Color::White))
            .map(|current_color| {
                *current_color = color.clone();
                current_color
            })
            .is_some()
        {
            color_neighbors(&adjacent, color.clone(), data)
        }
    });
}

fn shortest_path(
    start: &(usize, usize),
    end: &(usize, usize),
    data: &Array2<Memory>,
) -> Option<usize> {
    let mut queue = VecDeque::new();
    queue.push_back((*start, 0_usize));

    let mut visited = Array2::from_elem(data.dim(), false);

    breadth_first_search(
        queue,
        &mut visited,
        &mut BreadthFirstSearchLifecycle::get_adjacent(|(point, distance)| {
            let new_distance = distance + 1;
            get_adjacent(point, data).map(move |new_point| (new_point, new_distance))
        })
        .with_first_visit(|(point, distance)| (point == end).then_some(*distance)),
    )
}

fn get_adjacent<'a>(
    point: &(usize, usize),
    data: &'a Array2<Memory>,
) -> impl Iterator<Item = (usize, usize)> + 'a {
    point.into_iter_cardinal_adjacent().filter(|adjacent| {
        data.get(*adjacent)
            .is_some_and(|value| matches!(value, Memory::Safe))
    })
}

use crate::libs::{
    cli::{CliProblem, Freeze, new_cli_problem},
    graph::{
        BoundedPoint, CARDINAL_DIRECTIONS, CardinalDirection, Direction, PlanarCoordinate,
        dijkstras,
    },
    parse::{ParserExt, StringParse, parse_table2},
    problem::Problem,
};
use adventofcode_macro::{StringParse, problem_day, problem_parse};
use ahash::AHashSet;
use chumsky::{
    Parser,
    error::Rich,
    extra,
    prelude::{choice, just},
};
use clap::{Args, ValueEnum};
use ndarray::{Array2, Array3};
use priority_queue::PriorityQueue;
use std::{cmp::Reverse, collections::VecDeque, iter::once, sync::LazyLock};

pub static DAY_16: LazyLock<CliProblem<Day16, CommandLineArguments, Freeze>> =
    LazyLock::new(|| {
        new_cli_problem(
            "day16",
            "Finds stats on the path through a maze",
            "2d maze with start and end points.",
        )
        .with_part(
            "Computes the total cost of the shortest path through the maze.",
            CommandLineArguments {
                path_stat: PathStat::ShortestWeight,
            },
            vec![("sample.txt", 7036), ("sample2.txt", 11048)],
        )
        .with_part(
            "Computes the number of unique tiles all shortest paths take through the maze.",
            CommandLineArguments {
                path_stat: PathStat::TotalSeats,
            },
            vec![("sample.txt", 45), ("sample2.txt", 64)],
        )
        .freeze()
    });

#[derive(ValueEnum, Clone)]
enum PathStat {
    ShortestWeight,
    TotalSeats,
}

#[derive(Args)]
pub struct CommandLineArguments {
    #[arg(short, long, help = "What stat about the maze to calculate")]
    path_stat: PathStat,
}

#[derive(Debug, Clone, StringParse)]
enum Maze {
    #[literal("S")]
    Start,
    #[literal("E")]
    End,
    #[literal(".")]
    Open,
    #[literal("#")]
    Wall,
}

pub struct Day16(Array2<Maze>);

#[problem_parse]
fn parse<'a>() -> impl Parser<'a, &'a str, Day16, extra::Err<Rich<'a, char>>> {
    parse_table2(Maze::parse()).map(Day16).end()
}

#[problem_day]
fn run(Day16(input): Day16, arguments: &CommandLineArguments) -> usize {
    let (max_x, max_y) = BoundedPoint::maxes_from_table(&input);

    let start = input
        .indexed_iter()
        .find(|(_, item)| matches!(item, Maze::Start))
        .map(|(index, _)| BoundedPoint::from_table_index(index, max_x, max_y))
        .expect("Exists");

    match arguments.path_stat {
        PathStat::ShortestWeight => {
            find_shortest_path_weight(&(start.y, start.x), &input).expect("Exists")
        }
        PathStat::TotalSeats => {
            find_all_shortest_paths(&(start.y, start.x), &input).expect("Exists")
        }
    }
}

fn find_all_shortest_paths(start: &(usize, usize), maze: &Array2<Maze>) -> Option<usize> {
    let mut queue = PriorityQueue::new();
    queue.push((*start, CardinalDirection::Right), Reverse(0));

    let mut visited = Array3::from_elem((maze.dim().0, maze.dim().1, 4), false);
    let mut visited_path = Array3::from_elem(
        (maze.dim().0, maze.dim().1, 4),
        (
            AHashSet::<(CardinalDirection, (usize, usize))>::new(),
            usize::MAX,
        ),
    );

    dijkstras(
        queue,
        &mut visited,
        |_| None,
        |((point, _), cost)| {
            maze.get(*point)
                .filter(|maze_type| matches!(maze_type, Maze::End))
                .map(|_| (*cost, *point))
        },
        |((point, direction), _)| get_valid_moves(direction, point, maze),
        |((point, direction), _), ((new_point, new_direction), new_cost)| {
            let path = visited_path
                .get_mut((new_point.0, new_point.1, new_direction.array_index()))
                .expect("Exists");

            match new_cost.cmp(&path.1) {
                std::cmp::Ordering::Less => {
                    path.0.clear();
                    path.0.insert((*direction, *point));
                    path.1 = *new_cost;
                }
                std::cmp::Ordering::Equal => {
                    path.0.insert((*direction, *point));
                }
                std::cmp::Ordering::Greater => (),
            }
        },
    )
    .map(|(score, end_point)| {
        let mut on_shortest_path = AHashSet::new();
        let mut path_queue = VecDeque::new();
        on_shortest_path.insert(&end_point);
        CARDINAL_DIRECTIONS.iter().for_each(|direction| {
            let (previous_points, priority) = visited_path
                .get((end_point.0, end_point.1, direction.array_index()))
                .expect("Exists");
            if *priority == score {
                previous_points.iter().for_each(|(direction, point)| {
                    on_shortest_path.insert(point);
                    path_queue.push_back((direction, point));
                });
            }
        });

        while let Some((direction, point)) = path_queue.pop_front() {
            let (previous_points, _) = visited_path
                .get((point.0, point.1, direction.array_index()))
                .expect("Exists");
            previous_points.iter().for_each(|(direction, point)| {
                on_shortest_path.insert(point);
                path_queue.push_back((direction, point));
            });
        }

        on_shortest_path.len()
    })
}

fn find_shortest_path_weight(start: &(usize, usize), maze: &Array2<Maze>) -> Option<usize> {
    let mut queue = PriorityQueue::new();
    queue.push((*start, CardinalDirection::Right), Reverse(0));

    let mut visited = Array3::from_elem((maze.dim().0, maze.dim().1, 4), false);

    dijkstras(
        queue,
        &mut visited,
        |_| None,
        |((point, _), cost)| {
            maze.get(*point)
                .filter(|maze_type| matches!(maze_type, Maze::End))
                .map(|_| *cost)
        },
        |((point, direction), _)| get_valid_moves(direction, point, maze),
        |_, _| (),
    )
}

fn get_valid_moves(
    direction: &CardinalDirection,
    point: &(usize, usize),
    maze: &Array2<Maze>,
) -> impl Iterator<Item = (((usize, usize), CardinalDirection), usize)> + use<> {
    point
        .get_adjacent(*direction)
        .filter(|point| {
            maze.get(*point)
                .is_some_and(|location| matches!(location, Maze::Open | Maze::End | Maze::Start))
        })
        .map(|point| ((point, *direction), 1))
        .into_iter()
        .chain(once(((*point, direction.get_clockwise()), 1000)))
        .chain(once(((*point, direction.get_counter_clockwise()), 1000)))
}

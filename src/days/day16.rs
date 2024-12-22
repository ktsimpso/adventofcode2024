use crate::libs::{
    cli::{new_cli_problem, CliProblem, Freeze},
    graph::{BoundedPoint, CardinalDirection, Direction, CARDINAL_DIRECTIONS},
    parse::{parse_table2, StringParse},
    problem::Problem,
};
use ahash::AHashSet;
use chumsky::{
    error::Rich,
    extra,
    prelude::{choice, just},
    Parser,
};
use clap::{Args, ValueEnum};
use ndarray::{Array2, Array3};
use priority_queue::PriorityQueue;
use std::{cmp::Reverse, collections::VecDeque, iter::once, sync::LazyLock};

pub static DAY_16: LazyLock<CliProblem<Input, CommandLineArguments, Day16, Freeze>> =
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

pub struct Input(Array2<Maze>);

impl StringParse for Input {
    fn parse<'a>() -> impl Parser<'a, &'a str, Self, extra::Err<Rich<'a, char>>> {
        let start = just("S").to(Maze::Start);
        let end = just("E").to(Maze::End);
        let open = just(".").to(Maze::Open);
        let wall = just("#").to(Maze::Wall);
        let maze = parse_table2(choice((start, end, open, wall)));
        maze.map(Input)
    }
}

#[derive(Debug, Clone)]
enum Maze {
    Start,
    End,
    Open,
    Wall,
}

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

pub struct Day16 {}

impl Problem<Input, CommandLineArguments> for Day16 {
    type Output = usize;

    fn run(input: Input, arguments: &CommandLineArguments) -> Self::Output {
        let (max_x, max_y) = BoundedPoint::maxes_from_table(&input.0);

        let start = input
            .0
            .indexed_iter()
            .find(|(_, item)| matches!(item, Maze::Start))
            .map(|(index, _)| BoundedPoint::from_table_index(index, max_x, max_y))
            .expect("Exists");

        match arguments.path_stat {
            PathStat::ShortestWeight => {
                find_shortest_path_weight(&start, &input.0).expect("Exists")
            }
            PathStat::TotalSeats => find_all_shortest_paths(&start, &input.0).expect("Exists"),
        }
    }
}

fn find_all_shortest_paths(start: &BoundedPoint, maze: &Array2<Maze>) -> Option<usize> {
    let (max_x, max_y) = BoundedPoint::maxes_from_table(maze);
    let mut queue = PriorityQueue::new();
    queue.push((CardinalDirection::Right, *start), Reverse(0));

    let mut result = None;
    let mut visited = Array3::from_elem((max_y + 1, max_x + 1, 4), false);
    let mut visited_path = Array3::from_elem(
        (max_y + 1, max_x + 1, 4),
        (
            AHashSet::<(CardinalDirection, BoundedPoint)>::new(),
            usize::MAX,
        ),
    );

    while let Some(((direction, point), priority)) = queue.pop() {
        if matches!(point.get_from_table(maze).expect("exists"), Maze::End) {
            result = Some((priority.0, point));
            break;
        }

        let has_visited = visited
            .get_mut((point.y, point.x, direction_to_index(&direction)))
            .expect("Exists");
        if *has_visited {
            continue;
        }

        *has_visited = true;

        get_valid_moves(&direction, &point, maze)
            .filter(|(direction, point, _)| {
                !visited
                    .get((point.y, point.x, direction_to_index(direction)))
                    .expect("Exists")
            })
            .for_each(|(new_direction, new_point, score)| {
                let current_priority = queue.get_priority(&(new_direction, new_point));
                let new_priority = priority.0 + score;

                let path = visited_path
                    .get_mut((new_point.y, new_point.x, direction_to_index(&new_direction)))
                    .expect("Exists");

                match new_priority.cmp(&path.1) {
                    std::cmp::Ordering::Less => {
                        path.0.clear();
                        path.0.insert((direction, point));
                        path.1 = new_priority;
                    }
                    std::cmp::Ordering::Equal => {
                        path.0.insert((direction, point));
                    }
                    std::cmp::Ordering::Greater => (),
                }

                match current_priority {
                    Some(current) => {
                        if new_priority < current.0 {
                            queue.change_priority(
                                &(new_direction, new_point),
                                Reverse(new_priority),
                            );
                        }
                    }
                    None => {
                        queue.push((new_direction, new_point), Reverse(new_priority));
                    }
                };
            });
    }

    result.map(|(score, end_point)| {
        let mut on_shortest_path = AHashSet::new();
        let mut path_queue = VecDeque::new();
        on_shortest_path.insert(&end_point);
        CARDINAL_DIRECTIONS.iter().for_each(|direction| {
            let (previous_points, priority) = visited_path
                .get((end_point.y, end_point.x, direction_to_index(direction)))
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
                .get((point.y, point.x, direction_to_index(direction)))
                .expect("Exists");
            previous_points.iter().for_each(|(direction, point)| {
                on_shortest_path.insert(point);
                path_queue.push_back((direction, point));
            });
        }

        on_shortest_path.len()
    })
}

fn find_shortest_path_weight(start: &BoundedPoint, maze: &Array2<Maze>) -> Option<usize> {
    let (max_x, max_y) = BoundedPoint::maxes_from_table(maze);
    let mut queue = PriorityQueue::new();
    queue.push((CardinalDirection::Right, *start), Reverse(0));

    let mut result = None;
    let mut visited = Array3::from_elem((max_y + 1, max_x + 1, 4), false);

    while let Some(((direction, point), priority)) = queue.pop() {
        if matches!(point.get_from_table(maze).expect("exists"), Maze::End) {
            result = Some(priority.0);
            break;
        }

        let has_visited = visited
            .get_mut((point.y, point.x, direction_to_index(&direction)))
            .expect("Exists");
        if *has_visited {
            continue;
        }

        *has_visited = true;

        get_valid_moves(&direction, &point, maze)
            .filter(|(direction, point, _)| {
                !visited
                    .get((point.y, point.x, direction_to_index(direction)))
                    .expect("Exists")
            })
            .for_each(|(new_direction, new_point, score)| {
                let current_priority = queue.get_priority(&(new_direction, new_point));
                let new_priority = priority.0 + score;

                match current_priority {
                    Some(current) => {
                        if new_priority < current.0 {
                            queue.change_priority(
                                &(new_direction, new_point),
                                Reverse(new_priority),
                            );
                        }
                    }
                    None => {
                        queue.push((new_direction, new_point), Reverse(new_priority));
                    }
                };
            });
    }

    result
}

fn direction_to_index(direction: &CardinalDirection) -> usize {
    match direction {
        CardinalDirection::Up => 0,
        CardinalDirection::Down => 1,
        CardinalDirection::Left => 2,
        CardinalDirection::Right => 3,
    }
}

fn get_valid_moves(
    direction: &CardinalDirection,
    point: &BoundedPoint,
    maze: &Array2<Maze>,
) -> impl Iterator<Item = (CardinalDirection, BoundedPoint, usize)> {
    point
        .get_adjacent(*direction)
        .filter(|point| {
            matches!(
                point.get_from_table(maze).expect("Exists"),
                Maze::Open | Maze::End | Maze::Start
            )
        })
        .map(|point| (*direction, point, 1))
        .into_iter()
        .chain(once((direction.get_clockwise(), *point, 1000)))
        .chain(once((direction.get_counter_clockwise(), *point, 1000)))
}

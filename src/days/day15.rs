use crate::libs::{
    cli::{new_cli_problem, CliProblem, Freeze},
    graph::{CardinalDirection, Direction, PlanarCoordinate},
    parse::{parse_lines, parse_table2, ParserExt, StringParse},
    problem::Problem,
};
use adventofcode_macro::{problem_day, problem_parse, StringParse};
use chumsky::{
    error::Rich,
    extra,
    prelude::{choice, just},
    text, IterParser, Parser,
};
use clap::Args;
use itertools::Itertools;
use ndarray::Array2;
use std::{iter::once, sync::LazyLock};

pub static DAY_15: LazyLock<CliProblem<Day15, CommandLineArguments, Freeze>> =
    LazyLock::new(|| {
        new_cli_problem(
            "day15",
            "Finds the gps score of the boxes in a warehouse after a robot moves",
            "The starting state of the warehouse followed by the robot movements.",
        )
        .with_part(
            "Computes gps score for a regular width warehouse.",
            CommandLineArguments { wide: false },
            vec![("sample2.txt", 2028), ("sample.txt", 10092)],
        )
        .with_part(
            "Computes gps score for a wide warehouse.",
            CommandLineArguments { wide: true },
            vec![("sample.txt", 9021)],
        )
        .freeze()
    });

#[derive(Args)]
pub struct CommandLineArguments {
    #[arg(short, long, help = "If the warehouse is wide or not")]
    wide: bool,
}

#[derive(Debug)]
pub struct Day15 {
    warehouse: Array2<WarehouseFloor>,
    movements: Vec<CardinalDirection>,
}

#[derive(Debug, Clone, StringParse)]
enum WarehouseFloor {
    #[literal("#")]
    Wall,
    #[literal(".")]
    Open,
    #[literal("O")]
    LeftBox,
    RightBox,
    #[literal("@")]
    Robot,
}

#[problem_parse]
fn parse<'a>() -> impl Parser<'a, &'a str, Day15, extra::Err<Rich<'a, char>>> {
    let up = just("^").to(CardinalDirection::Up);
    let down = just("v").to(CardinalDirection::Down);
    let left = just("<").to(CardinalDirection::Left);
    let right = just(">").to(CardinalDirection::Right);
    let direction = choice((up, down, left, right));

    let warehouse = parse_table2(WarehouseFloor::parse());

    let directions = parse_lines(direction.repeated().at_least(1).collect::<Vec<_>>())
        .map(|items| items.into_iter().flatten().collect::<Vec<_>>());

    warehouse
        .then_ignore(text::newline().repeated().at_least(1))
        .then(directions)
        .map(|(warehouse, movements)| Day15 {
            warehouse,
            movements,
        })
        .end()
}

#[problem_day]
fn run(
    Day15 {
        mut warehouse,
        movements,
    }: Day15,
    arguments: &CommandLineArguments,
) -> usize {
    if arguments.wide {
        let mut wide_warehouse = widen_warehouse(&warehouse);

        let mut robot_position = wide_warehouse
            .indexed_iter()
            .find(|(_, floor)| matches!(floor, WarehouseFloor::Robot))
            .map(|(index, _)| index)
            .expect("One robot exists");

        movements.into_iter().for_each(|movement| {
            robot_position = move_direction_wide(robot_position, movement, &mut wide_warehouse);
        });
        gps_score(&wide_warehouse)
    } else {
        let mut robot_position = warehouse
            .indexed_iter()
            .find(|(_, floor)| matches!(floor, WarehouseFloor::Robot))
            .map(|(index, _)| index)
            .expect("One robot exists");

        movements.into_iter().for_each(|movement| {
            robot_position = move_direction(robot_position, movement, &mut warehouse);
        });

        gps_score(&warehouse)
    }
}

fn widen_warehouse(warehouse: &Array2<WarehouseFloor>) -> Array2<WarehouseFloor> {
    Array2::from_shape_vec(
        (warehouse.dim().0, warehouse.dim().1 * 2),
        warehouse
            .into_iter()
            .flat_map(|tile| match tile {
                WarehouseFloor::Wall => [WarehouseFloor::Wall, WarehouseFloor::Wall],
                WarehouseFloor::Open => [WarehouseFloor::Open, WarehouseFloor::Open],
                WarehouseFloor::LeftBox => [WarehouseFloor::LeftBox, WarehouseFloor::RightBox],
                WarehouseFloor::Robot => [WarehouseFloor::Robot, WarehouseFloor::Open],
                WarehouseFloor::RightBox => unreachable!(),
            })
            .collect::<Vec<_>>(),
    )
    .expect("Works")
}

#[allow(dead_code)]
fn print_warehouse(warehouse: &Array2<WarehouseFloor>, wide: bool) {
    let warehouse = warehouse
        .rows()
        .into_iter()
        .map(|row| {
            row.into_iter()
                .map(|tile| match tile {
                    WarehouseFloor::Wall => "#",
                    WarehouseFloor::Open => ".",
                    WarehouseFloor::LeftBox => {
                        if wide {
                            "["
                        } else {
                            "O"
                        }
                    }
                    WarehouseFloor::RightBox => "]",
                    WarehouseFloor::Robot => "@",
                })
                .join("")
        })
        .join("\n");

    println!("{}", warehouse);
}

fn gps_score(warehouse: &Array2<WarehouseFloor>) -> usize {
    warehouse
        .indexed_iter()
        .filter(|(_, tile)| matches!(tile, WarehouseFloor::LeftBox))
        .map(|((y, x), _)| 100 * y + x)
        .sum()
}

fn move_direction_wide(
    robot_position: (usize, usize),
    direction: CardinalDirection,
    warehouse: &mut Array2<WarehouseFloor>,
) -> (usize, usize) {
    let adjacent = robot_position.get_adjacent(direction).expect("Exists");
    match warehouse.get(adjacent).expect("Exists") {
        WarehouseFloor::Wall => robot_position,
        WarehouseFloor::Open => {
            *warehouse.get_mut(adjacent).expect("Exists") = WarehouseFloor::Robot;
            *warehouse.get_mut(robot_position).expect("Exists") = WarehouseFloor::Open;
            adjacent
        }
        floor @ (WarehouseFloor::LeftBox | WarehouseFloor::RightBox) => match direction {
            CardinalDirection::Up | CardinalDirection::Down => {
                let companion = match floor {
                    WarehouseFloor::LeftBox => adjacent.get_adjacent(CardinalDirection::Right),
                    WarehouseFloor::RightBox => adjacent.get_adjacent(CardinalDirection::Left),
                    _ => unreachable!(),
                }
                .expect("Exists");

                let mut box_locations = vec![adjacent, companion];
                let mut finish_locations = Vec::new();
                let result = loop {
                    let adjacent_locations = box_locations
                        .into_iter()
                        .filter_map(|box_| {
                            let tile = warehouse.get(box_).expect("Exists");
                            let next = box_.get_adjacent(direction).expect("Exists");
                            finish_locations.push((box_, next, tile.clone()));
                            let next_tile = warehouse.get(next).expect("Exists");

                            if matches!(next_tile, WarehouseFloor::Open) {
                                return None;
                            }

                            Some(next)
                        })
                        .collect::<Vec<_>>();

                    if adjacent_locations
                        .iter()
                        .map(|location| warehouse.get(*location).expect("Exists"))
                        .any(|floor| matches!(floor, WarehouseFloor::Wall))
                    {
                        break robot_position;
                    }

                    if adjacent_locations.is_empty() {
                        finish_locations.into_iter().rev().for_each(
                            |(old_location, new_location, value)| {
                                *warehouse.get_mut(old_location).expect("Exists") =
                                    WarehouseFloor::Open;
                                *warehouse.get_mut(new_location).expect("Exists") = value;
                            },
                        );

                        *warehouse.get_mut(adjacent).expect("Exists") = WarehouseFloor::Robot;
                        *warehouse.get_mut(robot_position).expect("Exists") = WarehouseFloor::Open;
                        break adjacent;
                    }

                    box_locations = adjacent_locations
                        .into_iter()
                        .flat_map(|location| {
                            let box_ = warehouse.get(location).expect("Exists");
                            let companion = match box_ {
                                WarehouseFloor::LeftBox => {
                                    location.get_adjacent(CardinalDirection::Right)
                                }
                                WarehouseFloor::RightBox => {
                                    location.get_adjacent(CardinalDirection::Left)
                                }
                                WarehouseFloor::Open => None,
                                _ => unreachable!(),
                            };
                            once(location).chain(companion)
                        })
                        .unique()
                        .collect()
                };

                result
            }
            _ => adjacent
                .into_iter_direction(direction)
                .find(|point| {
                    let floor = warehouse.get(*point).expect("exists");
                    !matches!(floor, WarehouseFloor::LeftBox | WarehouseFloor::RightBox)
                })
                .filter(|space| {
                    let floor = warehouse.get(*space).expect("exists");
                    matches!(floor, WarehouseFloor::Open)
                })
                .map(|open_space| {
                    once(open_space)
                        .chain(open_space.into_iter_direction(direction.get_opposite()))
                        .take_while_inclusive(|point| *point != robot_position)
                        .tuple_windows()
                        .for_each(|(current, next)| {
                            *warehouse.get_mut(current).expect("Exists") =
                                warehouse.get(next).expect("Exists").clone();
                        });
                    *warehouse.get_mut(robot_position).expect("Exists") = WarehouseFloor::Open;

                    adjacent
                })
                .unwrap_or(robot_position),
        },
        _ => unreachable!(),
    }
}

fn move_direction(
    robot_position: (usize, usize),
    direction: CardinalDirection,
    warehouse: &mut Array2<WarehouseFloor>,
) -> (usize, usize) {
    let adjacent = robot_position.get_adjacent(direction).expect("Exists");
    match warehouse.get(adjacent).expect("Exists") {
        WarehouseFloor::Wall => robot_position,
        WarehouseFloor::Open => {
            *warehouse.get_mut(adjacent).expect("Exists") = WarehouseFloor::Robot;
            *warehouse.get_mut(robot_position).expect("Exists") = WarehouseFloor::Open;
            adjacent
        }
        WarehouseFloor::LeftBox => adjacent
            .into_iter_direction(direction)
            .find(|point| {
                let floor = warehouse.get(*point).expect("exists");
                !matches!(floor, WarehouseFloor::LeftBox)
            })
            .filter(|space| {
                let floor = warehouse.get(*space).expect("exists");
                matches!(floor, WarehouseFloor::Open)
            })
            .map(|open_space| {
                *warehouse.get_mut(open_space).expect("Exists") = WarehouseFloor::LeftBox;
                *warehouse.get_mut(adjacent).expect("Exists") = WarehouseFloor::Robot;
                *warehouse.get_mut(robot_position).expect("Exists") = WarehouseFloor::Open;

                adjacent
            })
            .unwrap_or(robot_position),
        _ => unreachable!(),
    }
}

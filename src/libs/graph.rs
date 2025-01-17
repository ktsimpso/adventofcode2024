use std::{
    collections::{HashSet, VecDeque},
    marker::PhantomData,
};

use ahash::AHashSet;
use ndarray::{Array2, Array3};
use subenum::subenum;

pub const CARDINAL_DIRECTIONS: [CardinalDirection; 4] = [
    CardinalDirection::Down,
    CardinalDirection::Left,
    CardinalDirection::Right,
    CardinalDirection::Up,
];

pub const DIAGNALS: [DiagnalDirection; 4] = [
    DiagnalDirection::UpRight,
    DiagnalDirection::DownRight,
    DiagnalDirection::DownLeft,
    DiagnalDirection::UpLeft,
];

pub const RADIAL_DIRECTIONS: [PointDirection; 8] = [
    PointDirection::Down,
    PointDirection::DownLeft,
    PointDirection::DownRight,
    PointDirection::Left,
    PointDirection::Right,
    PointDirection::Up,
    PointDirection::UpLeft,
    PointDirection::UpRight,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BoundedPoint {
    pub x: usize,
    pub y: usize,
    pub max_x: usize,
    pub max_y: usize,
}

impl BoundedPoint {
    pub fn maxes_from_table<T>(table: &Array2<T>) -> (usize, usize) {
        let max_x = table.dim().1 - 1;
        let max_y = table.dim().0 - 1;
        (max_x, max_y)
    }

    pub fn from_table_index((y, x): (usize, usize), max_x: usize, max_y: usize) -> Self {
        BoundedPoint { x, y, max_x, max_y }
    }

    pub fn get_from_table<'a, T>(&self, table: &'a Array2<T>) -> Option<&'a T> {
        table.get((self.y, self.x))
    }

    pub fn get_mut_from_table<'a, T>(&self, table: &'a mut Array2<T>) -> Option<&'a mut T> {
        table.get_mut((self.y, self.x))
    }

    pub fn insert_into_table<T>(&self, value: T, table: &mut Array2<T>) {
        *table.get_mut((self.y, self.x)).expect("position exists") = value;
    }

    pub fn get_adjacent_wrapping(self, point_direction: impl Into<PointDirection>) -> BoundedPoint {
        match point_direction.into() {
            PointDirection::Up => BoundedPoint {
                y: if self.y > 0 { self.y - 1 } else { self.max_y },
                ..self
            },
            PointDirection::Down => BoundedPoint {
                y: if self.y < self.max_y { self.y + 1 } else { 0 },
                ..self
            },
            PointDirection::Left => BoundedPoint {
                x: if self.x > 0 { self.x - 1 } else { self.max_x },
                ..self
            },
            PointDirection::Right => BoundedPoint {
                x: if self.x < self.max_x { self.x + 1 } else { 0 },
                ..self
            },
            PointDirection::UpRight => BoundedPoint {
                x: if self.x < self.max_x { self.x + 1 } else { 0 },
                y: if self.y > 0 { self.y - 1 } else { self.max_y },
                ..self
            },
            PointDirection::UpLeft => BoundedPoint {
                x: if self.x > 0 { self.x - 1 } else { self.max_x },
                y: if self.y > 0 { self.y - 1 } else { self.max_y },
                ..self
            },
            PointDirection::DownRight => BoundedPoint {
                x: if self.x < self.max_x { self.x + 1 } else { 0 },
                y: if self.y < self.max_y { self.y + 1 } else { 0 },
                ..self
            },
            PointDirection::DownLeft => BoundedPoint {
                x: if self.x > 0 { self.x - 1 } else { self.max_x },
                y: if self.y < self.max_y { self.y + 1 } else { 0 },
                ..self
            },
        }
    }
}

impl PlanarCoordinate for BoundedPoint {
    fn jump_to(
        &self,
        horizontal_distance: usize,
        horizontal_direction: HorizontalDirection,
        vertical_distance: usize,
        vertical_direction: VerticalDirection,
    ) -> Option<BoundedPoint> {
        match horizontal_direction {
            HorizontalDirection::Left => {
                if self.x >= horizontal_distance {
                    Some(self.x - horizontal_distance)
                } else {
                    None
                }
            }
            HorizontalDirection::Right => {
                if self.x + horizontal_distance <= self.max_x {
                    Some(self.x + horizontal_distance)
                } else {
                    None
                }
            }
        }
        .and_then(|x: usize| match vertical_direction {
            VerticalDirection::Up => {
                if self.y >= vertical_distance {
                    Some(BoundedPoint {
                        x,
                        y: self.y - vertical_distance,
                        ..*self
                    })
                } else {
                    None
                }
            }
            VerticalDirection::Down => {
                if self.y + vertical_distance <= self.max_y {
                    Some(BoundedPoint {
                        x,
                        y: self.y + vertical_distance,
                        ..*self
                    })
                } else {
                    None
                }
            }
        })
    }

    fn stride_to(
        &self,
        distance: usize,
        direction: impl Into<PointDirection>,
    ) -> Option<BoundedPoint> {
        match direction.into() {
            PointDirection::Up => {
                if distance <= self.y {
                    Some(BoundedPoint {
                        y: self.y - distance,
                        ..*self
                    })
                } else {
                    None
                }
            }
            PointDirection::Down => {
                if self.y + distance <= self.max_y {
                    Some(BoundedPoint {
                        y: self.y + distance,
                        ..*self
                    })
                } else {
                    None
                }
            }
            PointDirection::Left => {
                if distance <= self.x {
                    Some(BoundedPoint {
                        x: self.x - distance,
                        ..*self
                    })
                } else {
                    None
                }
            }
            PointDirection::Right => {
                if self.x + distance <= self.max_x {
                    Some(BoundedPoint {
                        x: self.x + distance,
                        ..*self
                    })
                } else {
                    None
                }
            }
            _ => todo!(),
        }
    }

    fn get_adjacent(&self, point_direction: impl Into<PointDirection>) -> Option<BoundedPoint> {
        match point_direction.into() {
            PointDirection::Up => {
                if self.y > 0 {
                    Some(BoundedPoint {
                        y: self.y - 1,
                        ..*self
                    })
                } else {
                    None
                }
            }
            PointDirection::Down => {
                if self.y < self.max_y {
                    Some(BoundedPoint {
                        y: self.y + 1,
                        ..*self
                    })
                } else {
                    None
                }
            }
            PointDirection::Left => {
                if self.x > 0 {
                    Some(BoundedPoint {
                        x: self.x - 1,
                        ..*self
                    })
                } else {
                    None
                }
            }
            PointDirection::Right => {
                if self.x < self.max_x {
                    Some(BoundedPoint {
                        x: self.x + 1,
                        ..*self
                    })
                } else {
                    None
                }
            }
            PointDirection::UpRight => {
                if self.y > 0 && self.x < self.max_x {
                    Some(BoundedPoint {
                        x: self.x + 1,
                        y: self.y - 1,
                        ..*self
                    })
                } else {
                    None
                }
            }
            PointDirection::UpLeft => {
                if self.y > 0 && self.x > 0 {
                    Some(BoundedPoint {
                        x: self.x - 1,
                        y: self.y - 1,
                        ..*self
                    })
                } else {
                    None
                }
            }
            PointDirection::DownRight => {
                if self.y < self.max_y && self.x < self.max_x {
                    Some(BoundedPoint {
                        x: self.x + 1,
                        y: self.y + 1,
                        ..*self
                    })
                } else {
                    None
                }
            }
            PointDirection::DownLeft => {
                if self.y < self.max_y && self.x > 0 {
                    Some(BoundedPoint {
                        x: self.x - 1,
                        y: self.y + 1,
                        ..*self
                    })
                } else {
                    None
                }
            }
        }
    }

    fn relative_horizontal_position_to(&self, other: &Self) -> HorizontalDirection {
        if self.x > other.x {
            HorizontalDirection::Right
        } else {
            HorizontalDirection::Left
        }
    }

    fn relative_vertical_position_to(&self, other: &Self) -> VerticalDirection {
        if self.y > other.y {
            VerticalDirection::Down
        } else {
            VerticalDirection::Up
        }
    }
}

#[derive(Debug, Clone)]
pub enum RotationDegrees {
    Zero,
    FortyFive,
    Ninety,
    OneHundredThirtyFive,
    OneHundredEighty,
    TwoHundredFive,
    TwoHundredSeventy,
    ThreeHundredFive,
}

pub trait Direction {
    fn get_rotation(&self, other: &Self) -> RotationDegrees;

    fn get_opposite(&self) -> Self;

    fn get_clockwise(&self) -> Self;

    fn get_counter_clockwise(&self) -> Self;

    fn array_index(&self) -> usize;
}

#[subenum(
    CardinalDirection,
    HorizontalDirection,
    VerticalDirection,
    DiagnalDirection
)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum PointDirection {
    #[subenum(CardinalDirection, VerticalDirection)]
    Up,
    #[subenum(DiagnalDirection)]
    UpRight,
    #[subenum(DiagnalDirection)]
    UpLeft,
    #[subenum(CardinalDirection, VerticalDirection)]
    Down,
    #[subenum(DiagnalDirection)]
    DownRight,
    #[subenum(DiagnalDirection)]
    DownLeft,
    #[subenum(CardinalDirection, HorizontalDirection)]
    Left,
    #[subenum(CardinalDirection, HorizontalDirection)]
    Right,
}

impl Direction for PointDirection {
    fn get_rotation(&self, other: &PointDirection) -> RotationDegrees {
        match (self, other) {
            (p1, p2) if p1 == p2 => RotationDegrees::Zero,
            (p1, p2) if &p1.get_clockwise() == p2 => RotationDegrees::FortyFive,
            (p1, p2) if &p1.get_clockwise().get_clockwise() == p2 => RotationDegrees::Ninety,
            (p1, p2) if &p1.get_opposite().get_counter_clockwise() == p2 => {
                RotationDegrees::OneHundredThirtyFive
            }
            (p1, p2) if &p1.get_opposite() == p2 => RotationDegrees::OneHundredEighty,
            (p1, p2) if &p1.get_opposite().get_clockwise() == p2 => RotationDegrees::TwoHundredFive,
            (p1, p2) if &p1.get_counter_clockwise().get_counter_clockwise() == p2 => {
                RotationDegrees::TwoHundredSeventy
            }
            _ => RotationDegrees::ThreeHundredFive,
        }
    }

    fn get_opposite(&self) -> PointDirection {
        match self {
            PointDirection::Up => PointDirection::Down,
            PointDirection::Down => PointDirection::Up,
            PointDirection::Left => PointDirection::Right,
            PointDirection::Right => PointDirection::Left,
            PointDirection::UpRight => PointDirection::DownLeft,
            PointDirection::UpLeft => PointDirection::DownRight,
            PointDirection::DownRight => PointDirection::UpLeft,
            PointDirection::DownLeft => PointDirection::DownRight,
        }
    }

    fn get_clockwise(&self) -> PointDirection {
        match self {
            PointDirection::Up => PointDirection::UpRight,
            PointDirection::UpRight => PointDirection::Right,
            PointDirection::Right => PointDirection::DownRight,
            PointDirection::DownRight => PointDirection::Down,
            PointDirection::Down => PointDirection::DownLeft,
            PointDirection::DownLeft => PointDirection::Left,
            PointDirection::Left => PointDirection::UpLeft,
            PointDirection::UpLeft => PointDirection::Up,
        }
    }

    fn get_counter_clockwise(&self) -> PointDirection {
        match self {
            PointDirection::Up => PointDirection::UpLeft,
            PointDirection::UpLeft => PointDirection::Left,
            PointDirection::Left => PointDirection::DownLeft,
            PointDirection::DownLeft => PointDirection::Down,
            PointDirection::Down => PointDirection::DownRight,
            PointDirection::DownRight => PointDirection::Right,
            PointDirection::Right => PointDirection::UpRight,
            PointDirection::UpRight => PointDirection::Up,
        }
    }

    fn array_index(&self) -> usize {
        match self {
            PointDirection::Up => 0,
            PointDirection::UpRight => 1,
            PointDirection::UpLeft => 2,
            PointDirection::Down => 3,
            PointDirection::DownRight => 4,
            PointDirection::DownLeft => 5,
            PointDirection::Left => 6,
            PointDirection::Right => 7,
        }
    }
}

impl Direction for CardinalDirection {
    fn get_rotation(&self, other: &CardinalDirection) -> RotationDegrees {
        match (self, other) {
            (p1, p2) if p1 == p2 => RotationDegrees::Zero,
            (p1, p2) if &p1.get_opposite() == p2 => RotationDegrees::OneHundredEighty,
            (p1, p2) if &p1.get_counter_clockwise() == p2 => RotationDegrees::TwoHundredSeventy,
            _ => RotationDegrees::Ninety,
        }
    }

    fn get_opposite(&self) -> CardinalDirection {
        match self {
            CardinalDirection::Up => CardinalDirection::Down,
            CardinalDirection::Down => CardinalDirection::Up,
            CardinalDirection::Left => CardinalDirection::Right,
            CardinalDirection::Right => CardinalDirection::Left,
        }
    }

    fn get_clockwise(&self) -> CardinalDirection {
        match self {
            CardinalDirection::Up => CardinalDirection::Right,
            CardinalDirection::Down => CardinalDirection::Left,
            CardinalDirection::Left => CardinalDirection::Up,
            CardinalDirection::Right => CardinalDirection::Down,
        }
    }

    fn get_counter_clockwise(&self) -> CardinalDirection {
        match self {
            CardinalDirection::Up => CardinalDirection::Left,
            CardinalDirection::Down => CardinalDirection::Right,
            CardinalDirection::Left => CardinalDirection::Down,
            CardinalDirection::Right => CardinalDirection::Up,
        }
    }

    fn array_index(&self) -> usize {
        match self {
            CardinalDirection::Up => 0,
            CardinalDirection::Down => 1,
            CardinalDirection::Left => 2,
            CardinalDirection::Right => 3,
        }
    }
}

impl From<HorizontalDirection> for CardinalDirection {
    fn from(value: HorizontalDirection) -> Self {
        match value {
            HorizontalDirection::Left => CardinalDirection::Left,
            HorizontalDirection::Right => CardinalDirection::Right,
        }
    }
}

impl From<VerticalDirection> for CardinalDirection {
    fn from(value: VerticalDirection) -> Self {
        match value {
            VerticalDirection::Up => CardinalDirection::Up,
            VerticalDirection::Down => CardinalDirection::Down,
        }
    }
}

impl Direction for DiagnalDirection {
    fn get_rotation(&self, other: &Self) -> RotationDegrees {
        match (self, other) {
            (p1, p2) if p1 == p2 => RotationDegrees::Zero,
            (p1, p2) if &p1.get_opposite() == p2 => RotationDegrees::OneHundredEighty,
            (p1, p2) if &p1.get_counter_clockwise() == p2 => RotationDegrees::TwoHundredSeventy,
            _ => RotationDegrees::Ninety,
        }
    }

    fn get_opposite(&self) -> Self {
        match self {
            DiagnalDirection::UpRight => Self::DownLeft,
            DiagnalDirection::UpLeft => Self::DownRight,
            DiagnalDirection::DownRight => Self::UpLeft,
            DiagnalDirection::DownLeft => Self::UpRight,
        }
    }

    fn get_clockwise(&self) -> Self {
        match self {
            DiagnalDirection::UpRight => Self::DownRight,
            DiagnalDirection::UpLeft => Self::UpRight,
            DiagnalDirection::DownRight => Self::DownLeft,
            DiagnalDirection::DownLeft => Self::UpLeft,
        }
    }

    fn get_counter_clockwise(&self) -> Self {
        match self {
            DiagnalDirection::UpRight => Self::UpLeft,
            DiagnalDirection::UpLeft => Self::DownLeft,
            DiagnalDirection::DownRight => Self::UpRight,
            DiagnalDirection::DownLeft => Self::DownRight,
        }
    }

    fn array_index(&self) -> usize {
        match self {
            DiagnalDirection::UpRight => 0,
            DiagnalDirection::UpLeft => 1,
            DiagnalDirection::DownRight => 2,
            DiagnalDirection::DownLeft => 3,
        }
    }
}

impl DiagnalDirection {
    pub fn to_horizontal_and_vertical(self) -> (HorizontalDirection, VerticalDirection) {
        match self {
            DiagnalDirection::UpRight => (HorizontalDirection::Right, VerticalDirection::Up),
            DiagnalDirection::UpLeft => (HorizontalDirection::Left, VerticalDirection::Up),
            DiagnalDirection::DownRight => (HorizontalDirection::Right, VerticalDirection::Down),
            DiagnalDirection::DownLeft => (HorizontalDirection::Left, VerticalDirection::Down),
        }
    }

    pub fn from_horziontal_and_vertical(
        horizatonal: &HorizontalDirection,
        vertical: &VerticalDirection,
    ) -> DiagnalDirection {
        match (horizatonal, vertical) {
            (HorizontalDirection::Left, VerticalDirection::Up) => Self::UpLeft,
            (HorizontalDirection::Left, VerticalDirection::Down) => Self::DownLeft,
            (HorizontalDirection::Right, VerticalDirection::Up) => Self::UpRight,
            (HorizontalDirection::Right, VerticalDirection::Down) => Self::DownRight,
        }
    }
}

impl Direction for HorizontalDirection {
    fn get_rotation(&self, other: &Self) -> RotationDegrees {
        if self == other {
            RotationDegrees::Zero
        } else {
            RotationDegrees::Ninety
        }
    }

    fn get_opposite(&self) -> Self {
        match self {
            HorizontalDirection::Left => HorizontalDirection::Right,
            HorizontalDirection::Right => HorizontalDirection::Left,
        }
    }

    fn get_clockwise(&self) -> Self {
        match self {
            HorizontalDirection::Left => HorizontalDirection::Right,
            HorizontalDirection::Right => HorizontalDirection::Left,
        }
    }

    fn get_counter_clockwise(&self) -> Self {
        match self {
            HorizontalDirection::Left => HorizontalDirection::Right,
            HorizontalDirection::Right => HorizontalDirection::Left,
        }
    }

    fn array_index(&self) -> usize {
        match self {
            HorizontalDirection::Left => 0,
            HorizontalDirection::Right => 1,
        }
    }
}

impl Direction for VerticalDirection {
    fn get_rotation(&self, other: &Self) -> RotationDegrees {
        if self == other {
            RotationDegrees::Zero
        } else {
            RotationDegrees::Ninety
        }
    }

    fn get_opposite(&self) -> Self {
        match self {
            VerticalDirection::Up => VerticalDirection::Down,
            VerticalDirection::Down => VerticalDirection::Up,
        }
    }

    fn get_clockwise(&self) -> Self {
        match self {
            VerticalDirection::Up => VerticalDirection::Down,
            VerticalDirection::Down => VerticalDirection::Up,
        }
    }

    fn get_counter_clockwise(&self) -> Self {
        match self {
            VerticalDirection::Up => VerticalDirection::Down,
            VerticalDirection::Down => VerticalDirection::Up,
        }
    }

    fn array_index(&self) -> usize {
        match self {
            VerticalDirection::Up => 0,
            VerticalDirection::Down => 1,
        }
    }
}

pub struct DirectionIntoIterator<T: PlanarCoordinate + Copy> {
    point: T,
    direction: PointDirection,
}

impl<T> Iterator for DirectionIntoIterator<T>
where
    T: PlanarCoordinate + Copy,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.point.get_adjacent(self.direction);
        result.iter().for_each(|point| self.point = *point);
        result
    }
}

pub struct CardinalAdjacentIterator<T: PlanarCoordinate> {
    point: T,
    index: usize,
}

impl<T> Iterator for CardinalAdjacentIterator<T>
where
    T: PlanarCoordinate,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= CARDINAL_DIRECTIONS.len() {
            return None;
        }
        let mut result = self.point.get_adjacent(CARDINAL_DIRECTIONS[self.index]);
        self.index += 1;

        result = match result {
            None => self.next(),
            _ => result,
        };
        result
    }
}

pub struct RadialAdjacentIterator<T: PlanarCoordinate> {
    point: T,
    index: usize,
}

impl<T> Iterator for RadialAdjacentIterator<T>
where
    T: PlanarCoordinate,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= RADIAL_DIRECTIONS.len() {
            return None;
        }
        let mut result = self.point.get_adjacent(RADIAL_DIRECTIONS[self.index]);
        self.index += 1;

        result = match result {
            None => self.next(),
            _ => result,
        };
        result
    }
}

pub struct JumpingIterator<T: PlanarCoordinate + Copy> {
    point: T,
    horizontal_length: usize,
    horizontal_direction: HorizontalDirection,
    vertical_length: usize,
    vertical_direction: VerticalDirection,
}

impl<T> Iterator for JumpingIterator<T>
where
    T: PlanarCoordinate + Copy,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(point) = self.point.jump_to(
            self.horizontal_length,
            self.horizontal_direction,
            self.vertical_length,
            self.vertical_direction,
        ) {
            self.point = point;
            Some(point)
        } else {
            None
        }
    }
}

pub trait PlanarCoordinate {
    fn into_iter_cardinal_adjacent(self) -> impl Iterator<Item = Self>
    where
        Self: Sized,
    {
        CardinalAdjacentIterator {
            point: self,
            index: 0,
        }
    }

    fn into_iter_radial_adjacent(self) -> impl Iterator<Item = Self>
    where
        Self: Sized,
    {
        RadialAdjacentIterator {
            point: self,
            index: 0,
        }
    }

    fn into_iter_direction(
        self,
        point_direction: impl Into<PointDirection>,
    ) -> impl Iterator<Item = Self>
    where
        Self: Copy,
    {
        DirectionIntoIterator {
            point: self,
            direction: point_direction.into(),
        }
    }

    fn into_iter_jumping(
        self,
        horizontal_length: usize,
        horizontal_direction: HorizontalDirection,
        vertical_length: usize,
        vertical_direction: VerticalDirection,
    ) -> impl Iterator<Item = Self>
    where
        Self: Copy,
    {
        JumpingIterator {
            point: self,
            horizontal_length,
            horizontal_direction,
            vertical_length,
            vertical_direction,
        }
    }

    fn jump_to(
        &self,
        horizontal_distance: usize,
        horizontal_direction: HorizontalDirection,
        vertical_distance: usize,
        vertical_direction: VerticalDirection,
    ) -> Option<Self>
    where
        Self: Sized;

    fn stride_to(&self, distance: usize, direction: impl Into<PointDirection>) -> Option<Self>
    where
        Self: Sized;

    fn get_adjacent(&self, point_direction: impl Into<PointDirection>) -> Option<Self>
    where
        Self: Sized;

    fn relative_horizontal_position_to(&self, other: &Self) -> HorizontalDirection;

    fn relative_vertical_position_to(&self, other: &Self) -> VerticalDirection;

    fn relative_position_to(&self, other: &Self) -> (HorizontalDirection, VerticalDirection) {
        (
            self.relative_horizontal_position_to(other),
            self.relative_vertical_position_to(other),
        )
    }
}

impl PlanarCoordinate for (usize, usize) {
    fn jump_to(
        &self,
        horizontal_distance: usize,
        horizontal_direction: HorizontalDirection,
        vertical_distance: usize,
        vertical_direction: VerticalDirection,
    ) -> Option<Self> {
        match horizontal_direction {
            HorizontalDirection::Left => self.1.checked_sub(horizontal_distance),
            HorizontalDirection::Right => Some(self.1 + horizontal_distance),
        }
        .and_then(|x| {
            match vertical_direction {
                VerticalDirection::Up => self.0.checked_sub(vertical_distance),
                VerticalDirection::Down => Some(self.0 + vertical_distance),
            }
            .map(|y| (y, x))
        })
    }

    fn stride_to(&self, distance: usize, direction: impl Into<PointDirection>) -> Option<Self> {
        match direction.into() {
            PointDirection::Up => self.0.checked_sub(distance).map(|result| (result, self.1)),
            PointDirection::UpRight => self
                .0
                .checked_sub(distance)
                .map(|result| (result, self.1 + distance)),
            PointDirection::UpLeft => self
                .0
                .checked_sub(distance)
                .and_then(|y| self.1.checked_sub(distance).map(|x| (y, x))),
            PointDirection::Down => Some((self.0 + distance, self.1)),
            PointDirection::DownRight => Some((self.0 + distance, self.1 + distance)),
            PointDirection::DownLeft => self
                .1
                .checked_sub(distance)
                .map(|result| (self.0 + distance, result)),
            PointDirection::Left => self.1.checked_sub(distance).map(|result| (self.0, result)),
            PointDirection::Right => Some((self.0, self.1 + distance)),
        }
    }

    fn get_adjacent(&self, point_direction: impl Into<PointDirection>) -> Option<Self> {
        match point_direction.into() {
            PointDirection::Up => self.0.checked_sub(1).map(|result| (result, self.1)),
            PointDirection::UpRight => self.0.checked_sub(1).map(|result| (result, self.1 + 1)),
            PointDirection::UpLeft => self
                .0
                .checked_sub(1)
                .and_then(|y| self.1.checked_sub(1).map(|x| (y, x))),
            PointDirection::Down => Some((self.0 + 1, self.1)),
            PointDirection::DownRight => Some((self.0 + 1, self.1 + 1)),
            PointDirection::DownLeft => self.1.checked_sub(1).map(|result| (self.0 + 1, result)),
            PointDirection::Left => self.1.checked_sub(1).map(|result| (self.0, result)),
            PointDirection::Right => Some((self.0, self.1 + 1)),
        }
    }

    fn relative_horizontal_position_to(&self, other: &Self) -> HorizontalDirection {
        if self.1 > other.1 {
            HorizontalDirection::Right
        } else {
            HorizontalDirection::Left
        }
    }

    fn relative_vertical_position_to(&self, other: &Self) -> VerticalDirection {
        if self.0 > other.0 {
            VerticalDirection::Down
        } else {
            VerticalDirection::Up
        }
    }
}

fn default_on_visit<T, R>(_value: &T) -> Option<R> {
    None
}

fn default_on_insert<T>(_value: &T, _adjacent: &T) {}

pub struct BreadthFirstSearchLifecycle<
    'a,
    const ON_REPEAT_VISIT: bool,
    const FIRST_VISIT: bool,
    const ON_INSERT: bool,
    T,
    I,
    R,
    E,
    F,
    G,
    H,
> where
    E: FnMut(&T) -> Option<R>,
    F: FnMut(&T) -> I,
    I: Iterator<Item = T> + 'a,
    G: FnMut(&T, &T),
    H: FnMut(&T) -> Option<R>,
{
    on_repeat_visit: E,
    first_visit: H,
    get_adjacent: F,
    on_insert: G,
    _marker: PhantomData<&'a (T, I, R)>,
}

impl<'a, T, I, F>
    BreadthFirstSearchLifecycle<
        'a,
        false,
        false,
        false,
        T,
        I,
        (),
        fn(&T) -> Option<()>,
        F,
        fn(&T, &T),
        fn(&T) -> Option<()>,
    >
where
    F: FnMut(&T) -> I,
    I: Iterator<Item = T> + 'a,
{
    pub fn get_adjacent<R>(
        get_adjacent: F,
    ) -> BreadthFirstSearchLifecycle<
        'a,
        false,
        false,
        false,
        T,
        I,
        R,
        impl FnMut(&T) -> Option<R>,
        F,
        impl FnMut(&T, &T),
        impl FnMut(&T) -> Option<R>,
    > {
        BreadthFirstSearchLifecycle {
            on_repeat_visit: default_on_visit,
            first_visit: default_on_visit,
            get_adjacent,
            on_insert: default_on_insert,
            _marker: PhantomData,
        }
    }
}

impl<'a, const ON_REPEAT_VISIT: bool, const FIRST_VISIT: bool, T, I, R, E, F, G, H>
    BreadthFirstSearchLifecycle<'a, ON_REPEAT_VISIT, FIRST_VISIT, false, T, I, R, E, F, G, H>
where
    E: FnMut(&T) -> Option<R>,
    F: FnMut(&T) -> I,
    I: Iterator<Item = T> + 'a,
    G: FnMut(&T, &T),
    H: FnMut(&T) -> Option<R>,
{
    pub fn with_on_insert(
        self,
        on_insert: impl FnMut(&T, &T),
    ) -> BreadthFirstSearchLifecycle<
        'a,
        ON_REPEAT_VISIT,
        FIRST_VISIT,
        true,
        T,
        I,
        R,
        E,
        F,
        impl FnMut(&T, &T),
        H,
    > {
        BreadthFirstSearchLifecycle {
            on_repeat_visit: self.on_repeat_visit,
            first_visit: self.first_visit,
            get_adjacent: self.get_adjacent,
            on_insert,
            _marker: PhantomData,
        }
    }
}

impl<'a, const ON_REPEAT_VISIT: bool, const ON_INSERT: bool, T, I, R, E, F, G, H>
    BreadthFirstSearchLifecycle<'a, ON_REPEAT_VISIT, false, ON_INSERT, T, I, R, E, F, G, H>
where
    E: FnMut(&T) -> Option<R>,
    F: FnMut(&T) -> I,
    I: Iterator<Item = T> + 'a,
    G: FnMut(&T, &T),
    H: FnMut(&T) -> Option<R>,
{
    pub fn with_first_visit(
        self,
        first_visit: impl FnMut(&T) -> Option<R>,
    ) -> BreadthFirstSearchLifecycle<
        'a,
        ON_REPEAT_VISIT,
        true,
        ON_INSERT,
        T,
        I,
        R,
        E,
        F,
        G,
        impl FnMut(&T) -> Option<R>,
    > {
        BreadthFirstSearchLifecycle {
            on_repeat_visit: self.on_repeat_visit,
            first_visit,
            get_adjacent: self.get_adjacent,
            on_insert: self.on_insert,
            _marker: PhantomData,
        }
    }
}

impl<'a, const FIRST_VISIT: bool, const ON_INSERT: bool, T, I, R, E, F, G, H>
    BreadthFirstSearchLifecycle<'a, false, FIRST_VISIT, ON_INSERT, T, I, R, E, F, G, H>
where
    E: FnMut(&T) -> Option<R>,
    F: FnMut(&T) -> I,
    I: Iterator<Item = T> + 'a,
    G: FnMut(&T, &T),
    H: FnMut(&T) -> Option<R>,
{
    pub fn with_on_repeat_visit(
        self,
        on_repeat_visit: impl FnMut(&T) -> Option<R>,
    ) -> BreadthFirstSearchLifecycle<
        'a,
        true,
        FIRST_VISIT,
        ON_INSERT,
        T,
        I,
        R,
        impl FnMut(&T) -> Option<R>,
        F,
        G,
        H,
    > {
        BreadthFirstSearchLifecycle {
            on_repeat_visit,
            first_visit: self.first_visit,
            get_adjacent: self.get_adjacent,
            on_insert: self.on_insert,
            _marker: PhantomData,
        }
    }
}

pub fn breadth_first_search<
    'a,
    const ON_REPEAT_VISIT: bool,
    const FIRST_VISIT: bool,
    const ON_INSERT: bool,
    T,
    I,
    R,
    E,
    F,
    G,
    H,
>(
    mut queue: VecDeque<T>,
    visitor: &mut impl Visitor<T>,
    lifecycle: &mut BreadthFirstSearchLifecycle<
        'a,
        ON_REPEAT_VISIT,
        FIRST_VISIT,
        ON_INSERT,
        T,
        I,
        R,
        E,
        F,
        G,
        H,
    >,
) -> Option<R>
where
    E: FnMut(&T) -> Option<R>,
    F: FnMut(&T) -> I,
    I: Iterator<Item = T> + 'a,
    G: FnMut(&T, &T),
    H: FnMut(&T) -> Option<R>,
{
    while let Some(value) = queue.pop_front() {
        if visitor.visit(&value) {
            match (lifecycle.on_repeat_visit)(&value) {
                r @ Some(_) => return r,
                None => continue,
            }
        }

        let stop = (lifecycle.first_visit)(&value);
        if stop.is_some() {
            return stop;
        }

        (lifecycle.get_adjacent)(&value)
            .filter(|adjacent| !visitor.has_visited(adjacent))
            .for_each(|adjacent| {
                (lifecycle.on_insert)(&value, &adjacent);
                queue.push_back(adjacent);
            })
    }

    None
}

pub trait Visitor<K> {
    fn visit(&mut self, key: &K) -> bool;

    fn has_visited(&self, key: &K) -> bool;
}

impl<D> Visitor<(BoundedPoint, D)> for Array3<bool>
where
    D: Direction,
{
    fn visit(&mut self, (point, direction): &(BoundedPoint, D)) -> bool {
        let visit = self
            .get_mut((point.y, point.x, direction.array_index()))
            .expect("Exists");
        if *visit {
            return true;
        }
        *visit = true;
        false
    }

    fn has_visited(&self, (point, direction): &(BoundedPoint, D)) -> bool {
        self.get((point.y, point.x, direction.array_index()))
            .is_some_and(|x| *x)
    }
}

impl<D> Visitor<((usize, usize), D)> for Array3<bool>
where
    D: Direction,
{
    fn visit(&mut self, ((y, x), direction): &((usize, usize), D)) -> bool {
        let visit = self
            .get_mut((*y, *x, direction.array_index()))
            .expect("Exists");
        if *visit {
            return true;
        }
        *visit = true;
        false
    }

    fn has_visited(&self, ((y, x), direction): &((usize, usize), D)) -> bool {
        self.get((*y, *x, direction.array_index()))
            .is_some_and(|x| *x)
    }
}

impl Visitor<(usize, usize)> for Array2<bool> {
    fn visit(&mut self, key: &(usize, usize)) -> bool {
        let visit = self.get_mut(*key).expect("Exists");
        if *visit {
            return true;
        }
        *visit = true;
        false
    }

    fn has_visited(&self, key: &(usize, usize)) -> bool {
        self.get(*key).is_some_and(|x| *x)
    }
}

impl<T> Visitor<((usize, usize), T)> for Array2<bool> {
    fn visit(&mut self, (key, _): &((usize, usize), T)) -> bool {
        let visit = self.get_mut(*key).expect("Exists");
        if *visit {
            return true;
        }
        *visit = true;
        false
    }

    fn has_visited(&self, (key, _): &((usize, usize), T)) -> bool {
        self.get(*key).is_some_and(|x| *x)
    }
}

impl<T: Clone> Visitor<((usize, usize), T)> for Array2<Option<T>> {
    fn visit(&mut self, (key, value): &((usize, usize), T)) -> bool {
        let visit = self.get_mut(*key).expect("Exists");
        if visit.is_some() {
            return true;
        }
        *visit = Some(value.clone());
        false
    }

    fn has_visited(&self, (key, _): &((usize, usize), T)) -> bool {
        self.get(*key).is_some_and(|x| x.is_some())
    }
}

impl Visitor<BoundedPoint> for Array2<bool> {
    fn visit(&mut self, key: &BoundedPoint) -> bool {
        let visit = key.get_mut_from_table(self).expect("Exists");
        if *visit {
            return true;
        }
        *visit = true;
        false
    }

    fn has_visited(&self, key: &BoundedPoint) -> bool {
        key.get_from_table(self).is_some_and(|x| *x)
    }
}

impl<T> Visitor<(BoundedPoint, T)> for Array2<bool> {
    fn visit(&mut self, (key, _): &(BoundedPoint, T)) -> bool {
        let visit = key.get_mut_from_table(self).expect("Exists");
        if *visit {
            return true;
        }
        *visit = true;
        false
    }

    fn has_visited(&self, (key, _): &(BoundedPoint, T)) -> bool {
        key.get_from_table(self).is_some_and(|x| *x)
    }
}

impl<T: Clone> Visitor<(BoundedPoint, T)> for Array2<Option<T>> {
    fn visit(&mut self, (key, value): &(BoundedPoint, T)) -> bool {
        let visit = key.get_mut_from_table(self).expect("Exists");
        if visit.is_some() {
            return true;
        }
        *visit = Some(value.clone());
        false
    }

    fn has_visited(&self, (key, _): &(BoundedPoint, T)) -> bool {
        key.get_from_table(self).is_some_and(|x| x.is_some())
    }
}

impl<T> Visitor<T> for AHashSet<T>
where
    T: Eq + std::hash::Hash + Clone,
{
    fn visit(&mut self, key: &T) -> bool {
        if self.contains(key) {
            return true;
        }
        self.insert(key.clone());
        false
    }

    fn has_visited(&self, key: &T) -> bool {
        self.contains(key)
    }
}

impl<T, U> Visitor<(T, U)> for AHashSet<T>
where
    T: Eq + std::hash::Hash + Clone,
{
    fn visit(&mut self, (key, _): &(T, U)) -> bool {
        if self.contains(key) {
            return true;
        }
        self.insert(key.clone());
        false
    }

    fn has_visited(&self, (key, _): &(T, U)) -> bool {
        self.contains(key)
    }
}

impl<T> Visitor<T> for HashSet<T>
where
    T: Eq + std::hash::Hash + Clone,
{
    fn visit(&mut self, key: &T) -> bool {
        if self.contains(key) {
            return true;
        }
        self.insert(key.clone());
        false
    }

    fn has_visited(&self, key: &T) -> bool {
        self.contains(key)
    }
}

impl<T, U> Visitor<(T, U)> for HashSet<T>
where
    T: Eq + std::hash::Hash + Clone,
{
    fn visit(&mut self, (key, _): &(T, U)) -> bool {
        if self.contains(key) {
            return true;
        }
        self.insert(key.clone());
        false
    }

    fn has_visited(&self, (key, _): &(T, U)) -> bool {
        self.contains(key)
    }
}

impl Visitor<usize> for Vec<bool> {
    fn visit(&mut self, key: &usize) -> bool {
        let visit = self.get_mut(*key).expect("Exists");
        if *visit {
            return true;
        }
        *visit = true;
        false
    }

    fn has_visited(&self, key: &usize) -> bool {
        self.get(*key).is_some_and(|x| *x)
    }
}

impl<T> Visitor<(usize, T)> for Vec<bool> {
    fn visit(&mut self, (key, _): &(usize, T)) -> bool {
        let visit = self.get_mut(*key).expect("Exists");
        if *visit {
            return true;
        }
        *visit = true;
        false
    }

    fn has_visited(&self, (key, _): &(usize, T)) -> bool {
        self.get(*key).is_some_and(|x| *x)
    }
}

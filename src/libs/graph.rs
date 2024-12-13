use ndarray::Array2;
use subenum::subenum;

pub const CARDINAL_DIRECTIONS: [CardinalDirection; 4] = [
    CardinalDirection::Down,
    CardinalDirection::Left,
    CardinalDirection::Right,
    CardinalDirection::Up,
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

    pub fn insert_into_table<T>(&self, value: T, table: &mut Array2<T>) {
        *table.get_mut((self.y, self.x)).expect("position exists") = value;
    }

    pub fn into_iter_direction(
        self,
        point_direction: impl Into<PointDirection>,
    ) -> BoundedPointIntoIterator {
        BoundedPointIntoIterator {
            point: self,
            direction: point_direction.into(),
        }
    }

    pub fn relative_horizontal_position_to(&self, other: &Self) -> HorizontalDirection {
        if self.x > other.x {
            HorizontalDirection::Right
        } else {
            HorizontalDirection::Left
        }
    }

    pub fn relative_vertical_position_to(&self, other: &Self) -> VerticalDirection {
        if self.y > other.y {
            VerticalDirection::Down
        } else {
            VerticalDirection::Up
        }
    }

    pub fn relative_position_to(&self, other: &Self) -> (HorizontalDirection, VerticalDirection) {
        (
            self.relative_horizontal_position_to(other),
            self.relative_vertical_position_to(other),
        )
    }

    pub fn jump_to(
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

    pub fn stride_to(
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

    pub fn into_iter_jumping(
        self,
        horizontal_length: usize,
        horizontal_direction: HorizontalDirection,
        vertical_length: usize,
        vertical_direction: VerticalDirection,
    ) -> JumpingIterator {
        JumpingIterator {
            point: self,
            horizontal_length,
            horizontal_direction,
            vertical_length,
            vertical_direction,
        }
    }

    pub fn into_iter_cardinal_adjacent(self) -> CardinalAdjacentIterator {
        CardinalAdjacentIterator {
            point: self,
            index: 0,
        }
    }

    pub fn into_iter_radial_adjacent(self) -> RadialAdjacentIterator {
        RadialAdjacentIterator {
            point: self,
            index: 0,
        }
    }

    pub fn get_adjacent(self, point_direction: impl Into<PointDirection>) -> Option<BoundedPoint> {
        match point_direction.into() {
            PointDirection::Up => {
                if self.y > 0 {
                    Some(BoundedPoint {
                        y: self.y - 1,
                        ..self
                    })
                } else {
                    None
                }
            }
            PointDirection::Down => {
                if self.y < self.max_y {
                    Some(BoundedPoint {
                        y: self.y + 1,
                        ..self
                    })
                } else {
                    None
                }
            }
            PointDirection::Left => {
                if self.x > 0 {
                    Some(BoundedPoint {
                        x: self.x - 1,
                        ..self
                    })
                } else {
                    None
                }
            }
            PointDirection::Right => {
                if self.x < self.max_x {
                    Some(BoundedPoint {
                        x: self.x + 1,
                        ..self
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
                        ..self
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
                        ..self
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
                        ..self
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
                        ..self
                    })
                } else {
                    None
                }
            }
        }
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
}

#[subenum(CardinalDirection, HorizontalDirection, VerticalDirection)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum PointDirection {
    #[subenum(CardinalDirection, VerticalDirection)]
    Up,
    UpRight,
    UpLeft,
    #[subenum(CardinalDirection, VerticalDirection)]
    Down,
    DownRight,
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
}

pub struct BoundedPointIntoIterator {
    point: BoundedPoint,
    direction: PointDirection,
}

impl Iterator for BoundedPointIntoIterator {
    type Item = BoundedPoint;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.point.get_adjacent(self.direction);
        result.iter().for_each(|point| self.point = *point);
        result
    }
}

pub struct CardinalAdjacentIterator {
    point: BoundedPoint,
    index: usize,
}

impl Iterator for CardinalAdjacentIterator {
    type Item = BoundedPoint;

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

pub struct RadialAdjacentIterator {
    point: BoundedPoint,
    index: usize,
}

impl Iterator for RadialAdjacentIterator {
    type Item = BoundedPoint;

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

pub struct JumpingIterator {
    point: BoundedPoint,
    horizontal_length: usize,
    horizontal_direction: HorizontalDirection,
    vertical_length: usize,
    vertical_direction: VerticalDirection,
}

impl Iterator for JumpingIterator {
    type Item = BoundedPoint;

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

use ndarray::Array2;

pub const CARDINAL_DIRECTIONS: [PointDirection; 4] = [
    PointDirection::Down,
    PointDirection::Left,
    PointDirection::Right,
    PointDirection::Up,
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

    pub fn into_iter_direction(self, point_direction: PointDirection) -> BoundedPointIntoIterator {
        BoundedPointIntoIterator {
            point: self,
            direction: point_direction,
        }
    }

    pub fn relative_horizontal_position_to(&self, other: &Self) -> PointDirection {
        if self.x > other.x {
            PointDirection::Right
        } else {
            PointDirection::Left
        }
    }

    pub fn relative_vertical_position_to(&self, other: &Self) -> PointDirection {
        if self.y > other.y {
            PointDirection::Down
        } else {
            PointDirection::Up
        }
    }

    pub fn relative_position_to(&self, other: &Self) -> (PointDirection, PointDirection) {
        (
            self.relative_horizontal_position_to(other),
            self.relative_vertical_position_to(other),
        )
    }

    pub fn jump_to(
        &self,
        horizontal_distance: usize,
        horizontal_direction: PointDirection,
        vertical_distance: usize,
        vertical_direction: PointDirection,
    ) -> Option<BoundedPoint> {
        match horizontal_direction {
            PointDirection::Left => {
                if self.x >= horizontal_distance {
                    Some(self.x - horizontal_distance)
                } else {
                    None
                }
            }
            PointDirection::Right => {
                if self.x + horizontal_distance <= self.max_x {
                    Some(self.x + horizontal_distance)
                } else {
                    None
                }
            }
            _ => unreachable!(),
        }
        .and_then(|x| match vertical_direction {
            PointDirection::Up => {
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
            PointDirection::Down => {
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
            _ => unreachable!(),
        })
    }

    pub fn stride_to(&self, distance: usize, direction: PointDirection) -> Option<BoundedPoint> {
        match direction {
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
        horizontal_direction: PointDirection,
        vertical_length: usize,
        vertical_direction: PointDirection,
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

    pub fn get_adjacent(self, point_direction: &PointDirection) -> Option<BoundedPoint> {
        match point_direction {
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

    pub fn get_adjacent_wrapping(self, point_direction: &PointDirection) -> BoundedPoint {
        match point_direction {
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
    Ninety,
    OneHundredEighty,
    TwoHundredSeventy,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum PointDirection {
    Up,
    UpRight,
    UpLeft,
    Down,
    DownRight,
    DownLeft,
    Left,
    Right,
}

impl PointDirection {
    pub fn get_rotation(&self, other: &PointDirection) -> RotationDegrees {
        match (self, other) {
            (p1, p2) if p1 == p2 => RotationDegrees::Zero,
            (p1, p2) if &p1.get_opposite() == p2 => RotationDegrees::OneHundredEighty,
            (p1, p2) if &p1.get_counter_clockwise() == p2 => RotationDegrees::TwoHundredSeventy,
            _ => RotationDegrees::Ninety,
        }
    }

    pub fn get_opposite(&self) -> PointDirection {
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

    pub fn get_clockwise(&self) -> PointDirection {
        match self {
            PointDirection::Up => PointDirection::Right,
            PointDirection::Down => PointDirection::Left,
            PointDirection::Left => PointDirection::Up,
            PointDirection::Right => PointDirection::Down,
            // this only kind of makes sense, might want cardinal and radial versions of this function
            PointDirection::UpRight => PointDirection::DownRight,
            PointDirection::UpLeft => PointDirection::UpRight,
            PointDirection::DownRight => PointDirection::DownLeft,
            PointDirection::DownLeft => PointDirection::UpLeft,
        }
    }

    pub fn get_counter_clockwise(&self) -> PointDirection {
        match self {
            PointDirection::Up => PointDirection::Left,
            PointDirection::Down => PointDirection::Right,
            PointDirection::Left => PointDirection::Down,
            PointDirection::Right => PointDirection::Up,
            // ditto for here
            PointDirection::UpRight => PointDirection::UpLeft,
            PointDirection::UpLeft => PointDirection::DownLeft,
            PointDirection::DownRight => PointDirection::UpRight,
            PointDirection::DownLeft => PointDirection::DownRight,
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
        let result = self.point.get_adjacent(&self.direction);
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
        let mut result = self.point.get_adjacent(&CARDINAL_DIRECTIONS[self.index]);
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
        let mut result = self.point.get_adjacent(&RADIAL_DIRECTIONS[self.index]);
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
    horizontal_direction: PointDirection,
    vertical_length: usize,
    vertical_direction: PointDirection,
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

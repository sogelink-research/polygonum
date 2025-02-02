/// Three dimensional point
#[derive(Clone, Copy, Debug)]
pub struct Point {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// Oriented segment connecting two [Point]s.
pub type Segment = (Point, Point);

impl PartialEq for Point {
    /// Equality between points is given by their coordinates
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y && self.z == other.z
    }
}

impl Eq for Point {}

impl Ord for Point {
    /// Coordinates wise ordering
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.x < other.x {
            std::cmp::Ordering::Less
        } else if self.x > other.x {
            std::cmp::Ordering::Greater
        } else if self.y < other.y {
            std::cmp::Ordering::Less
        } else if self.y > other.y {
            std::cmp::Ordering::Greater
        } else if self.z < other.z {
            std::cmp::Ordering::Less
        } else if self.z > other.z {
            std::cmp::Ordering::Greater
        } else {
            std::cmp::Ordering::Equal
        }
    }
}

impl PartialOrd for Point {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::hash::Hash for Point {
    /// Hashing is based on the coordinates' bits
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.x.to_bits().hash(state);
        self.y.to_bits().hash(state);
        self.z.to_bits().hash(state);
    }
}

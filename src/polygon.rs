use super::point::Point;

use hashbrown::HashSet;
use std::collections::BTreeSet;

/// A polygon is represented by an ordered set of vertices.
pub struct Polygon {
    /// Unique set of vertices belonging to the polygon.
    set: BTreeSet<Point>,
    /// Ordered sequences of vertices with positive normal where `sequence.first() == sequence.last()`.
    sequence: Vec<Point>,
    /// Precomputed bounding box around the polygon.
    boundary: (Point, Point),
}

impl Polygon {
    /// Constructs a polygon from an ordered path of unique vertices, last one not repeating the first.
    pub fn from(mut vertices: Vec<Point>) -> Self {
        // replicates the opening vertex as the closing one such that `sequence.first() == sequence.last()`
        if let Some(&root) = vertices.first() {
            vertices.push(root);
        }
        // flips the order of the vertices if the plane's normal is detected as negative when projected on the z-axis
        if super::plane::normal(&vertices).z < 0f64 {
            vertices.reverse();
        }
        // also constructs the bounding box of the polygon
        Self {
            boundary: Self::boundary(&vertices),
            set: vertices.iter().copied().collect(),
            sequence: vertices,
        }
    }

    /// Constructs the bounding box around the polygon.
    fn boundary(vertices: &[Point]) -> (Point, Point) {
        // minimum point according to the three dimensions
        let mut min = Point {
            x: f64::INFINITY,
            y: f64::INFINITY,
            z: f64::NAN,
        };
        // maximum point according to the three dimensions
        let mut max = Point {
            x: f64::NEG_INFINITY,
            y: f64::NEG_INFINITY,
            z: f64::NAN,
        };
        // computes minimum and maximum points
        for Point { x, y, .. } in vertices {
            if *x < min.x {
                min.x = *x;
            }

            if *x > max.x {
                max.x = *x;
            }

            if *y < min.y {
                min.y = *y;
            }

            if *y > max.y {
                max.y = *y;
            }
        }
        // bounding box
        (min, max)
    }

    /// Checks whether the polygon's bounding box fully contains the bounding box of `other`.
    fn contains_boundary_of(&self, other: &Self) -> bool {
        self.boundary.0.x <= other.boundary.0.x
            && self.boundary.1.x >= other.boundary.1.x
            && self.boundary.0.y <= other.boundary.0.y
            && self.boundary.1.y >= other.boundary.1.y
    }

    /// Checks whether the polygon contains `point` either within or on the edges.
    fn contains_point(&self, point: &Point) -> bool {
        // first check whether the point is one of the vertices
        if self.set.contains(point) {
            return true;
        }
        // otherwise it checks whether it is contained inside
        let n = self.sequence.len() - 1;
        let mut inside = false;
        // otherwise it applies the iterative procedure to verify if `point` is contained
        for i in 0..n {
            let a = self.sequence[i];
            let b = self.sequence[(i + 1) % n];

            if (a.y > point.y) != (b.y > point.y)
                && point.x < a.x + ((point.y - a.y) * (b.x - a.x) / (b.y - a.y))
            {
                inside = !inside;
            }
        }
        // this means fully inside the polygon's region
        inside
    }

    /// Checks whether the polygon shares sides with `other`.
    fn shares_sides_with(&self, other: &Self) -> bool {
        for i in 0..(self.sequence.len() - 1) {
            for j in 0..(other.sequence.len() - 1) {
                if (self.sequence[i], self.sequence[i + 1])
                    == (other.sequence[j], other.sequence[j + 1])
                    || (self.sequence[i], self.sequence[i + 1])
                        == (other.sequence[j + 1], other.sequence[j])
                {
                    return true;
                }
            }
        }

        false
    }

    /// Checks whether the polygon contains fully `other`.
    fn contains(&self, other: &Self) -> bool {
        self.contains_boundary_of(other)
            && other
                .sequence
                .iter()
                .all(|point| self.contains_point(point))
    }

    /// Assuming the polygon is quasi-bidimensional, computes the area on its plane.
    fn area(&self) -> f64 {
        super::plane::normal(&self.sequence).norm() / 2f64
    }

    /// Projects the polygon on the xy plane and computes its area (from above).
    fn area_projected(&self) -> f64 {
        super::plane::normal(&self.sequence).z.abs() / 2f64
    }

    /// Constructs an iterator to visit the vertices where the last equals the first.
    pub fn iter(&self) -> PolygonIterator {
        PolygonIterator {
            polygon: self,
            index: 0usize,
        }
    }
}

impl PartialEq for Polygon {
    /// Two polygons are equal if they have the same vertices
    fn eq(&self, other: &Self) -> bool {
        self.set.eq(&other.set)
    }
}

impl Eq for Polygon {}

impl std::hash::Hash for Polygon {
    /// Computes the hash of the polygon as the hash of its vertices.
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.set.iter().for_each(|point| point.hash(state));
    }
}

/// The polygon iterator iterates through its vertices.
pub struct PolygonIterator<'a> {
    /// Reference to the original polygon.
    polygon: &'a Polygon,
    /// Iterating index.
    index: usize,
}

impl Iterator for PolygonIterator<'_> {
    type Item = Point;
    /// Yields next vertex along the ordered sequence.
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.polygon.sequence.len() {
            self.index += 1;
            Some(self.polygon.sequence[self.index - 1])
        } else {
            None
        }
    }
}

/// Filters the set `polygons` by discarding those that contain other smaller polygons and share sides with them.
/// Also, the procedure discards those polygons whose [Polygon::area_projected] is less than `minimum_area_projected`.
///
/// Note that this is a greedy selection procedure that first discard polygons with very small projected area, then it
/// sorts the left ones by the "real" area, and finally, it iteratively picks those that do not contain the previously
/// selected polygons.
pub fn filter(
    polygons: Vec<Polygon>,
    minimum_area_projected: f64,
) -> impl Iterator<Item = Polygon> {
    // discards the polygons whose projected area on the xy plane is less than `minimum_area_projected`
    let mut polygons = polygons
        .into_iter()
        .filter(|polygon| polygon.area_projected() >= minimum_area_projected)
        .collect::<Vec<Polygon>>();
    // the mask contains the indices of the polygons that will be taken eventually
    let mut mask = HashSet::<usize>::new();
    // sorts the polygons by their area
    polygons.sort_by(|a, b| a.area().partial_cmp(&b.area()).unwrap());
    // iteratively picks the valid polygons
    'selection: for (i, polygon) in polygons.iter().enumerate() {
        // checks whether `polygon` contains any of the previously selected polygons
        for &j in &mask {
            // containing means either insides on sharing common sides
            if polygon.contains(&polygons[j]) && polygon.shares_sides_with(&polygons[j]) {
                continue 'selection;
            }
        }
        // when valid it saves the index in the selection mask
        mask.insert(i);
    }
    // applies the selection mask and yields the valid polygons
    polygons
        .into_iter()
        .enumerate()
        .filter(move |(index, _)| mask.contains(index))
        .map(|(_, polygon)| polygon)
}

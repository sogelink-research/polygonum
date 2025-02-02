use super::point::{Point, Segment};

/// A three dimensional vector.
#[derive(Clone, Copy, Debug)]
pub(super) struct Vector {
    pub(super) x: f64,
    pub(super) y: f64,
    pub(super) z: f64,
}

impl Vector {
    /// Constructs the zero vector.
    pub(super) fn zero() -> Self {
        Self {
            x: 0f64,
            y: 0f64,
            z: 0f64,
        }
    }

    /// Constructs a vector from [Point].
    pub(super) fn from(point: &Point) -> Self {
        Self {
            x: point.x,
            y: point.y,
            z: point.z,
        }
    }

    /// Constructs an oriented vector from [Segment].
    pub(super) fn between(segment: &Segment) -> Self {
        Self {
            x: segment.1.x - segment.0.x,
            y: segment.1.y - segment.0.y,
            z: segment.1.z - segment.0.z,
        }
    }

    /// Like [Self::between] but normalizes the resulting vector.
    pub(super) fn unit(segment: &Segment) -> Self {
        Self::between(segment).normalize()
    }

    /// Computes the euclidean norm of the vector.
    pub(super) fn norm(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    /// Normalizes the vector.
    pub(super) fn normalize(&self) -> Vector {
        // first computes its norm
        let norm = self.norm();
        // if the vector is zero it cannot be normalized at all
        if norm <= f64::EPSILON {
            Vector::zero()
        } else {
            Vector {
                x: self.x / norm,
                y: self.y / norm,
                z: self.z / norm,
            }
        }
    }

    // Computes the asymmetric cross product with `other`.
    pub(super) fn cross(&self, other: &Self) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    // Computes the symmetric scalar product with `other`.
    pub(super) fn dot(&self, other: &Self) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    // Adds `other` and returns a new vector.
    pub(super) fn add(&self, other: &Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }

    // Subtracts `other` and returns a new vector.
    pub(super) fn subtract(&self, other: &Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }

    // Rescales the magnitude by `factor` a new vector.
    pub(super) fn scale(&self, factor: f64) -> Self {
        Self {
            x: self.x * factor,
            y: self.y * factor,
            z: self.z * factor,
        }
    }

    // Computes the clockwise angle with `other` projected on the xy plane.
    pub(super) fn theta(&self, other: &Self) -> f64 {
        std::f64::consts::PI
            + (other.y * self.x - other.x * self.y).atan2(self.x * other.x + self.y * other.y)
    }
}

/// Computes the clockwise angle projected on the xy plane between two consecutive segments.
#[inline]
pub(super) fn theta(a: &Segment, b: &Segment) -> f64 {
    Vector::unit(a).theta(&Vector::unit(b))
}

/// Computes the coplanarity between four points as the volume of the described tetrahedron.
#[inline]
pub(super) fn coplanarity(a: Point, b: Point, c: Point, d: Point) -> f64 {
    Vector::between(&(a, b))
        .cross(&Vector::between(&(a, c)))
        .dot(&Vector::between(&(a, d)))
        .abs()
        / 6f64
}

/// Computes the normal vector of the plane described by a polygon enclosed by a set of `vertices`.
#[inline]
pub(super) fn normal(vertices: &[Point]) -> Vector {
    // computes the center of the polygon to reduce big coordinates values in the computation and stabilize it
    let offset = center(vertices);
    // ensures that the last vertices corresponds to the first
    debug_assert_eq!(vertices.first(), vertices.last());
    // computes the normal describing the polygon's plane
    (0..(vertices.len() - 1))
        .map(|index| {
            Vector::from(&vertices[index])
                .subtract(&offset)
                .cross(&Vector::from(&vertices[index + 1]).subtract(&offset))
        })
        .reduce(|accumulator, element| accumulator.add(&element))
        .unwrap()
}

/// Computes the unweighted center point of a polygon.
#[inline]
pub(super) fn center(vertices: &[Point]) -> Vector {
    // ensures that the last vertices corresponds to the first
    debug_assert_eq!(vertices.first(), vertices.last());
    // skips the first vertex because it is repeated in `vertices`
    vertices[1..]
        .iter()
        .map(Vector::from)
        .reduce(|accumulator, vertex| accumulator.add(&vertex))
        .map(|total| total.scale(1f64 / (vertices.len() - 1) as f64))
        .unwrap()
}

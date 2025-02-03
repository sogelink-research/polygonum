extern crate polygonum;

macro_rules! point {
    ($x:expr, $y:expr, $z:expr) => {
        polygonum::Point {
            x: $x,
            y: $y,
            z: $z,
        }
    };
}

macro_rules! segment {
    ($x1:expr, $y1:expr, $z1:expr => $x2:expr, $y2:expr, $z2:expr) => {
        (point!($x1, $y1, $z1), point!($x2, $y2, $z2))
    };
}

#[test]
fn extract_one_polygon() {
    assert_eq!(
        1,
        polygonum::polygonalize(
            &vec![
                segment!(0f64, 0f64, 0f64 => 0f64, 10f64, 0f64),
                segment!(0f64, 10f64, 0f64 => 10f64, 10f64, 5f64),
                segment!(10f64, 10f64, 5f64 => 10f64, 0f64, 5f64),
                segment!(10f64, 0f64, 5f64 => 0f64, 0f64, 0f64),
                segment!(10f64, 10f64, 5f64 => 20f64, 10f64, 0f64),
                segment!(20f64, 10f64, 0f64 => 20f64, 0f64, 0f64),
            ],
            true,
            0.01,
        )
        .len(),
        "This structure exactly contains one plane because one is incomplete."
    );
}

#[test]
fn extract_two_polygons() {
    assert_eq!(
        2,
        polygonum::polygonalize(
            &vec![
                segment!(0f64, 0f64, 0f64 => 0f64, 10f64, 0f64),
                segment!(0f64, 10f64, 0f64 => 10f64, 10f64, 5f64),
                segment!(10f64, 10f64, 5f64 => 10f64, 0f64, 5f64),
                segment!(10f64, 0f64, 5f64 => 0f64, 0f64, 0f64),
                segment!(10f64, 10f64, 5f64 => 20f64, 10f64, 0f64),
                segment!(20f64, 10f64, 0f64 => 20f64, 0f64, 0f64),
                segment!(20f64, 0f64, 0f64 => 10f64, 0f64, 5f64),
            ],
            true,
            0.01,
        )
        .len(),
        "This structure exactly contains two planes."
    );
}

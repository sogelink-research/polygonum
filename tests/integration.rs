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

macro_rules! dataset {
    ($name:expr) => {
        &io::parse(
            [env!("CARGO_MANIFEST_DIR"), "resources", "data", $name]
                .iter()
                .collect::<std::path::PathBuf>()
                .to_str()
                .unwrap(),
        )
    };
}

#[test]
fn one() {
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
fn two() {
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
        "This structure exactly contains two polygons."
    );
}

#[test]
fn house() {
    assert_eq!(
        18,
        polygonum::polygonalize(dataset!("house.geojson"), true, 0.01).len(),
        "This structure exactly contains 18 polygons."
    );
}

#[test]
fn compound() {
    assert_eq!(
        144,
        polygonum::polygonalize(dataset!("compound.geojson"), true, 0.01).len(),
        "This structure exactly contains 144 polygons."
    );
}

#[test]
fn church() {
    assert_eq!(
        126,
        polygonum::polygonalize(dataset!("church.geojson"), true, 0.01).len(),
        "This structure exactly contains 126 polygons."
    );
}

mod io {
    pub(super) fn parse(filename: &str) -> Vec<polygonum::Segment> {
        match std::fs::read_to_string(filename) {
            Ok(content) => serde_json::from_str::<serde_json::Value>(&content).unwrap()["features"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|&element| element["geometry"]["type"] == "LineString")
                .map(|element| {
                    let coordinates = element["geometry"]["coordinates"].as_array().unwrap();
                    let from = coordinates[0].as_array().unwrap();
                    let to = coordinates[1].as_array().unwrap();

                    segment!(
                        from[0].as_f64().unwrap(),
                        from[1].as_f64().unwrap(),
                        from[2].as_f64().unwrap()
                        =>
                        to[0].as_f64().unwrap(),
                        to[1].as_f64().unwrap(),
                        to[2].as_f64().unwrap()
                    )
                })
                .collect::<Vec<_>>(),
            Err(_) => panic!("unable to read data file to run test"),
        }
    }
}

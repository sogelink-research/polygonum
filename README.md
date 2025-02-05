# Polygonum

Polygonum is a Rust-powered crate to construct approximately-bidimensional polygons from a set of three dimensional lines.

## Installation

To add Polygonum to your Rust project's dependecies, just place yourself into your project's directory and type as follows.

```sh
cargo add --git "https://github.com/sogelink-research/polygonum.git"
```

Or alternatively add the following line to your `Cargo.toml` file.

```toml
[dependencies]
polygonum = { git = "https://github.com/sogelink-research/polygonum.git" }
```

## Usage

The following example illustartes how Polygonum digests a GeoJSON dataset and constructs the Polygon geometries from its LineString geometries. Finally, we display these in well-known text format.

```rust title="main.rs"
use polygonum;

fn main() {
    // read file of linestrings, aka our segments
    let segments = parse("data.geojson");
    // construct polygons using a parallelized pipeline and 0.01 as minimum polygon's area on the xy plane
    let polygons = polygonum::polygonalize(&segments, true, 0.01);
    // print polygons in well-known text format
    polygons.iter().for_each(|polygon| println!("{}", polygon.wkt()));
}

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

trait WKT {
    fn wkt(&self) -> String;
}

impl WKT for polygonum::Polygon {
    fn wkt(&self) -> String {
        format!(
            "POLYGON (({}))",
            self.iter()
                .map(|point| format!(
                    "{} {} {}",
                    point.x, point.y, point.z
                ))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

fn parse(filename: &str) -> Vec<polygonum::Segment> {
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
        Err(_) => panic!("unable to read data file"),
    }
}
```

To compile and run the previous code snippet, inside within the project directory, type as follows.

```sh
cargo run
```

## Design

The design of the whole pipeline used by Polygonum to construct a set of polygons from a set of segments is illustrated down below.

![pipeline](resources/images/polygonum-pipeline.png)

## Dependencies

We use the following dependencies in our implementation.

- [Rayon](https://github.com/rayon-rs/rayon) is leveraged to parallelize the pipeline due to the graph partitioning into independent subgraphs.
- [Hashbrown](https://github.com/rust-lang/hashbrown) is employed to achieve faster hashing when using hash maps or sets.

---

Authored by [Emanuel Buttaci](https://github.com/buttaciemanuel), reach me at emanuel.buttaci@sogelink.com.

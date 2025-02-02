CREATE OR REPLACE FUNCTION plrust.polygonalize(inputs TEXT[])
    RETURNS SETOF TEXT
    LANGUAGE plrust STRICT
AS $$
[dependencies]
    rayon = "1.10.0"
    polygonum = { git = "https://github.com/sogelink-research/polygonum.git" }
[code]
    // once the table is created, call the routine as
    // select * from plrust.polygonalize((select array_agg(linestring) from lines));
    use polygonum::*;
    // linestring to pair of points
    fn from_wkt(line: &str) -> Segment {
        let begin = line.find('(').unwrap();
        let end = line.find(')').unwrap();
        let comma = line.find(',').unwrap();
        let a = &line[(begin + 1)..comma]
            .trim()
            .split(" ")
            .collect::<Vec<&str>>();
        let b = &line[(comma + 1)..end]
            .trim()
            .split(" ")
            .collect::<Vec<&str>>();

        (
            Point {
                x: a[0].parse().unwrap(),
                y: a[1].parse().unwrap(),
                z: a[2].parse().unwrap(),
            },
            Point {
                x: b[0].parse().unwrap(),
                y: b[1].parse().unwrap(),
                z: b[2].parse().unwrap(),
            },
        )
    }
    // well-known text conversion
    trait WellKnownTextConversion {
        fn wkt(&self) -> String;
    }
    // implement well-known text conversion
    impl WellKnownTextConversion for Polygon {
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
    // construct segments
    let segments = inputs
        .iter()
        .map(|linestring| from_wkt(linestring.unwrap()))
        .collect::<Vec<Segment>>();
    // constructs all polygons
    let polygons = polygonalize(&segments, true, 0.01);
    // in well-known text format
    Ok(Some(SetOfIterator::new(
        polygons.into_iter().map(|polygon| Some(polygon.wkt())),
    )))
$$;
pub mod graph;
pub mod pipeline;
pub mod plane;
pub mod point;
pub mod polygon;

pub use graph::*;
pub use pipeline::*;
pub use point::*;
pub use polygon::*;

/// Constructs a set of polygons from a set of [point::Segment]s.
///
/// Filtering polygons is possible through `minimum_area_projected` and also
/// parallel processing can be enabled through `parallelize`.
pub fn polygonalize(
    segments: &[point::Segment],
    parallelize: bool,
    minimum_area_projected: f64,
) -> Vec<polygon::Polygon> {
    if parallelize {
        // parallel processing pipeline
        pipeline::Pipeline::from(segments)
            .partition()
            .apply(|subgraph| {
                // constructs the polygons from each subgraph and filters them
                polygon::filter(
                    subgraph.segment().into_iter().collect(),
                    minimum_area_projected,
                )
            })
    } else {
        // sequential processing
        pipeline::Pipeline::from(segments).apply(|graph| {
            // constructs the polygons from the graph and filters them
            polygon::filter(
                graph.segment().into_iter().collect(),
                minimum_area_projected,
            )
        })
    }
}

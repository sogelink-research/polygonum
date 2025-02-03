use super::{
    graph::{PointGraph, SegmentGraph},
    point::{Point, Segment},
};

use hashbrown::HashSet;
use rayon::prelude::*;

/// A pipeline processes a list of segments and delivers a set of polygons.
pub struct Pipeline {
    /// The adjacency list that represents the graph of points.
    graph: PointGraph,
}

impl Pipeline {
    /// Instantiate the pipeline from a set of segments.
    pub fn from(segments: &[Segment]) -> Self {
        Self {
            // prune the graph by removing dead ends
            graph: PointGraph::from(segments).prune(),
        }
    }

    /// Takes ownership of the pipeline to construct a pipeline doing parallel processesing on the graph's
    /// connected components.
    pub fn partition(self) -> PartitionPipeline {
        PartitionPipeline { graph: self.graph }
    }

    /// Applies a transformation function to the constructed [SegmentGraph] and collects the outputs as a vector.
    ///
    /// Note that this performs sequential processing and might be slow for large graphs where [PartitionPipeline]
    /// is suggested.
    pub fn apply<F, I, R>(&self, transform: F) -> Vec<R>
    where
        I: Iterator<Item = R>,
        F: Fn(SegmentGraph) -> I + Send + Sync,
        R: Send + Sync,
    {
        // constructs the full graph of segments
        transform(SegmentGraph::from(&self.graph.fullgraph())).collect::<Vec<R>>()
    }
}

/// This pipeline is constructed from [Pipeline] to parallelize processing across disconnected [SegmentGraph]s.
pub struct PartitionPipeline {
    /// The adjacency list that represents the graph of points.
    graph: PointGraph,
}

impl PartitionPipeline {
    /// Applies `transform` independently on each disconnected [SegmentGraph] and collects all results as flattened list.
    ///
    /// This performs better than [Pipeline::apply] because it leverages parallel processing on each connected component.
    pub fn apply<F, I, R>(&self, transform: F) -> Vec<R>
    where
        I: Iterator<Item = R>,
        F: Fn(SegmentGraph) -> I + Send + Sync,
        R: Send + Sync,
    {
        // explored vertices when identifying connected components
        let mut explored = HashSet::<Point>::new();
        // first instantiate each graph as an independent connected component and performs parallel processing
        self.graph
            .adjacencies
            .keys()
            .filter_map(|point| {
                // constructs each connected component from the graph of points first
                if !explored.contains(point) {
                    // if the point has not been visited yet it will detect its associated connected component
                    let mut points = HashSet::<Point>::new();
                    // recursive exploration as depth first traversal
                    self.explore(point, &mut explored, &mut points);
                    // returns the list of points as a connected component
                    Some(points)
                } else {
                    None
                }
            })
            .par_bridge()
            .flat_map_iter(|points| {
                // this will run in parallel for each connected component given by an independent graph of points
                // so we construct the associated graph of segments with the connected component `points` and
                // we apply `transform` and collect all its results
                transform(SegmentGraph::from(&self.graph.subgraph(points)))
            })
            .collect::<Vec<R>>()
    }

    /// Performs a depth first search from node `point` to detect all points in connected component `partition`.
    fn explore(
        &self,
        point: &Point,
        explored: &mut HashSet<Point>,
        partition: &mut HashSet<Point>,
    ) {
        // visit only if not visited already
        if !explored.contains(point) {
            // point is added to the connected component
            explored.insert(*point);
            partition.insert(*point);
            // recursive traversal is applied to each of its neighboring points
            self.graph.adjacencies[point].iter().for_each(|neighbor| {
                self.explore(neighbor, explored, partition);
            });
        }
    }
}

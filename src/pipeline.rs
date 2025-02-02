use super::{
    graph::SegmentGraph,
    point::{Point, Segment},
};

use rayon::prelude::*;
use std::collections::{HashMap, HashSet};

/// A pipeline processes a list of segments and delivers a set of polygons.
pub struct Pipeline {
    /// The adjacency list that represents the graph of points.
    adjacencies: HashMap<Point, HashSet<Point>>,
}

impl Pipeline {
    /// Instantiate the pipeline from a set of segments.
    pub fn from(segments: &[Segment]) -> Self {
        Self {
            // prune the graph by removing dead ends
            adjacencies: Self::prune_adjacency_list_of_points(
                // constructs the full graph of points as its adjacency list
                Self::construct_adjacency_list_of_points(segments),
            ),
        }
    }

    /// Takes ownership of the pipeline to construct a pipeline doing parallel processesing on the graph's
    /// connected components.
    pub fn partition(self) -> PartitionPipeline {
        PartitionPipeline {
            adjacencies: self.adjacencies,
        }
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
        transform(Self::build_segment_graph_from_adjacency_list_of_points(
            None,
            &self.adjacencies,
        ))
        .collect::<Vec<R>>()
    }

    /// Given a list of segments, it constructs the graph of all detected and connected points.
    fn construct_adjacency_list_of_points(segments: &[Segment]) -> HashMap<Point, HashSet<Point>> {
        // empty adjacency list of points
        let mut adjacencies = HashMap::<Point, HashSet<Point>>::new();
        // iterates over every segment
        segments.iter().for_each(|&(u, v)| {
            // adds the segment to the graph as an edge between the two points
            adjacencies
                .entry(u)
                .and_modify(|to| {
                    to.insert(v);
                })
                .or_insert(HashSet::from([v]));
            // does the same for its flipped counterpart
            adjacencies
                .entry(v)
                .and_modify(|to| {
                    to.insert(u);
                })
                .or_insert(HashSet::from([u]));
        });
        // yields the constructed graph of points
        adjacencies
    }

    /// Prunes the graph of points in-place by removing dead ends and related points and interconnections.
    fn prune_adjacency_list_of_points(
        mut adjacencies: HashMap<Point, HashSet<Point>>,
    ) -> HashMap<Point, HashSet<Point>> {
        // detects the points which are dead ends and have degree equals to 1
        let mut leaves = adjacencies
            .iter()
            .filter(|(_, to)| to.len() == 1)
            .map(|(&leaf, _)| leaf)
            .collect::<HashSet<_>>();
        // iteratively prunes the leaves until no dead ends are left
        while !leaves.is_empty() {
            // next round leaves
            let mut updated = HashSet::<Point>::new();
            // iteratively prunes each leaf
            for leaf in &leaves {
                // prune only if it was not pruned already
                if adjacencies.contains_key(leaf) {
                    // prunes the leaf from each of its connected neighboring points
                    if let Some(&adjacent) = adjacencies[leaf].iter().next() {
                        // the neighbor will be a new leaf if it was poorly connected
                        if adjacencies[&adjacent].len() <= 2 {
                            updated.insert(adjacent);
                        }
                        // removes the leaf from its neighbors' adjacencies
                        adjacencies.entry(adjacent).and_modify(|to| {
                            to.remove(leaf);
                        });
                    }
                    // definitely removes the leaf
                    adjacencies.remove(leaf);
                }
            }
            // new leaves consequently resulting as a smaller subset of previous leaves
            leaves = updated;
        }
        // pruned adjacency list of points
        adjacencies
    }

    /// Constructs the [SegmentGraph] from a list of source `points` and their `adjacencies`.
    fn build_segment_graph_from_adjacency_list_of_points(
        points: Option<&HashSet<Point>>,
        adjacencies: &HashMap<Point, HashSet<Point>>,
    ) -> SegmentGraph {
        // the finally delivered adjacency list of segments
        let mut graph = HashMap::<Segment, HashSet<Segment>>::new();
        // for each considered `point` in `points`, it connects its ingoing segments to its outgoing segments
        adjacencies
            .iter()
            .filter(|(point, _)| points.map_or(true, |values| values.contains(point)))
            .for_each(|(&point, neighbors)| {
                // using the `neighbors` of `point`, it links ingoing to outgoing segments
                neighbors
                    .iter()
                    .flat_map(|x| std::iter::repeat(x).zip(neighbors))
                    .for_each(|(&from, &to)| {
                        // obviously avoids creating unwanted cycles
                        if from != to {
                            graph
                                .entry((from, point))
                                .and_modify(|segments| {
                                    segments.insert((point, to));
                                })
                                .or_insert(HashSet::from([(point, to)]));
                        }
                    });
            });
        // instantiate the segment graph from its adjacency list
        SegmentGraph { adjacencies: graph }
    }
}

/// This pipeline is constructed from [Pipeline] to parallelize processing across disconnected [SegmentGraph]s.
pub struct PartitionPipeline {
    /// The adjacency list that represents the graph of points.
    adjacencies: HashMap<Point, HashSet<Point>>,
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
        self.adjacencies
            .keys()
            .filter_map(|point| {
                // constructs each connected component from the graph of points first
                if !explored.contains(point) {
                    // if the point has not been visited yet it will detect its associated connected component
                    let mut partition = HashSet::<Point>::new();
                    // recursive exploration as depth first traversal
                    self.explore(point, &mut explored, &mut partition);
                    // returns the list of points as a connected component
                    Some(partition)
                } else {
                    None
                }
            })
            .par_bridge()
            .flat_map_iter(|partition| {
                // this will run in parallel for each connected component given by an independent graph of points
                // so we construct the associated graph of segments with the connected component `partition` and
                // we apply `transform` and collect all its results
                transform(Pipeline::build_segment_graph_from_adjacency_list_of_points(
                    Some(&partition),
                    &self.adjacencies,
                ))
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
            self.adjacencies[point].iter().for_each(|neighbor| {
                self.explore(neighbor, explored, partition);
            });
        }
    }
}

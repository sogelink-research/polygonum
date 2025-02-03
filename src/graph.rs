use super::point::{Point, Segment};

use hashbrown::{HashMap, HashSet};
use std::collections::{BTreeMap, BTreeSet};

pub(super) struct PointGraph {
    /// The adjacency list that represents the graph of points.
    pub(super) adjacencies: HashMap<Point, HashSet<Point>>,
}

pub(super) struct PointSubGraph<'a> {
    /// Reference to the main graph
    pub(super) graph: &'a PointGraph,
    pub(super) points: Option<HashSet<Point>>,
}

impl PointGraph {
    /// Given a list of segments, it constructs the graph of all detected and connected points.
    pub(super) fn from(segments: &[Segment]) -> Self {
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
        Self { adjacencies }
    }

    /// Prunes the graph of points in-place by removing dead ends and related points and interconnections.
    pub(super) fn prune(mut self) -> Self {
        // detects the points which are dead ends and have degree equals to 1
        let mut leaves = self
            .adjacencies
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
                if self.adjacencies.contains_key(leaf) {
                    // prunes the leaf from each of its connected neighboring points
                    if let Some(&adjacent) = self.adjacencies[leaf].iter().next() {
                        // the neighbor will be a new leaf if it was poorly connected
                        if self.adjacencies[&adjacent].len() <= 2 {
                            updated.insert(adjacent);
                        }
                        // removes the leaf from its neighbors' adjacencies
                        self.adjacencies.entry(adjacent).and_modify(|to| {
                            to.remove(leaf);
                        });
                    }
                    // definitely removes the leaf
                    self.adjacencies.remove(leaf);
                }
            }
            // new leaves consequently resulting as a smaller subset of previous leaves
            leaves = updated;
        }
        // pruned adjacency list of points
        self
    }

    /// Constructs a slice of the graph based on a set of its points.
    pub(super) fn subgraph(&self, points: HashSet<Point>) -> PointSubGraph {
        PointSubGraph {
            graph: self,
            points: Some(points),
        }
    }

    /// Constructs a slice of the graph with all points.
    pub(super) fn fullgraph(&self) -> PointSubGraph {
        PointSubGraph {
            graph: self,
            points: None,
        }
    }
}

/// This graph contains the edges between points as oriented segments.
pub struct SegmentGraph {
    /// The adjacency list representation of the graph.
    pub(super) adjacencies: HashMap<Segment, HashSet<Segment>>,
}

impl SegmentGraph {
    /// Constructs the graph from a list of source `points` and their `adjacencies`.
    pub(super) fn from(subgraph: &PointSubGraph) -> SegmentGraph {
        // the finally delivered adjacency list of segments
        let mut graph = HashMap::<Segment, HashSet<Segment>>::new();
        // for each considered `point` in `points`, it connects its ingoing segments to its outgoing segments
        subgraph
            .graph
            .adjacencies
            .iter()
            .filter(|(&point, _)| {
                subgraph
                    .points
                    .as_ref()
                    .map_or(true, |values| values.contains(&point))
            })
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

impl std::hash::Hash for SegmentGraph {
    /// The hash is computed as the overall hash of the adjacency list representation of the graph.
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.adjacencies
            .iter()
            .map(|(&segment, successor)| (segment, successor.iter().collect::<BTreeSet<_>>()))
            .collect::<BTreeMap<_, _>>()
            .hash(state);
    }
}

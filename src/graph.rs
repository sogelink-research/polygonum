use super::{
    point::{Point, Segment},
    polygon::Polygon,
};

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

/// This graph contains the edges between points as oriented segments.
pub struct SegmentGraph {
    /// The adjacency list representation of the graph.
    pub(super) adjacencies: HashMap<Segment, HashSet<Segment>>,
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

/// The result of the recursive graph traversal when constructing its faces, namely polygons.
enum TraversalResult {
    /// When backtracking to previous recursion level because the current segment has already been explored.
    Backtracking,
    /// When exhausting a specific recursion level on a segment by fully visiting it.
    Exploring,
    /// When a new path has been closed and a new polygon has been construced.
    PathClosing,
}

impl SegmentGraph {
    /// Constructs a set of unique polygons from the graph by performing a composite graph traversal.
    ///
    /// The inexact procedure is pretty efficient because it does not instantiate a branching recursion tree.
    /// This means that the complexity is `O(E * k)` where `E` is the total number of connections between all
    /// segments and `k` is the average polygon's size. This ensures that the complexity is always polynomial
    /// and NEVER degenerates to exponential by design. Moreover, two different criteria are employed to chose
    /// on which segment to recur when following a path. First, we pick the next segment minimizing the pair
    /// `(theta, coplanarity)` where `theta` is the clockwise angle between the current segment and the next
    /// candidate projected on the xy plane whereas coplanarity is the area of the tetrahedron considering the
    /// four points belonging to the previous segment, the current one and the next candidate. Second, we repeat
    /// the recursive traversal by constructing other polygons using as criterion the minimization of the opposite
    /// pair, that is `(coplanarity, theta)`. This helps identifies polygons that vertically overlap but are distinct.
    pub fn segment(&self) -> HashSet<Polygon> {
        // recursion stack to keep track of the visited segments
        let mut stack = Vec::<Segment>::new();
        // to keep track of visited the vertices and rapidly lookup their level in the recursion stack
        let mut depth = HashMap::<Segment, usize>::new();
        // saves polygons as closed paths
        let mut paths = HashSet::<Polygon>::new();
        // iteratively begins from every segment as uses the first criterion in the recursive traversal
        for (source, successors) in &self.adjacencies {
            // the source is put at the base of the recursion stack
            depth.insert(*source, 0);
            stack.push(*source);
            // naively tries every successor to have a `previous` segment in further recursive calls
            for successor in successors {
                // recursive traversal
                self.traverse(
                    successor,
                    source,
                    |previous, current, next| {
                        (
                            super::plane::theta(current, next),
                            super::plane::coplanarity(previous.0, current.0, current.1, next.1),
                        )
                    },
                    &mut stack,
                    &mut depth,
                    &mut paths,
                );
            }
            // at debug time verifies that the source is still at the root of the recursion stack
            debug_assert_eq!(stack.len(), 1);
            debug_assert_eq!(depth.len(), 1);
            // iteratively begins from every segment as uses the second criterion in the recursive traversal
            for successor in successors {
                // new traversal to detect overlaying polygons
                self.traverse(
                    successor,
                    source,
                    |previous, current, next| {
                        (
                            super::plane::coplanarity(previous.0, current.0, current.1, next.1),
                            super::plane::theta(current, next),
                        )
                    },
                    &mut stack,
                    &mut depth,
                    &mut paths,
                );
            }
            // removes the source from the root of the stack
            if let Some(segment) = stack.pop() {
                depth.remove(&segment);
            }
            // ensures that the recursion stack is empty
            debug_assert_eq!(stack.len(), 0);
            debug_assert_eq!(depth.len(), 0);
        }
        // yields found polygons
        paths
    }

    /// Recursive traversal of `current` segment from `previous` where the minimization of `criterion(previous, current, candidate)`
    /// is employed to choose which candidate will be next in the recursion traversal.
    fn traverse<F, T>(
        &self,
        current: &Segment,
        previous: &Segment,
        criterion: F,
        stack: &mut Vec<Segment>,
        depth: &mut HashMap<Segment, usize>,
        paths: &mut HashSet<Polygon>,
    ) -> TraversalResult
    where
        F: Fn(&Segment, &Segment, &Segment) -> T,
        T: PartialOrd,
    {
        if depth.contains_key(&(current.1, current.0)) {
            // we are traversing an already explored segment by walking on it in the opposite sense thus we must backtrack
            TraversalResult::Backtracking
        } else if let Some(&position) = depth.get(current) {
            // we are visiting an already visited segment, this means we are closing a path
            paths.insert(Polygon::from(
                stack[position..]
                    .iter()
                    .map(|segment| segment.0)
                    .collect::<Vec<Point>>(),
            ));
            // we save the detected polygon and we go back one level
            TraversalResult::PathClosing
        } else {
            // otherwise we explore the new segment by pushing it onto the stack
            if let Some(last) = stack.last() {
                depth.insert(*current, depth[last] + 1);
                stack.push(*current);
            }
            // chooses the next segment that minimizes the criterion
            if let Some(successor) = self.adjacencies[current]
                .iter()
                .map(|segment| (*segment, criterion(previous, current, segment)))
                .min_by(|(_, alpha), (_, beta)| alpha.partial_cmp(beta).unwrap())
                .map(|(successor, _)| successor)
            {
                // and recursively traverses it
                self.traverse(&successor, current, criterion, stack, depth, paths);
            }
            // removes `segment` which corresponds to `current` from the recursion stack
            if let Some(segment) = stack.pop() {
                depth.remove(&segment);
            }
            // `current` has been exhaustively explored and we can go back one level
            TraversalResult::Exploring
        }
    }
}

use super::{
    graph::SegmentGraph,
    point::{Point, Segment},
    polygon::Polygon,
};

use hashbrown::{HashMap, HashSet};

/// The result of the recursive graph traversal when constructing its faces, namely polygons.
enum Status {
    /// When backtracking to previous recursion level because the current segment has already been explored.
    Backtracking,
    /// When exhausting a specific recursion level on a segment by fully visiting it.
    Exploring,
    /// When a new path has been closed and a new polygon has been construced.
    PathClosing,
}

/// Strategy algorithm to elect optimal segment as successor when recursively traversing the graph.
trait ElectionStrategy {
    /// Elects optimal segment as successor when recursively traversing the graph.
    fn elect(&mut self, previous: Segment, current: Segment) -> Option<Segment>;
}

/// This election strategy runs in `O(m)` where `m` is the number of adjacencies of the each segment
/// using the policy function and the referenced graph.
struct GreedyElectionStrategy<'a, T>
where
    T: PartialOrd,
{
    cache: HashMap<(Segment, Segment), Option<Segment>>,
    graph: &'a SegmentGraph,
    policy: fn(Segment, Segment, Segment) -> T,
}

impl<'a, T> GreedyElectionStrategy<'a, T>
where
    T: PartialOrd,
{
    /// Constructs a greedy election strategy using a specific policy and referencing the given graph.
    fn from(graph: &'a SegmentGraph, policy: fn(Segment, Segment, Segment) -> T) -> Self {
        Self {
            cache: HashMap::new(),
            graph,
            policy,
        }
    }
}

impl<T> ElectionStrategy for GreedyElectionStrategy<'_, T>
where
    T: PartialOrd,
{
    /// Elects optimal segment as successor when recursively traversing the graph using the policy [CachingGreedyElectionStrategy::policy].
    fn elect(&mut self, previous: Segment, current: Segment) -> Option<Segment> {
        // gets the optiomal successor if cached otherwise computes it with the policy function
        *self.cache.entry((previous, current)).or_insert_with(|| {
            // leverages the ordering of the policy result to choose the best
            self.graph.adjacencies[&current]
                .iter()
                .map(|&segment| (segment, (self.policy)(previous, current, segment)))
                .min_by(|(_, alpha), (_, beta)| alpha.partial_cmp(beta).unwrap())
                .map(|(successor, _)| successor)
        })
    }
}

/// A traversal instance recursively visits a graph and extracts its polygons according to specific policies.
struct Traversal<'a> {
    graph: &'a SegmentGraph,
    stack: Vec<Segment>,
    depth: HashMap<Segment, usize>,
    paths: HashSet<Polygon>,
}

impl<'a> Traversal<'a> {
    /// Instantiates a traversal from a [SegmentGraph] to construct polygons.
    pub fn from(graph: &'a SegmentGraph) -> Self {
        Self {
            graph,
            stack: Vec::new(),
            depth: HashMap::new(),
            paths: HashSet::new(),
        }
    }

    /// Constructs a set of unique polygons from the graph by performing a policy-guided graph traversal.
    ///
    /// The inexact procedure is pretty efficient because it does not instantiate a branching recursion tree.
    /// This means that the complexity is `O(E * k)` where `E` is the total number of connections between all
    /// segments and `k` is the average polygon's size. This ensures that the complexity is always polynomial
    /// and NEVER degenerates to exponential by design.
    pub fn run(mut self, strategies: &mut [impl ElectionStrategy]) -> Vec<Polygon> {
        // traverses the whole graph using all strategies
        self.graph
            .adjacencies
            .iter()
            .for_each(|(source, successors)| {
                // the source is put at the base of the recursion stack
                self.depth.insert(*source, 0);
                self.stack.push(*source);
                // naively tries every successor to have a `previous` segment in further recursive calls
                successors.iter().for_each(|successor| {
                    // applies every traversal strategy
                    strategies.iter_mut().for_each(|strategy| {
                        // recursive traversal from `successor` on
                        self.traverse(successor, source, strategy).ok();
                        // at debug time verifies that the source is still at the root of the recursion stack
                        debug_assert_eq!(self.stack.len(), 1);
                        debug_assert_eq!(self.depth.len(), 1);
                    });
                });
                // removes the source from the root of the stack
                if let Some(segment) = self.stack.pop() {
                    self.depth.remove(&segment);
                }
                // ensures that the recursion stack is empty
                debug_assert_eq!(self.stack.len(), 0);
                debug_assert_eq!(self.depth.len(), 0);
            });
        // yields found polygons
        self.paths.into_iter().collect()
    }

    /// Recursive traversal of `current` segment from `previous` where the minimization of `criterion(previous, current, candidate)`
    /// is employed to choose which candidate will be next in the recursion traversal.
    fn traverse(
        &mut self,
        current: &Segment,
        previous: &Segment,
        strategy: &mut impl ElectionStrategy,
    ) -> Result<Status, ()> {
        if self.depth.contains_key(&(current.1, current.0)) {
            // we are traversing an already explored segment by walking on it in the opposite sense thus we must backtrack
            Ok(Status::Backtracking)
        } else if let Some(&position) = self.depth.get(current) {
            // we are visiting an already visited segment, this means we are closing a path
            self.paths.insert(Polygon::from(
                self.stack[position..]
                    .iter()
                    .map(|segment| segment.0)
                    .collect::<Vec<Point>>(),
            ));
            // we save the detected polygon and we go back one level
            Ok(Status::PathClosing)
        } else {
            // otherwise we explore the new segment by pushing it onto the stack
            if let Some(last) = self.stack.last() {
                self.depth.insert(*current, self.depth[last] + 1);
                self.stack.push(*current);
            }
            // chooses the next segment that minimizes the criterion
            if let Some(successor) = strategy.elect(*previous, *current) {
                // and recursively traverses it
                self.traverse(&successor, current, strategy).ok();
            }
            // removes `segment` which corresponds to `current` from the recursion stack
            if let Some(segment) = self.stack.pop() {
                self.depth.remove(&segment);
            }
            // `current` has been exhaustively explored and we can go back one level
            Ok(Status::Exploring)
        }
    }
}

/// Applies two distinct policies based on clockwise angle between segments and coplanarity to extract polygons.
///
/// Two different criteria are employed to chose on which segment to recur when following a path. First, we pick
/// the next segment minimizing the pair `(theta, coplanarity)` where `theta` is the clockwise angle between the
/// current segment and the next candidate projected on the xy plane whereas coplanarity is the area of the tetrahedron
/// considering the four points belonging to the previous segment, the current one and the next candidate. Second, we
/// repeat the recursive traversal by constructing other polygons using as criterion the minimization of the opposite
/// pair, that is `(coplanarity, theta)`. This helps identifies polygons that vertically overlap but are distinct.
#[inline]
pub(super) fn traverse(graph: &SegmentGraph) -> Vec<Polygon> {
    // by default we traverse using two strategies to detect polygons
    Traversal::from(graph).run(&mut [
        // first strategy to elect successor segment prioritizes the clockwise angle projected on the xy plane
        GreedyElectionStrategy::from(graph, |previous, current, next| {
            (
                super::plane::theta(&current, &next),
                super::plane::coplanarity(previous.0, current.0, current.1, next.1),
            )
        }),
        // second strategy to elect successor segment prioritizes the coplanarity
        GreedyElectionStrategy::from(graph, |previous, current, next| {
            (
                super::plane::coplanarity(previous.0, current.0, current.1, next.1),
                super::plane::theta(&current, &next),
            )
        }),
    ])
}

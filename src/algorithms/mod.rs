pub use bellman_ford::*;
pub use branch_bound::*;
pub use brute_force::*;
pub use dijkstra::*;
pub use double_tree::*;
pub use edmonds_karp::*;
pub use kruskal::*;
pub use nearest_neighbor::*;
pub use prim::*;
pub use search::*;

use crate::prelude::{AdjacencyList, EdgeIndex, Get, NodeIndex};
use thiserror::Error;

mod bellman_ford;
mod branch_bound;
mod brute_force;
mod dijkstra;
mod double_tree;
mod edmonds_karp;
mod kruskal;
mod nearest_neighbor;
mod prim;
mod search;

#[derive(Debug, Error, PartialEq, Eq, Clone, Copy)]
#[error("Negative Cycle detected")]
pub struct NegativeCycle;

pub struct ConnectedComponents {
    components: Vec<Vec<NodeIndex>>,
}

impl ConnectedComponents {
    pub fn new(components: Vec<Vec<NodeIndex>>) -> Self {
        Self { components }
    }

    pub fn count(&self) -> usize {
        self.components.len()
    }
}

#[derive(Debug)]
pub struct Tour<W> {
    pub route: Vec<NodeIndex>,
    pub weight: W,
}

impl<W> Tour<W> {
    pub fn new(route: Vec<NodeIndex>, weight: W) -> Self {
        Self { route, weight }
    }

    pub fn edges(&self) -> impl Iterator<Item = (&NodeIndex, &NodeIndex)> {
        self.route.array_windows::<2>().map(|[from, to]| (from, to))
    }

    pub fn nodes<'a, N, G>(&'a self, graph: &'a G) -> impl Iterator<Item = &'a N> + 'a
    where
        G: Get<N, W>,
    {
        self.route.iter().map(|index| graph.node(*index))
    }

    pub fn map<F, T>(self, mut f: F) -> Tour<T>
    where
        F: FnMut(W) -> T,
    {
        let Tour { route, weight } = self;
        let weight = f(weight);
        Tour { route, weight }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Distances<W> {
    pub distances: Vec<Option<W>>,
    pub from: NodeIndex,
}

impl<W> Distances<W> {
    pub fn new(from: NodeIndex, distances: Vec<Option<W>>) -> Self {
        Self { distances, from }
    }

    pub fn to(&self, to: NodeIndex) -> Option<&W> {
        self.distances[to.0].as_ref()
    }
}

#[derive(Debug)]
pub struct MinimumSpanningTree<N, W> {
    pub tree: AdjacencyList<N, W>,
    pub root: NodeIndex,
}

impl<N, W> MinimumSpanningTree<N, W> {
    pub fn new(tree: AdjacencyList<N, W>, root: NodeIndex) -> Self {
        Self { tree, root }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Flow<W> {
    pub current: W,
    pub max: W,
}

impl<W: Default> Flow<W> {
    pub fn new(max: W) -> Self {
        Self {
            max,
            current: W::default(),
        }
    }
}

pub struct ParentPath {
    pub(crate) parent: Vec<Option<NodeIndex>>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct AugmentedPath {
    pub(crate) edges: Vec<EdgeIndex>,
}

impl AugmentedPath {
    pub(crate) fn new(edges: Vec<EdgeIndex>) -> Self {
        Self { edges }
    }
}

#[derive(Debug)]
pub struct UnionFind {
    parent: Vec<NodeIndex>,
    rank: Vec<usize>,
    path: Vec<NodeIndex>,
}

impl UnionFind {
    pub fn root(&self) -> NodeIndex {
        self.parent[0]
    }

    pub fn rank(&self, index: NodeIndex) -> usize {
        self.rank[index.0]
    }

    pub fn find(&mut self, needle: NodeIndex) -> NodeIndex {
        let mut root = needle;

        self.path.clear();

        while self.parent[root.0] != root {
            self.path.push(root);
            root = self.parent[root.0];
        }

        // set root of every cached index in path to "root"
        // when union find is run for a longer time the
        // performance might degrade as find must traverse
        // more parents in the former loop
        // this allows to skip intermediate nodes and improves the performance
        for index in &self.path {
            self.parent[index.0] = root;
        }
        root
    }

    pub fn union(&mut self, x: NodeIndex, y: NodeIndex) {
        let mut root_x = self.find(x);
        let mut root_y = self.find(y);
        if root_x == root_y {
            return;
        }

        // keep depth of trees small by appending small tree to big tree
        // ensures find operation is not doing effectively a linked list search
        if self.rank[root_x.0] < self.rank[root_y.0] {
            std::mem::swap(&mut root_x, &mut root_y);
        }
        self.parent[root_y.0] = root_x;
        self.rank[root_x.0] += self.rank[root_y.0];
    }
}

// Set every parent of each tree to itself
// Meaning that every tree == 1 node
impl<T: Iterator<Item = NodeIndex>> From<T> for UnionFind {
    fn from(nodes: T) -> Self {
        let parent: Vec<NodeIndex> = nodes.collect();
        //parent.sort();

        let rank = vec![1; parent.len()];

        Self {
            parent,
            rank,
            path: Vec::new(),
        }
    }
}

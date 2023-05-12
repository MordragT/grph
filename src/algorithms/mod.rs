pub use branch_bound::*;
pub use brute_force::*;
pub use dijkstra::*;
pub use double_tree::*;
pub use kruskal::*;
pub use nearest_neighbor::*;
pub use prim::*;
pub use search::*;

use crate::prelude::{AdjacencyList, GraphAccess, NodeIndex};

mod branch_bound;
mod brute_force;
mod dijkstra;
mod double_tree;
mod kruskal;
mod nearest_neighbor;
mod prim;
mod search;

#[derive(Debug)]
pub struct Tour<W> {
    pub route: Vec<NodeIndex>,
    pub weight: W,
}

impl<W> Tour<W> {
    pub fn new(route: Vec<NodeIndex>, weight: W) -> Self {
        Self { route, weight }
    }

    pub fn nodes<'a, N, G>(&'a self, graph: &'a G) -> impl Iterator<Item = &'a N> + 'a
    where
        G: GraphAccess<N, W>,
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

#[derive(Debug)]
pub struct Distances<W> {
    distances: Vec<Option<W>>,
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
    pub graph: AdjacencyList<N, W>,
    pub root: NodeIndex,
}

impl<N, W> MinimumSpanningTree<N, W> {
    pub fn new(graph: AdjacencyList<N, W>, root: NodeIndex) -> Self {
        Self { graph, root }
    }
}

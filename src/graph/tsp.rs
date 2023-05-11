use crate::{
    adjacency_list::AdjacencyOptions,
    edge::{Edge, EdgeRef},
    error::{GraphError, GraphResult},
    indices::NodeIndex,
    prelude::AdjacencyList,
};
use std::ops::{Add, AddAssign};

use super::{
    access::{GraphAccess, GraphCompare},
    mst::{PrivateGraphMst, Sortable},
    search::PrivateGraphSearch,
    topology::{GraphAdjacentTopology, GraphTopology},
    GraphMst,
};

// Sortable + PartialEq

pub trait GraphTsp<
    N: PartialEq,
    W: Sortable + PartialOrd + Default + Add<W, Output = W> + AddAssign + Clone,
>:
    GraphTopology<N, W>
    + GraphAdjacentTopology<N, W>
    + GraphAccess<N, W>
    + GraphCompare<N, W>
    + GraphMst<N, W>
    + Sized
    + Clone
{
    fn nearest_neighbor(&self) -> GraphResult<W> {
        match self.indices().next() {
            Some(start) => self._nearest_neighbor(start),
            None => Ok(W::default()),
        }
    }

    fn double_tree(&mut self) -> GraphResult<W> {
        let mut mst = AdjacencyList::with(AdjacencyOptions {
            directed: self.directed(),
            nodes: Some(self.nodes().collect()),
        });

        let union_find = self._kruskal(|edge| {
            mst.add_edge(edge.from, edge.to, edge.weight.clone())
                .unwrap();
            mst.add_edge(edge.to, edge.from, edge.weight.clone())
                .unwrap();
        });
        let root = union_find.root();

        let mut euler_tour = vec![];
        let mut visited = vec![false; self.node_count()];

        mst.depth_search(root, &mut visited, true, |index| {
            euler_tour.push(index);
        });

        euler_tour.push(root);

        let mut total_weight = W::default();
        for [from, to] in euler_tour.array_windows::<2>() {
            let weight = match mst.contains_edge(*from, *to) {
                Some(index) => mst.weight(index).clone(),
                None => self.djikstra(*from, *to).ok_or(GraphError::NoCycle)?,
            };
            total_weight += weight;
        }

        if visited.into_iter().all(|visit| visit == true) {
            Ok(total_weight)
        } else {
            Err(GraphError::NoCycle)
        }
    }

    fn branch_bound(&self) -> GraphResult<W> {
        match self.indices().next() {
            Some(start) => self._branch_bound(start),
            None => Ok(W::default()),
        }
    }
}

impl<
        N: PartialEq,
        W: PartialOrd + Default + Add<W, Output = W> + AddAssign + Clone + Sortable,
        T: GraphTopology<N, W>
            + GraphAdjacentTopology<N, W>
            + GraphAccess<N, W>
            + GraphCompare<N, W>
            + Clone,
    > GraphTsp<N, W> for T
{
}

trait PrivateGraphTsp<
    N: PartialEq,
    W: PartialOrd + Default + Add<W, Output = W> + AddAssign + Clone,
>: GraphTopology<N, W> + GraphAdjacentTopology<N, W> + GraphAccess<N, W> + GraphCompare<N, W>
{
    fn _branch_bound(&self, start: NodeIndex) -> GraphResult<W> {
        let mut stack = Vec::new();
        let mut total_cost = self._nearest_neighbor(start)?;

        stack.push((W::default(), vec![start], vec![false; self.node_count()]));

        while let Some((cost, path, visited)) = stack.pop() {
            let node = path
                .last()
                .expect("INTERNAL: Path always expected to have atleast one element");

            for EdgeRef {
                from: _,
                to,
                weight,
            } in self.adjacent_edges(*node)
            {
                let cost = cost.clone() + weight.clone();

                if !visited[to.0] && cost < total_cost {
                    let mut visited = visited.clone();
                    visited[to.0] = true;

                    let mut path = path.clone();
                    path.push(to);

                    if visited.iter().all(|v| *v == true) {
                        total_cost = cost;
                    } else {
                        stack.push((cost, path, visited));
                    }
                }
            }
        }

        Ok(total_cost)
    }

    fn _nearest_neighbor(&self, start: NodeIndex) -> GraphResult<W> {
        let mut visited = vec![false; self.node_count()];
        let mut total_weight = W::default();
        let mut route = vec![(start, W::default())];
        let mut prev = start;

        while let Some((node, weight)) = route.last() {
            visited[node.0] = true;
            total_weight += weight.to_owned();

            if visited.iter().all(|v| *v) {
                break;
            }

            let mut min_edge: Option<Edge<W>> = None;

            for edge in self.adjacent_edges(*node) {
                if !visited[edge.to.0] && edge.to != prev {
                    if let Some(min) = &min_edge {
                        if min.weight > *edge.weight {
                            min_edge = Some(edge.into());
                        }
                    } else {
                        min_edge = Some(edge.into());
                    }
                }
            }

            match min_edge {
                Some(edge) => {
                    route.push((edge.to, edge.weight));
                    prev = edge.to;
                }
                None => {
                    let dead_end = match route.pop() {
                        Some((end, _)) => end,
                        None => break,
                    };
                    visited[dead_end.0] = false;
                    prev = dead_end;
                }
            }
        }

        if visited.into_iter().all(|visit| visit == true) {
            if let Some(edge_index) = self.contains_edge(route.last().unwrap().0, start) {
                total_weight += self.weight(edge_index).to_owned();
                return Ok(total_weight);
            }
        }
        Err(GraphError::NNAbort)
    }
}

impl<
        N: PartialEq,
        W: PartialOrd + Default + Add<W, Output = W> + AddAssign + Clone,
        T: GraphTopology<N, W> + GraphAdjacentTopology<N, W> + GraphAccess<N, W> + GraphCompare<N, W>,
    > PrivateGraphTsp<N, W> for T
{
}

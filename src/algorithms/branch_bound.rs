use super::{_nearest_neighbor, dijkstra};
use crate::{
    edge::EdgeRef,
    prelude::{GraphAdjacentTopology, GraphTopology, Maximum, NodeIndex, Sortable},
};
use std::ops::{Add, AddAssign};

pub fn branch_bound<N, W, G>(graph: &G) -> Option<W>
where
    W: Default + Copy + AddAssign + Add<W, Output = W> + Maximum + Sortable,
    G: GraphTopology<N, W> + GraphAdjacentTopology<N, W>,
{
    match graph.indices().next() {
        Some(start) => Some(_branch_bound(graph, start)),
        None => None,
    }
}

pub(crate) fn _branch_bound<N, W, G>(graph: &G, start: NodeIndex) -> W
where
    W: Default + Copy + AddAssign + Add<W, Output = W> + Maximum + Sortable,
    G: GraphTopology<N, W> + GraphAdjacentTopology<N, W>,
{
    let mut stack = Vec::new();
    let mut total_cost = _nearest_neighbor(graph, start).unwrap_or(Maximum::max());

    let mut visited = vec![false; graph.node_count()];
    visited[start.0] = true;

    stack.push((W::default(), vec![start], visited));

    while let Some((cost, path, visited)) = stack.pop() {
        let node = path
            .last()
            .expect("INTERNAL: Path always expected to have atleast one element");

        for EdgeRef {
            from: _,
            to,
            weight,
        } in graph.adjacent_edges(*node)
        {
            let cost = cost + *weight;

            if !visited[to.0] && cost < total_cost {
                let mut visited = visited.clone();
                visited[to.0] = true;

                let mut path = path.clone();
                path.push(to);

                if visited.iter().all(|v| *v == true) {
                    if let Some(cost_to_start) = dijkstra(graph, path[path.len() - 1], start) {
                        let cost = cost + cost_to_start;

                        if cost < total_cost {
                            total_cost = cost;
                        }
                    }
                } else {
                    stack.push((cost, path, visited));
                }
            }
        }
    }

    total_cost
}

pub(crate) fn _branch_bound_rec<N, W, G>(
    graph: &G,
    node: NodeIndex,
    path: &Vec<NodeIndex>,
    visited: &Vec<bool>,
    mut cost: W,
    baseline: &mut W,
) where
    W: Copy + Add<W, Output = W> + PartialOrd,
    G: GraphAdjacentTopology<N, W>,
{
    for EdgeRef {
        from: _,
        to,
        weight,
    } in graph.adjacent_edges(node)
    {
        let combined_cost = cost + *weight;

        if !visited[to.0] && combined_cost < *baseline {}
    }
}

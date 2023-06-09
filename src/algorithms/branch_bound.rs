use super::{dijkstra_between, nearest_neighbor};
use crate::{
    graph::{Base, Count, EdgeCost, Index, IndexAdjacent, IterAdjacent, Maximum, Sortable},
    prelude::{EdgeRef, NodeId},
    structures::Route,
};
use std::ops::{Add, AddAssign};

pub fn branch_bound<N, W, C, G>(graph: &G) -> Option<(Route<G>, W::Cost)>
where
    C: Default + Copy + AddAssign + Add<C, Output = C> + Maximum + Sortable,
    W: EdgeCost<Cost = C>,
    G: Index + IndexAdjacent + Count + IterAdjacent + Base<Node = N, Weight = W>,
{
    match graph.node_ids().next() {
        Some(start) => Some(_branch_bound(graph, start)),
        None => None,
    }
}

pub fn branch_bound_rec<N, W, C, G>(graph: &G) -> Option<(Route<G>, W::Cost)>
where
    C: Default + Copy + Add<C, Output = C> + AddAssign + PartialOrd + Sortable + Maximum,
    W: EdgeCost<Cost = C>,
    G: Index + IndexAdjacent + Count + IterAdjacent + Base<Node = N, Weight = W>,
{
    match graph.node_ids().next() {
        Some(start) => {
            let mut baseline = nearest_neighbor(graph, start)
                .map(|tour| tour.1)
                .unwrap_or(Maximum::MAX);
            let mut path = vec![start];
            let mut visited = vec![false; graph.node_count()];
            let cost = C::default();

            _branch_bound_rec(
                start,
                graph,
                start,
                &mut path,
                &mut visited,
                cost,
                &mut baseline,
            );

            Some((Route::new(path), baseline))
        }
        None => None,
    }
}

pub(crate) fn _branch_bound<N, W, C, G>(graph: &G, start: NodeId<G::Id>) -> (Route<G>, W::Cost)
where
    C: Default + Copy + AddAssign + Add<C, Output = C> + Maximum + Sortable,
    W: EdgeCost<Cost = C>,
    G: Count + IndexAdjacent + IterAdjacent + Base<Node = N, Weight = W>,
{
    let mut stack = Vec::new();
    let mut total_cost = nearest_neighbor(graph, start)
        .map(|tour| tour.1)
        .unwrap_or(Maximum::MAX);
    let mut route = Vec::new();

    let mut visited = vec![false; graph.node_count()];
    visited[start.as_usize()] = true;

    stack.push((C::default(), vec![start], visited));

    while let Some((cost, path, visited)) = stack.pop() {
        let node = path
            .last()
            .expect("INTERNAL: Path always expected to have atleast one element");

        for EdgeRef { edge_id, weight } in graph.iter_adjacent_edges(*node) {
            let to = edge_id.to();
            let cost = cost + *weight.cost();

            if !visited[to.as_usize()] && cost < total_cost {
                let mut visited = visited.clone();
                visited[to.as_usize()] = true;

                let mut path = path.clone();
                path.push(to);

                if visited.iter().all(|v| *v == true) {
                    if let Some(cost_to_start) =
                        dijkstra_between(graph, path[path.len() - 1], start)
                    {
                        let cost = cost + cost_to_start;

                        if cost < total_cost {
                            total_cost = cost;
                            std::mem::swap(&mut path, &mut route);
                        }
                    }
                } else {
                    stack.push((cost, path, visited));
                }
            }
        }
    }

    (Route::new(route), total_cost)
}

pub(crate) fn _branch_bound_rec<N, W, C, G>(
    start: NodeId<G::Id>,
    graph: &G,
    node: NodeId<G::Id>,
    path: &mut Vec<NodeId<G::Id>>,
    visited: &mut Vec<bool>,
    cost: C,
    baseline: &mut C,
) where
    C: Default + Copy + Add<C, Output = C> + AddAssign + PartialOrd + Sortable,
    W: EdgeCost<Cost = C>,
    G: IndexAdjacent + IterAdjacent + Count + Base<Node = N, Weight = W>,
{
    if visited.iter().all(|v| *v) && let Some(cost_to_start) = dijkstra_between(graph, node, start) {
        let total_cost = cost + cost_to_start;
        if total_cost < *baseline {
            *baseline = total_cost;
        }
    }

    for EdgeRef { edge_id, weight } in graph.iter_adjacent_edges(node) {
        let to = edge_id.to();
        let cost = cost + *weight.cost();

        if !visited[to.as_usize()] && cost < *baseline {
            visited[to.as_usize()] = true;
            path.push(to);

            _branch_bound_rec(start, graph, to, path, visited, cost, baseline);

            visited[to.as_usize()] = false;
            path.pop();
        }
    }
}

#[cfg(test)]
mod test {
    extern crate test;

    use crate::{prelude::*, test::undigraph};
    use test::Bencher;

    #[bench]
    fn branch_bound_k_10_adj_list(b: &mut Bencher) {
        let graph: AdjacencyList<_, _> = undigraph("data/K_10.txt").unwrap();

        b.iter(|| {
            let total = graph.branch_bound().unwrap().1 as f32;
            assert_eq!(total, 38.41);
        })
    }

    #[bench]
    fn branch_bound_k_10e_adj_list(b: &mut Bencher) {
        let graph: AdjacencyList<_, _> = undigraph("data/K_10e.txt").unwrap();

        b.iter(|| {
            let total = graph.branch_bound().unwrap().1 as f32;
            assert_eq!(total, 27.26);
        })
    }

    #[bench]
    fn branch_bound_k_12_adj_list(b: &mut Bencher) {
        let graph: AdjacencyList<_, _> = undigraph("data/K_12.txt").unwrap();

        b.iter(|| {
            let total = graph.branch_bound().unwrap().1 as f32;
            assert_eq!(total, 45.19);
        })
    }

    #[bench]
    fn branch_bound_k_12e_adj_list(b: &mut Bencher) {
        let graph: AdjacencyList<_, _> = undigraph("data/K_12e.txt").unwrap();

        b.iter(|| {
            let total = graph.branch_bound().unwrap().1 as f32;
            assert_eq!(total, 36.13);
        })
    }

    #[bench]
    fn branch_bound_rec_k_10_adj_list(b: &mut Bencher) {
        let graph: AdjacencyList<_, _> = undigraph("data/K_10.txt").unwrap();

        b.iter(|| {
            let total = graph.branch_bound_rec().unwrap().1 as f32;
            assert_eq!(total, 38.41);
        })
    }

    #[bench]
    fn branch_bound_rec_k_10e_adj_list(b: &mut Bencher) {
        let graph: AdjacencyList<_, _> = undigraph("data/K_10e.txt").unwrap();

        b.iter(|| {
            let total = graph.branch_bound_rec().unwrap().1 as f32;
            assert_eq!(total, 27.26);
        })
    }

    #[bench]
    fn branch_bound_rec_k_12_adj_list(b: &mut Bencher) {
        let graph: AdjacencyList<_, _> = undigraph("data/K_12.txt").unwrap();

        b.iter(|| {
            let total = graph.branch_bound_rec().unwrap().1 as f32;
            assert_eq!(total, 45.19);
        })
    }

    #[bench]
    fn branch_bound_rec_k_12e_adj_list(b: &mut Bencher) {
        let graph: AdjacencyList<_, _> = undigraph("data/K_12e.txt").unwrap();

        b.iter(|| {
            let total = graph.branch_bound_rec().unwrap().1 as f32;
            assert_eq!(total, 36.13);
        })
    }

    #[bench]
    fn branch_bound_k_10_adj_mat(b: &mut Bencher) {
        let graph: AdjacencyMatrix<_, _> = undigraph("data/K_10.txt").unwrap();

        b.iter(|| {
            let total = graph.branch_bound().unwrap().1 as f32;
            assert_eq!(total, 38.41);
        })
    }

    #[bench]
    fn branch_bound_k_10e_adj_mat(b: &mut Bencher) {
        let graph: AdjacencyMatrix<_, _> = undigraph("data/K_10e.txt").unwrap();

        b.iter(|| {
            let total = graph.branch_bound().unwrap().1 as f32;
            assert_eq!(total, 27.26);
        })
    }

    #[bench]
    fn branch_bound_k_12_adj_mat(b: &mut Bencher) {
        let graph: AdjacencyMatrix<_, _> = undigraph("data/K_12.txt").unwrap();

        b.iter(|| {
            let total = graph.branch_bound().unwrap().1 as f32;
            assert_eq!(total, 45.19);
        })
    }

    #[bench]
    fn branch_bound_k_12e_adj_mat(b: &mut Bencher) {
        let graph: AdjacencyMatrix<_, _> = undigraph("data/K_12e.txt").unwrap();

        b.iter(|| {
            let total = graph.branch_bound().unwrap().1 as f32;
            assert_eq!(total, 36.13);
        })
    }

    #[bench]
    fn branch_bound_rec_k_10_adj_mat(b: &mut Bencher) {
        let graph: AdjacencyMatrix<_, _> = undigraph("data/K_10.txt").unwrap();

        b.iter(|| {
            let total = graph.branch_bound_rec().unwrap().1 as f32;
            assert_eq!(total, 38.41);
        })
    }

    #[bench]
    fn branch_bound_rec_k_10e_adj_mat(b: &mut Bencher) {
        let graph: AdjacencyMatrix<_, _> = undigraph("data/K_10e.txt").unwrap();

        b.iter(|| {
            let total = graph.branch_bound_rec().unwrap().1 as f32;
            assert_eq!(total, 27.26);
        })
    }

    #[bench]
    fn branch_bound_rec_k_12_adj_mat(b: &mut Bencher) {
        let graph: AdjacencyMatrix<_, _> = undigraph("data/K_12.txt").unwrap();

        b.iter(|| {
            let total = graph.branch_bound_rec().unwrap().1 as f32;
            assert_eq!(total, 45.19);
        })
    }

    #[bench]
    fn branch_bound_rec_k_12e_adj_mat(b: &mut Bencher) {
        let graph: AdjacencyMatrix<_, _> = undigraph("data/K_12e.txt").unwrap();

        b.iter(|| {
            let total = graph.branch_bound_rec().unwrap().1 as f32;
            assert_eq!(total, 36.13);
        })
    }
}

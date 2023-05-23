use std::ops::AddAssign;

use crate::graph::{Contains, Get, Index, Maximum};

use super::Tour;

pub fn brute_force<N, W, G>(graph: &G) -> Option<Tour<G::NodeId, W>>
where
    N: PartialEq,
    W: Default + Maximum + PartialOrd + AddAssign + Copy,
    G: Get<N, W> + Index + Contains<N>,
{
    let mut best_path = Vec::new();
    let mut best_weight = W::max();

    let start = graph.node_ids().collect::<Vec<_>>();

    for perm in permute::permutations_of(&start) {
        let mut perm = perm.map(ToOwned::to_owned).collect::<Vec<_>>();
        perm.push(perm[0]);

        let edges = perm
            .array_windows::<2>()
            .map(|w| graph.contains_edge(w[0], w[1]))
            .collect::<Option<Vec<_>>>();

        if let Some(edges) = edges {
            let total_weight = edges
                .into_iter()
                .map(|edge| *graph.weight(edge).unwrap())
                .fold(W::default(), |mut accu, w| {
                    accu += w;
                    accu
                });

            if total_weight < best_weight {
                best_path = perm.clone();
                best_weight = total_weight;
            }
        }
    }

    if best_weight == W::max() {
        None
    } else {
        Some(Tour::new(best_path, best_weight))
    }
}

use std::ops::AddAssign;

use crate::{
    graph::{Base, Contains, EdgeCost, Get, Index, Maximum},
    structures::Route,
};

pub fn brute_force<N, W, C, G>(graph: &G) -> Option<(Route<G>, C)>
where
    N: PartialEq,
    C: Default + Maximum + PartialOrd + AddAssign + Copy,
    W: EdgeCost<Cost = C>,
    G: Get + Index + Contains + Base<Node = N, Weight = W>,
{
    let mut best_path = Vec::new();
    let mut best_weight = C::MAX;

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
                .map(|edge| *graph.weight(edge).unwrap().cost())
                .fold(C::default(), |mut accu, w| {
                    accu += w;
                    accu
                });

            if total_weight < best_weight {
                best_path = perm.clone();
                best_weight = total_weight;
            }
        }
    }

    if best_weight == C::MAX {
        None
    } else {
        Some((Route::new(best_path), best_weight))
    }
}

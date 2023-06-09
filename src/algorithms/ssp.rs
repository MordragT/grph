use std::{
    cmp::min_by,
    fmt::Debug,
    ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign},
};

use num_traits::{Float, Pow};

use crate::{
    algorithms::{Mcf, _ford_fulkerson, bfs_sp},
    graph::{
        Base, Count, EdgeCapacity, EdgeCost, EdgeDirection, EdgeFlow, Get, GetMut, Index,
        IndexAdjacent, Insert, Iter, IterAdjacent, IterMut, NodeBalance, Remove, Sortable,
    },
    prelude::{EdgeRef, NodeId},
    structures::Parents,
};

use super::{bellman_ford, dijkstra};

/// successive shortest path
pub fn ssp<N, W, C, G>(graph: &G) -> Option<C>
where
    N: Default + NodeBalance<Balance = C>,
    W: EdgeCapacity<Capacity = C>
        + EdgeCost<Cost = C>
        + Default
        + EdgeDirection
        + EdgeFlow<Flow = C>,
    C: Default
        + PartialOrd
        + Copy
        + Neg<Output = C>
        + AddAssign
        + SubAssign
        + Debug
        + Sub<C, Output = C>
        + Mul<C, Output = C>
        + Add<C, Output = C>
        + Sortable,
    G: Index
        + Get
        + GetMut
        + Insert
        + Remove
        + Count
        + IndexAdjacent
        + IterAdjacent
        + Iter
        + IterMut
        + Clone
        + Base<Node = N, Weight = W>
        + Debug,
{
    // let Mcf {
    //     mut residual_graph,
    //     source,
    //     sink,
    // } = Mcf::init(graph);

    // pseudo flow, erfüllt kapazitätsbedinungen aber verletzt ddie massenbalancebedingung
    let mut delta = match graph
        .iter_edges()
        .map(|edge| edge.weight.capacity())
        .cloned()
        .reduce(C::min)
    {
        Some(d) => d,
        None => return None,
    };

    let mut graphR = graph.clone();

    for edge in graphR.iter_edges_mut() {
        *edge.weight.flow_mut() += delta;
    }

    for EdgeRef { edge_id, weight } in graph.iter_edges() {
        if !graphR.contains_edge_id(edge_id.rev()) {
            let mut w = W::default();
            *w.cost_mut() = -*weight.cost();
            *w.capacity_mut() = *weight.capacity();
            *w.flow_mut() = *weight.capacity() - *weight.flow();
            w.reverse();

            graphR.insert_edge(edge_id.to(), edge_id.from(), w);
        }
    }

    let cost = graphR.iter_edges().fold(C::default(), |mut akku, edge| {
        let weight = edge.weight;
        if !weight.is_reverse() {
            akku += *weight.flow() * *weight.cost();
        }

        akku
    });

    Some(cost)
}

#[cfg(test)]
mod test {
    use super::ssp;
    use crate::{prelude::AdjacencyList, test::bgraph};

    #[test]
    fn ssp_kostenminimal_1() {
        let graph: AdjacencyList<_, _, true> = bgraph("data/Kostenminimal1.txt").unwrap();
        let cost = ssp(&graph).unwrap();
        assert_eq!(cost, 3.0);
    }

    #[test]
    fn ssp_kostenminimal_2() {
        let graph: AdjacencyList<_, _, true> = bgraph("data/Kostenminimal2.txt").unwrap();
        let cost = ssp(&graph).unwrap();
        assert_eq!(cost, 0.0);
    }

    #[test]
    #[should_panic]
    fn ssp_kostenminimal_3() {
        let graph: AdjacencyList<_, _, true> = bgraph("data/Kostenminimal3.txt").unwrap();
        let _cost = ssp(&graph).unwrap();
    }

    #[test]
    #[should_panic]
    fn ssp_kostenminimal_4() {
        let graph: AdjacencyList<_, _, true> = bgraph("data/Kostenminimal4.txt").unwrap();
        let _cost = ssp(&graph).unwrap();
    }

    #[test]
    fn ssp_kostenminimal_gross_1() {
        let graph: AdjacencyList<_, _, true> = bgraph("data/Kostenminimal_gross1.txt").unwrap();
        let cost = ssp(&graph).unwrap();
        assert_eq!(cost, 1537.0);
    }

    #[test]
    fn ssp_kostenminimal_gross_2() {
        let graph: AdjacencyList<_, _, true> = bgraph("data/Kostenminimal_gross2.txt").unwrap();
        let cost = ssp(&graph).unwrap();
        assert_eq!(cost, 1838.0);
    }

    #[test]
    #[should_panic]
    fn ssp_kostenminimal_gross_3() {
        let graph: AdjacencyList<_, _, true> = bgraph("data/Kostenminimal_gross3.txt").unwrap();
        let _cost = ssp(&graph).unwrap();
    }
}

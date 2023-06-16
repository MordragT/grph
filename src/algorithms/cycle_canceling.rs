// no more sink and source (st) -> no st-flow
// b (balance)
// b-flow: sum of all b-flow in the graph is 0
// edge cannot carry more flow than its capacity
// the difference in flow entering and leaving a node
// must be equal b(v) (flow balance)

// wenn capacity von edge == 0 -> edge ignorieren
// supply und demand nodes
// wenn balance < 0 demand node, wenn balance > 0 supply node
// flow kann nicht mehr als die capacities sein aber auch nicht weniger als 0
// differenz zwischen flow der raus geht und reingeht muss gleich dem supply oder demand sein
// wenn flow durch nodes geht, dann muss bei demand-nodes der demand vom flow abgezogen werden,
// analog bei supply-nodes wird der supply addiert.
// möglicherweise nicht solvable wenn im netzwerk weniger supply als demand vorhanden ist, oder mehr supply als demand
// wir können checken ob MCF möglich wenn wir das problem in ein max flow problem überführen

// neuer graph g'

use std::{
    fmt::Debug,
    ops::{Add, AddAssign, Mul, Neg, SubAssign},
};

use either::Either;

use crate::{
    algorithms::{_edmonds_karp, bellman_ford},
    error::GraphResult,
    graph::{
        BalancedNode, Count, FlowWeight, Get, GetMut, Index, IndexAdjacent, Insert, Iter,
        IterAdjacent, Maximum, Remove, WeightCapacity, WeightCost,
    },
    prelude::{Edge, EdgeIdentifier, EdgeRef, GraphError, NodeIdentifier},
};

use super::_bellman_ford;

pub fn cycle_canceling<N, C, G>(graph: &G) -> GraphResult<C>
where
    N: Default,
    C: Maximum
        + Default
        + PartialOrd
        + Copy
        + Neg<Output = C>
        + AddAssign
        + SubAssign
        + Add<C, Output = C>
        + Debug,
    G: Index
        + Get<BalancedNode<N, C>, FlowWeight<C>>
        + GetMut<BalancedNode<N, C>, FlowWeight<C>>
        + Insert<BalancedNode<N, C>, FlowWeight<C>>
        + Remove<BalancedNode<N, C>, FlowWeight<C>>
        + Count
        + IndexAdjacent
        + Iter<BalancedNode<N, C>, FlowWeight<C>>
        + IterAdjacent<BalancedNode<N, C>, FlowWeight<C>>
        + Clone
        + Debug,
{
    let (start, mut residual_graph) = mcf_solvable(graph)?;
    let mut node_ids = residual_graph.node_ids();
    let start = node_ids.next().unwrap();
    // let start = node_ids.next().unwrap();
    let mut total_flow = C::default();

    // dbg!(residual_graph.iter_edges().collect::<Vec<_>>());

    drop(node_ids);

    // for start in graph.node_ids() {
    while let Either::Right(cycle) = _bellman_ford(&residual_graph, start, |weight| {
        weight.capacity() > &C::default()
    }) {
        // dbg!(&cycle);

        let mut bottleneck = C::max();

        assert!(bottleneck > C::default());

        for (&from, &to) in cycle.edges() {
            let edge_id = G::EdgeId::between(from, to);
            let residual_capacity = residual_graph.weight(edge_id).unwrap().capacity();

            if residual_capacity < &bottleneck {
                bottleneck = *residual_capacity;
            }
        }
        assert!(bottleneck > C::default());
        total_flow += bottleneck;

        // dbg!(bottleneck);

        for (&from, &to) in cycle.edges() {
            let edge_id = G::EdgeId::between(from, to);
            // dbg!(edge_id);

            let weight = residual_graph.weight_mut(edge_id).unwrap();
            // dbg!(&weight);
            *weight.capacity_mut() -= bottleneck;
            assert!(weight.capacity >= C::default());

            let weight_rev: &mut FlowWeight<C> = residual_graph.weight_mut(edge_id.rev()).unwrap();
            // dbg!(&weight_rev);
            *weight_rev.capacity_mut() += bottleneck;
            assert!(weight_rev.capacity >= C::default());
        }
        // }
    }

    Ok(total_flow)
}

fn init_residual_graph<N, C, G>(graph: &G) -> (G::NodeId, G::NodeId, G)
where
    N: Default,
    C: Default + PartialOrd + Copy + Neg<Output = C> + AddAssign + SubAssign + Debug,
    G: Index
        + Get<BalancedNode<N, C>, FlowWeight<C>>
        + GetMut<BalancedNode<N, C>, FlowWeight<C>>
        + Insert<BalancedNode<N, C>, FlowWeight<C>>
        + Remove<BalancedNode<N, C>, FlowWeight<C>>
        + Count
        + IndexAdjacent
        + Iter<BalancedNode<N, C>, FlowWeight<C>>
        + Clone,
{
    let mut residual_graph = graph.clone();

    let source = residual_graph.add_node(BalancedNode::new(N::default(), C::default()));
    let sink = residual_graph.add_node(BalancedNode::new(N::default(), C::default()));

    for node_id in graph.node_ids() {
        let node = residual_graph.node(node_id).unwrap();

        if node.balance > C::default() {
            // supply
            let edge_id = G::EdgeId::between(source, node_id);
            residual_graph.insert_edge(edge_id, FlowWeight::new(node.balance, C::default()));
            residual_graph.insert_edge(edge_id.rev(), FlowWeight::default());
        } else if node.balance < C::default() {
            // demand
            let edge_id = G::EdgeId::between(node_id, sink);
            residual_graph.insert_edge(edge_id, FlowWeight::new(-node.balance, C::default()));
            residual_graph.insert_edge(edge_id.rev(), FlowWeight::default());
        }
    }

    for EdgeRef { edge_id, weight } in graph.iter_edges() {
        if !residual_graph.contains_edge_id(edge_id.rev()) {
            let weight = FlowWeight::new(C::default(), -weight.cost);
            // dbg!(&weight);
            residual_graph.insert_edge(edge_id.rev(), weight);
        }
    }

    (source, sink, residual_graph)
}

fn mcf_solvable<N, C, G>(graph: &G) -> GraphResult<(G::NodeId, G)>
where
    N: Default,
    C: Default + PartialOrd + Copy + Neg<Output = C> + AddAssign + SubAssign + Debug,
    G: Index
        + Get<BalancedNode<N, C>, FlowWeight<C>>
        + GetMut<BalancedNode<N, C>, FlowWeight<C>>
        + Insert<BalancedNode<N, C>, FlowWeight<C>>
        + Remove<BalancedNode<N, C>, FlowWeight<C>>
        + Count
        + IndexAdjacent
        + Iter<BalancedNode<N, C>, FlowWeight<C>>
        + Clone,
{
    let (source, sink, mut residual_graph) = init_residual_graph(graph);

    let total_flow = _edmonds_karp(&mut residual_graph, source, sink);
    let expected = graph.iter_nodes().fold(C::default(), |mut akku, node| {
        // assert!(node.balance > C::default());
        if node.balance > C::default() {
            akku += node.balance;
        }
        akku
    });

    // dbg!(total_flow, expected);

    if total_flow != expected {
        Err(GraphError::McfNotSolvable)
    } else {
        // residual_graph.remove_node(source);
        // residual_graph.remove_node(sink);
        Ok((source, residual_graph))
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use more_asserts::assert_ge;

    use crate::{
        graph::{Get, IndexAdjacent, Iter},
        prelude::{AdjacencyList, EdgeIdentifier, EdgeIndex, EdgeRef, NodeIndex},
        test::bgraph,
    };

    use super::{cycle_canceling, init_residual_graph, mcf_solvable};

    fn mcf_residual_graph(path: &str) {
        let graph: AdjacencyList<_, _, true> = bgraph(path).unwrap();
        let (source, sink, residual_graph) = init_residual_graph(&graph);

        let mut nodes = HashSet::new();
        for node_id in residual_graph.adjacent_node_ids(source) {
            let weight = residual_graph
                .weight(EdgeIndex::between(source, node_id))
                .unwrap();
            let balance = residual_graph.node(node_id).unwrap();

            nodes.insert(node_id);
            assert_eq!(balance.balance, weight.capacity)
        }

        let expected = graph
            .iter_nodes()
            .enumerate()
            .filter_map(|(i, node)| {
                if node.balance > 0.0 {
                    Some(NodeIndex(i))
                } else {
                    None
                }
            })
            .collect::<HashSet<_>>();
        assert_eq!(nodes, expected);

        nodes.clear();

        for node_id in residual_graph.adjacent_node_ids(sink) {
            let weight = residual_graph
                .weight(EdgeIndex::between(node_id, sink))
                .unwrap();
            let balance = residual_graph.node(node_id).unwrap();

            assert_eq!(-balance.balance, weight.capacity);
            nodes.insert(node_id);
        }

        let expected = graph
            .iter_nodes()
            .enumerate()
            .filter_map(|(i, node)| {
                if node.balance < 0.0 {
                    Some(NodeIndex(i))
                } else {
                    None
                }
            })
            .collect::<HashSet<_>>();
        assert_eq!(nodes, expected);

        for EdgeRef { edge_id: _, weight } in residual_graph.iter_edges() {
            assert_ge!(weight.capacity, 0.0);
        }
    }

    #[test]
    fn mcf_residual_graph_kostenminimal_1() {
        mcf_residual_graph("data/Kostenminimal1.txt")
    }

    #[test]
    fn mcf_residual_graph_kostenminimal_gross_1() {
        mcf_residual_graph("data/Kostenminimal_gross1.txt")
    }

    #[test]
    fn mcf_residual_graph_kostenminimal_gross_2() {
        mcf_residual_graph("data/Kostenminimal_gross2.txt")
    }

    #[test]
    fn mcf_solvable_kostenminimal_1() {
        let graph: AdjacencyList<_, _, true> = bgraph("data/Kostenminimal1.txt").unwrap();
        mcf_solvable(&graph).unwrap();
    }

    #[test]
    fn mcf_solvable_kostenminimal_2() {
        let graph: AdjacencyList<_, _, true> = bgraph("data/Kostenminimal2.txt").unwrap();
        mcf_solvable(&graph).unwrap();
    }

    #[test]
    #[should_panic]
    fn mcf_solvable_kostenminimal_3() {
        let graph: AdjacencyList<_, _, true> = bgraph("data/Kostenminimal3.txt").unwrap();
        mcf_solvable(&graph).unwrap();
    }

    #[test]
    #[should_panic]
    fn mcf_solvable_kostenminimal_4() {
        let graph: AdjacencyList<_, _, true> = bgraph("data/Kostenminimal4.txt").unwrap();
        mcf_solvable(&graph).unwrap();
    }

    #[test]
    fn mcf_solvable_kostenminimal_gross_1() {
        let graph: AdjacencyList<_, _, true> = bgraph("data/Kostenminimal_gross1.txt").unwrap();
        mcf_solvable(&graph).unwrap();
    }

    #[test]
    fn mcf_solvable_kostenminimal_gross_2() {
        let graph: AdjacencyList<_, _, true> = bgraph("data/Kostenminimal_gross2.txt").unwrap();
        mcf_solvable(&graph).unwrap();
    }

    #[test]
    #[should_panic]
    fn mcf_solvable_kostenminimal_gross_3() {
        let graph: AdjacencyList<_, _, true> = bgraph("data/Kostenminimal_gross3.txt").unwrap();
        mcf_solvable(&graph).unwrap();
    }

    #[test]
    fn cycle_canceling_kostenminimal_1() {
        let graph: AdjacencyList<_, _, true> = bgraph("data/Kostenminimal1.txt").unwrap();
        let flow = cycle_canceling(&graph).unwrap();
        assert_eq!(flow, 3.0);
    }

    #[test]
    fn cycle_canceling_kostenminimal_2() {
        let graph: AdjacencyList<_, _, true> = bgraph("data/Kostenminimal2.txt").unwrap();
        let flow = cycle_canceling(&graph).unwrap();
        assert_eq!(flow, 0.0);
    }

    #[test]
    #[should_panic]
    fn cycle_canceling_kostenminimal_3() {
        let graph: AdjacencyList<_, _, true> = bgraph("data/Kostenminimal3.txt").unwrap();
        let _flow = cycle_canceling(&graph).unwrap();
    }

    #[test]
    #[should_panic]
    fn cycle_canceling_kostenminimal_4() {
        let graph: AdjacencyList<_, _, true> = bgraph("data/Kostenminimal4.txt").unwrap();
        let _flow = cycle_canceling(&graph).unwrap();
    }

    #[test]
    fn cycle_canceling_kostenminimal_gross_1() {
        let graph: AdjacencyList<_, _, true> = bgraph("data/Kostenminimal_gross1.txt").unwrap();
        let flow = cycle_canceling(&graph).unwrap();
        assert_eq!(flow, 1537.0);
    }

    #[test]
    fn cycle_canceling_kostenminimal_gross_2() {
        let graph: AdjacencyList<_, _, true> = bgraph("data/Kostenminimal_gross2.txt").unwrap();
        let flow = cycle_canceling(&graph).unwrap();
        assert_eq!(flow, 1838.0);
    }

    #[test]
    #[should_panic]
    fn cycle_canceling_kostenminimal_gross_3() {
        let graph: AdjacencyList<_, _, true> = bgraph("data/Kostenminimal_gross3.txt").unwrap();
        let _flow = cycle_canceling(&graph).unwrap();
    }
}

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

use std::ops::{AddAssign, Neg, SubAssign};

use crate::{
    algorithms::_edmonds_karp,
    error::GraphResult,
    graph::{
        BalancedNode, CapacityWeight, Count, EdgeCapacity, EdgeCost, Get, GetMut, Index,
        IndexAdjacent, Insert, Iter, Remove,
    },
    prelude::{Edge, EdgeIdentifier, EdgeRef, GraphError},
};

pub fn cycle_canceling<N, C, G>(graph: &G) -> GraphResult<C>
where
    N: Default,
    C: Default + PartialOrd + Copy + Neg<Output = C> + AddAssign + SubAssign,
    G: Index
        + Get<BalancedNode<N, C>, CapacityWeight<C>>
        + GetMut<BalancedNode<N, C>, CapacityWeight<C>>
        + Insert<BalancedNode<N, C>, CapacityWeight<C>>
        + Remove<BalancedNode<N, C>, CapacityWeight<C>>
        + Count
        + IndexAdjacent
        + Iter<BalancedNode<N, C>, CapacityWeight<C>>
        + Clone,
{
    let residual_graph = mcf_solvable(graph)?;

    todo!()
}

fn mcf_solvable<N, C, G>(graph: &G) -> GraphResult<G>
where
    N: Default,
    C: Default + PartialOrd + Copy + Neg<Output = C> + AddAssign + SubAssign,
    G: Index
        + Get<BalancedNode<N, C>, CapacityWeight<C>>
        + GetMut<BalancedNode<N, C>, CapacityWeight<C>>
        + Insert<BalancedNode<N, C>, CapacityWeight<C>>
        + Remove<BalancedNode<N, C>, CapacityWeight<C>>
        + Count
        + IndexAdjacent
        + Iter<BalancedNode<N, C>, CapacityWeight<C>>
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
            residual_graph.insert_edge(edge_id, CapacityWeight::new(node.balance, C::default()));
            residual_graph.insert_edge(edge_id.rev(), CapacityWeight::default());
        } else {
            // demand
            let edge_id = G::EdgeId::between(node_id, sink);
            residual_graph.insert_edge(edge_id, CapacityWeight::new(-node.balance, C::default()));
            residual_graph.insert_edge(edge_id.rev(), CapacityWeight::default());
        }
    }

    for EdgeRef { edge_id, weight } in graph.iter_edges() {
        if !residual_graph.contains_edge_id(edge_id.rev()) {
            residual_graph.insert_edge(
                edge_id.rev(),
                CapacityWeight::new(C::default(), weight.cost),
            );
        }
    }

    let total_flow = _edmonds_karp(&mut residual_graph, source, sink);
    let expected = graph.iter_nodes().fold(C::default(), |mut akku, node| {
        if node.balance > C::default() {
            akku += node.balance;
        }
        akku
    });

    if total_flow != expected {
        Err(GraphError::McfNotSolvable)
    } else {
        residual_graph.remove_node(source);
        residual_graph.remove_node(sink);
        Ok(residual_graph)
    }
}

#[cfg(test)]
mod test {
    use crate::{prelude::AdjacencyList, test::bgraph};

    use super::{cycle_canceling, mcf_solvable};

    #[test]
    fn mcf_solvable_kostenminimal_1() {
        let graph: AdjacencyList<_, _> = bgraph("data/Kostenminimal1.txt").unwrap();
        mcf_solvable(&graph).unwrap();
    }

    #[test]
    fn mcf_solvable_kostenminimal_2() {
        let graph: AdjacencyList<_, _> = bgraph("data/Kostenminimal2.txt").unwrap();
        mcf_solvable(&graph).unwrap();
    }

    #[test]
    #[should_panic]
    fn mcf_solvable_kostenminimal_3() {
        let graph: AdjacencyList<_, _> = bgraph("data/Kostenminimal3.txt").unwrap();
        mcf_solvable(&graph).unwrap();
    }

    #[test]
    #[should_panic]
    fn mcf_solvable_kostenminimal_4() {
        let graph: AdjacencyList<_, _> = bgraph("data/Kostenminimal4.txt").unwrap();
        mcf_solvable(&graph).unwrap();
    }

    #[test]
    fn cycle_canceling_kostenminimal_1() {
        let graph: AdjacencyList<_, _> = bgraph("data/Kostenminimal1.txt").unwrap();
        let flow = cycle_canceling(&graph).unwrap();
        assert_eq!(flow, 3.0);
    }

    #[test]
    fn cycle_canceling_kostenminimal_2() {
        let graph: AdjacencyList<_, _> = bgraph("data/Kostenminimal2.txt").unwrap();
        let flow = cycle_canceling(&graph).unwrap();
        assert_eq!(flow, 0.0);
    }

    #[test]
    #[should_panic]
    fn cycle_canceling_kostenminimal_3() {
        let graph: AdjacencyList<_, _> = bgraph("data/Kostenminimal3.txt").unwrap();
        let _flow = cycle_canceling(&graph).unwrap();
    }

    #[test]
    #[should_panic]
    fn cycle_canceling_kostenminimal_4() {
        let graph: AdjacencyList<_, _> = bgraph("data/Kostenminimal4.txt").unwrap();
        let _flow = cycle_canceling(&graph).unwrap();
    }

    #[test]
    fn cycle_canceling_kostenminimal_gross_1() {
        let graph: AdjacencyList<_, _> = bgraph("data/Kostenminimal_gross1.txt").unwrap();
        let flow = cycle_canceling(&graph).unwrap();
        assert_eq!(flow, 1537.0);
    }

    #[test]
    fn cycle_canceling_kostenminimal_gross_2() {
        let graph: AdjacencyList<_, _> = bgraph("data/Kostenminimal_gross2.txt").unwrap();
        let flow = cycle_canceling(&graph).unwrap();
        assert_eq!(flow, 1838.0);
    }

    #[test]
    #[should_panic]
    fn cycle_canceling_kostenminimal_gross_3() {
        let graph: AdjacencyList<_, _> = bgraph("data/Kostenminimal_gross3.txt").unwrap();
        let _flow = cycle_canceling(&graph).unwrap();
    }
}

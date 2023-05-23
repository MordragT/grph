use super::{EdgeIndex, NodeIndex};
use crate::{
    edge_list::EdgeList,
    graph::{
        Base, Capacity, Clear, Contains, Count, Create, Directed, EdgeIdentifier, Extend, Get,
        GetMut, Graph, Index, IndexAdjacent, Insert, Iter, IterAdjacent, IterAdjacentMut, IterMut,
        Remove, Reserve,
    },
    prelude::{EdgeRef, EdgeRefMut, WeightlessGraph},
    utils::SparseMatrix,
};

#[derive(Debug, Clone)]
pub struct AdjacencyMatrix<Node, Weight, const Di: bool = false> {
    pub(crate) nodes: Vec<Node>,
    pub(crate) edges: SparseMatrix<Weight>,
}

impl<W: Copy, const Di: bool> From<EdgeList<usize, W, Di>> for AdjacencyMatrix<usize, W, Di> {
    fn from(edge_list: EdgeList<usize, W, Di>) -> Self {
        let EdgeList {
            parents,
            children,
            weights,
            node_count,
        } = edge_list;

        let mut adj_mat = Self::with_capacity(node_count, parents.len());

        for ((from, to), weight) in parents
            .into_iter()
            .zip(children.into_iter())
            .zip(weights.into_iter())
        {
            adj_mat.nodes[from] = from;
            adj_mat.nodes[to] = to;

            let edge_id = EdgeIndex::new(NodeIndex(from), NodeIndex(to));

            if !Di {
                adj_mat.insert_edge(edge_id.rev(), weight);
            }

            adj_mat.insert_edge(edge_id, weight);
        }

        adj_mat
    }
}

impl<Node, Weight, const Di: bool> Base for AdjacencyMatrix<Node, Weight, Di> {
    type EdgeId = EdgeIndex;
    type NodeId = NodeIndex;
}

impl<Node, Weight, const Di: bool> Capacity for AdjacencyMatrix<Node, Weight, Di> {
    fn edges_capacity(&self) -> usize {
        todo!()
    }

    fn nodes_capacity(&self) -> usize {
        self.nodes.capacity()
    }
}

impl<Node, Weight, const Di: bool> Clear for AdjacencyMatrix<Node, Weight, Di> {
    fn clear(&mut self) {
        self.nodes.clear();
        self.edges.clear();
    }
}

impl<Node: PartialEq, Weight, const Di: bool> Contains<Node> for AdjacencyMatrix<Node, Weight, Di> {
    fn contains_node(&self, node: &Node) -> Option<Self::NodeId> {
        self.nodes
            .iter()
            .enumerate()
            .find(|(_i, other)| *other == node)
            .map(|(id, _)| NodeIndex(id))
    }

    fn contains_edge(&self, from: Self::NodeId, to: Self::NodeId) -> Option<Self::EdgeId> {
        let edge_id = EdgeIndex::new(from, to);
        if self.contains_edge_id(edge_id) {
            Some(edge_id)
        } else {
            None
        }
    }
}

impl<Node, Weight, const Di: bool> Count for AdjacencyMatrix<Node, Weight, Di> {
    fn edge_count(&self) -> usize {
        todo!()
    }

    fn node_count(&self) -> usize {
        self.nodes.len()
    }
}

impl<Node, Weight, const Di: bool> Create<Node> for AdjacencyMatrix<Node, Weight, Di> {
    fn with_capacity(nodes: usize, _edges: usize) -> Self {
        let edges = SparseMatrix::with_capacity(nodes, nodes);
        let nodes = Vec::with_capacity(nodes);

        Self { nodes, edges }
    }

    fn with_nodes(nodes: impl Iterator<Item = Node>) -> Self {
        let nodes: Vec<Node> = nodes.collect();
        let node_count = nodes.len();
        let edges = SparseMatrix::with_capacity(node_count, node_count);

        Self { nodes, edges }
    }
}

impl<Node, Weight, const Di: bool> Directed for AdjacencyMatrix<Node, Weight, Di> {
    fn directed(&self) -> bool {
        Di
    }
}

impl<Node, Weight, const Di: bool> Extend<Node, Weight> for AdjacencyMatrix<Node, Weight, Di> {
    fn extend_edges(&mut self, edges: impl Iterator<Item = (Self::EdgeId, Weight)>) {
        for (EdgeIndex { from, to }, weight) in edges {
            self.edges.insert(from.0, to.0, weight)
        }
    }

    fn extend_nodes(&mut self, nodes: impl Iterator<Item = Node>) {
        self.nodes.extend(nodes);
    }
}

impl<Node, Weight, const Di: bool> Get<Node, Weight> for AdjacencyMatrix<Node, Weight, Di> {
    fn node(&self, node_id: Self::NodeId) -> Option<&Node> {
        self.nodes.get(node_id.0)
    }
    fn weight(&self, edge_id: Self::EdgeId) -> Option<&Weight> {
        self.edges.get(edge_id.from.0, edge_id.to.0)
    }
}

impl<Node, Weight, const Di: bool> GetMut<Node, Weight> for AdjacencyMatrix<Node, Weight, Di> {
    fn node_mut(&mut self, node_id: Self::NodeId) -> Option<&mut Node> {
        self.nodes.get_mut(node_id.0)
    }
    fn weight_mut(&mut self, edge_id: Self::EdgeId) -> Option<&mut Weight> {
        self.edges.get_mut(edge_id.from.0, edge_id.to.0)
    }
}

impl<Node, Weight, const Di: bool> Insert<Node, Weight> for AdjacencyMatrix<Node, Weight, Di> {
    fn add_node(&mut self, node: Node) -> Self::NodeId {
        let node_id = NodeIndex(self.nodes.len());
        self.nodes.push(node);
        return node_id;
    }
    fn insert_edge(&mut self, edge_id: Self::EdgeId, weight: Weight) -> Option<Weight> {
        self.edges.insert(edge_id.from.0, edge_id.to.0, weight);
        None
    }
}

impl<Node, Weight, const Di: bool> Index for AdjacencyMatrix<Node, Weight, Di> {
    type EdgeIds<'a> = impl Iterator<Item = EdgeIndex> + 'a
    where Self: 'a;
    type NodeIds<'a> = impl Iterator<Item = NodeIndex> + 'a
    where Self: 'a;

    fn edge_ids<'a>(&'a self) -> Self::EdgeIds<'a> {
        self.edges
            .iter()
            .map(|(from, to, _)| EdgeIndex::new(NodeIndex(from), NodeIndex(to)))
    }

    fn node_ids<'a>(&'a self) -> Self::NodeIds<'a> {
        (0..self.nodes.len()).map(NodeIndex)
    }
}

impl<Node, Weight, const Di: bool> IndexAdjacent for AdjacencyMatrix<Node, Weight, Di> {
    type AdjacentEdgeIds<'a> = impl Iterator<Item = EdgeIndex> + 'a
    where Self: 'a;
    type AdjacentNodeIds<'a> = impl Iterator<Item = NodeIndex> + 'a
    where Self: 'a;

    fn adjacent_edge_ids<'a>(&'a self, node_id: Self::NodeId) -> Self::AdjacentEdgeIds<'a> {
        self.edges
            .row(node_id.0)
            .map(move |(to, _)| EdgeIndex::new(node_id, NodeIndex(to)))
    }
    fn adjacent_node_ids<'a>(&'a self, node_id: Self::NodeId) -> Self::AdjacentNodeIds<'a> {
        self.edges.row(node_id.0).map(|(to, _)| NodeIndex(to))
    }
}

impl<Node, Weight, const Di: bool> Iter<Node, Weight> for AdjacencyMatrix<Node, Weight, Di> {
    type Nodes<'a> = impl Iterator<Item = &'a Node> + 'a
    where
        Node: 'a,
        Self: 'a;

    type Edges<'a> = impl Iterator<Item = EdgeRef<'a, Self::EdgeId, Weight>> + 'a
    where
        Weight: 'a,
        Self: 'a;

    fn iter_nodes<'a>(&'a self) -> Self::Nodes<'a> {
        self.nodes.iter()
    }
    fn iter_edges<'a>(&'a self) -> Self::Edges<'a> {
        self.edges.iter().map(|(from, to, weight)| {
            EdgeRef::new(EdgeIndex::new(NodeIndex(from), NodeIndex(to)), weight)
        })
    }
}
impl<Node, Weight, const Di: bool> IterMut<Node, Weight> for AdjacencyMatrix<Node, Weight, Di> {
    type NodesMut<'a> = impl Iterator<Item = &'a mut Node> + 'a
    where
        Node: 'a,
        Self: 'a;

    type EdgesMut<'a> = impl Iterator<Item = EdgeRefMut<'a, Self::EdgeId, Weight>> + 'a
    where
        Weight: 'a,
        Self: 'a;

    fn iter_nodes_mut<'a>(&'a mut self) -> Self::NodesMut<'a> {
        self.nodes.iter_mut()
    }

    fn iter_edges_mut<'a>(&'a mut self) -> Self::EdgesMut<'a> {
        self.edges.iter_mut().map(|(from, to, weight)| {
            EdgeRefMut::new(EdgeIndex::new(NodeIndex(from), NodeIndex(to)), weight)
        })
    }
}

impl<Node, Weight, const Di: bool> IterAdjacent<Node, Weight>
    for AdjacencyMatrix<Node, Weight, Di>
{
    type Nodes<'a> = impl Iterator<Item = &'a Node> + 'a
    where
        Node: 'a,
        Self: 'a;

    type Edges<'a> = impl Iterator<Item = EdgeRef<'a, Self::EdgeId, Weight>> + 'a
    where
        Weight: 'a,
        Self: 'a;

    fn iter_adjacent_nodes<'a>(&'a self, node_id: Self::NodeId) -> Self::Nodes<'a> {
        self.adjacent_node_ids(node_id)
            .map(|node_id| self.node(node_id).unwrap())
    }

    fn iter_adjacent_edges<'a>(&'a self, node_id: Self::NodeId) -> Self::Edges<'a> {
        self.edges.row(node_id.0).map(move |(to, weight)| {
            let edge_id = EdgeIndex::new(node_id, NodeIndex(to));
            EdgeRef::new(edge_id, weight)
        })
    }
}
impl<Node, Weight, const Di: bool> IterAdjacentMut<Node, Weight>
    for AdjacencyMatrix<Node, Weight, Di>
{
    type NodesMut<'a> = impl Iterator<Item = &'a mut Node> + 'a
    where
        Node: 'a,
        Self: 'a;

    type EdgesMut<'a> = impl Iterator<Item = EdgeRefMut<'a, Self::EdgeId, Weight>> + 'a
    where
        Weight: 'a,
        Self: 'a;

    fn iter_adjacent_nodes_mut<'a>(&'a mut self, node_id: Self::NodeId) -> Self::NodesMut<'a> {
        let ids = self.adjacent_node_ids(node_id).collect::<Vec<_>>();
        self.nodes
            .iter_mut()
            .enumerate()
            .filter_map(move |(i, node)| {
                if ids.contains(&NodeIndex(i)) {
                    Some(node)
                } else {
                    None
                }
            })
    }

    fn iter_adjacent_edges_mut<'a>(&'a mut self, node_id: Self::NodeId) -> Self::EdgesMut<'a> {
        self.edges.row_mut(node_id.0).map(move |(to, weight)| {
            let edge_id = EdgeIndex::new(node_id, NodeIndex(to));
            EdgeRefMut::new(edge_id, weight)
        })
    }
}

impl<Node, Weight, const Di: bool> Remove<Node, Weight> for AdjacencyMatrix<Node, Weight, Di> {
    fn remove_node(&mut self, node_id: Self::NodeId) -> Option<Node> {
        todo!()
    }

    fn remove_edge(&mut self, edge_id: Self::EdgeId) -> Option<Weight> {
        todo!()
    }
}

impl<Node, Weight, const Di: bool> Reserve for AdjacencyMatrix<Node, Weight, Di> {
    fn reserve_edges(&mut self, additional: usize) {
        todo!()
    }

    fn reserve_nodes(&mut self, additional: usize) {
        self.nodes.reserve(additional)
    }
}

impl<Node: crate::graph::Node, Weight: crate::graph::Weight, const Di: bool> Graph<Node, Weight>
    for AdjacencyMatrix<Node, Weight, Di>
{
}

impl<Node: crate::graph::Node, const Di: bool> WeightlessGraph<Node>
    for AdjacencyMatrix<Node, (), Di>
{
}
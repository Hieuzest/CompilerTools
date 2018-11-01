use std::fmt;

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
struct Graph<V, E> {
	vertices: Vec<Vertex<V>>,
	edges: Vec<Edge<E>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Vertex<T> {
	pub in_edges: Vec<usize>,
	pub out_edges: Vec<usize>,
	pub data: T,
}

impl<T> Vertex<T> {
	pub fn new(data: T) -> Self {
		Vertex::<T> {
			in_edges: Vec::default(),
			out_edges: Vec::default(),
			data: data
		}
	}
}

impl<T: fmt::Display> fmt::Display for Vertex<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "Vertex[In: {:?}, Out: {:?}, Data: {:}] ", self.in_edges, self.out_edges, self.data)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Edge<T> {
	pub in_vertex: usize,
	pub out_vertex: usize,
	pub cost: T,
}

impl<T> Edge<T> {
	pub fn new(in_: usize, out_: usize, cost: T) -> Self {
		Edge::<T> {
			in_vertex: in_,
			out_vertex: out_,
			cost: cost
		}
	}
}

impl<T: fmt::Display> fmt::Display for Edge<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "Edge[In: {:?}, Out: {:?}, Cost: {:}] ", self.in_vertex, self.out_vertex, self.cost)
    }
}
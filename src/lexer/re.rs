use super::graph::*;
use std::collections::HashMap;
use std::fmt;

pub type SingleToken = char;
pub const EPSILON_SINGLETOKEN: SingleToken = '\0';

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum RegularExpression {
	Epsilon,
	Atomic {
		id: SingleToken,
	},
	Union {
		operands: Vec<RegularExpression>,
	},
	Concatenation {
		operands: Vec<RegularExpression>,		
	},
	Iteration {
		operand: Box<RegularExpression>,
	},
	Alias {
		id: String
	},
	Match {
		operand: Box<RegularExpression>,
	}
}

impl Default for RegularExpression {
	fn default() -> Self {
		RegularExpression::Epsilon
	}
}

impl RegularExpression {
	
	pub fn parse<'a, M: Into<Option<&'a HashMap<String, RegularExpression>>>>(src: &str, definitions: M) -> Result<RegularExpression, ()> {
		let mut stack: Vec<RegularExpressionNode> = Vec::new();
		let definitions = definitions.into();
		let mut buffer = String::new();
		let mut flag_escape = false;
		let mut flag_group = false;
		let mut flag_range = false;
		let mut flag_alias = false;

		let collect_union = |stack: &mut Vec<RegularExpressionNode>| {
			//collect union
			// println!("{:?}", stack);
			let mut rset = Vec::new();
			let mut rnegset = Vec::new();
			while let Some(node) = stack.pop() {
				match node {
					RegularExpressionNode::Expression(e) => rset.push(e),
					RegularExpressionNode::UnionOp => (),
					RegularExpressionNode::GroupNegate => {
						rnegset = rset.clone();
						rset.clear();
					},
					RegularExpressionNode::Tuple | RegularExpressionNode::Group => break,
					_ => ()
				}
			}
			rset = rset.into_iter().filter(|x| !rnegset.contains(x)).collect();
			if rset.len() == 1 {
				stack.push(RegularExpressionNode::Expression(rset.pop().unwrap()));
			} else if rset.len() > 1 {
				rset.reverse();
				let re = RegularExpression::Union {	operands: rset };
				stack.push(RegularExpressionNode::Expression(re));
			} else { panic!("Empty union") }
		};

		let collect_concat = |stack: &mut Vec<RegularExpressionNode>| {
			//collect concat
			// println!("{:?}", stack);
			let mut rset = Vec::new();
			while let Some(node) = stack.pop() {
				match node {
					RegularExpressionNode::Expression(e) => rset.push(e),
					RegularExpressionNode::Tuple | RegularExpressionNode::UnionOp => {
						stack.push(node);
						break;
					},
					_ => ()
				}
			}
			if rset.len() == 1 {
				stack.push(RegularExpressionNode::Expression(rset.pop().unwrap()));
			} else if rset.len() > 1 {
				rset.reverse();
				let re = RegularExpression::Concatenation {	operands: rset };
				stack.push(RegularExpressionNode::Expression(re));
			} else { panic!("Empty concat") }
		};

		for token in src.chars() {
			match token {
				'0' if flag_escape => {
					stack.push(RegularExpressionNode::Expression(RegularExpression::Epsilon));
					flag_escape = false;
				},
				_ if flag_escape => {
					stack.push(RegularExpressionNode::Expression(RegularExpression::Atomic{ id: match token {
						't' => '\t',
						'n' => '\n',
						'r' => '\r',
						_ => token
					} }));
					flag_escape = false;
				},
				_ if flag_range => {
					if let Some(RegularExpressionNode::Expression(RegularExpression::Atomic{ id: start })) = stack.pop() {
						let mut curr = start;
						while curr <= token {
							stack.push(RegularExpressionNode::Expression(RegularExpression::Atomic{ id: curr }));
							curr =(curr as u8 + 1) as char;
						}
					}
					// stack.push(RegularExpressionNode::Expression(RegularExpression::Atomic{ id: token }));
					// collect_union()
					flag_range = false;
					// Range
				},
				'}' if flag_alias => {
					// stack.push(RegularExpressionNode::Expression(RegularExpression::Alias{ id: buffer.clone() }));
					stack.push(RegularExpressionNode::Expression(definitions.expect("Expected alias table").get(&buffer).expect(&format!("Expected alias [{:}]", buffer)).clone()));
					buffer.clear();
					flag_alias = false;
				},
				_ if flag_alias => {
					buffer.push(token);
				}
				' ' | '\n' | '\t' | '\r' => (),
				'*' => {
					let re = stack.pop();
					if let Some(RegularExpressionNode::Expression(e)) = re {
						stack.push(RegularExpressionNode::Expression(RegularExpression::Iteration{ operand: Box::new(e) }));
					} else { panic!("Iteration after operation") }
				},
				'|' | '+' => {
					collect_concat(&mut stack);
					stack.push(RegularExpressionNode::UnionOp);
				},
				'(' => {
					// collect_concat(&mut stack);
					stack.push(RegularExpressionNode::Tuple);
				},
				')' => {
					collect_concat(&mut stack);
					collect_union(&mut stack);
				},
				'[' => {
					if flag_group { return Err(()); }
					stack.push(RegularExpressionNode::Group);
					flag_group = true;
				},
				']' => {
					collect_union(&mut stack);
					flag_group = false;
				},
				'{' => {
					flag_alias = true;
				},
				'-' if flag_group => {
					flag_range = true;
				},
				'^' if flag_group => {
					stack.push(RegularExpressionNode::GroupNegate);
				},
				'\\' => {
					flag_escape = true;
				},
				_ => {
					stack.push(RegularExpressionNode::Expression(RegularExpression::Atomic{ id: token }));
				}
			}
		}

		collect_concat(&mut stack);
		collect_union(&mut stack);

		if stack.len() == 1 {
			if let Some(RegularExpressionNode::Expression(e)) = stack.pop() {
				return Ok(e)
			}
		}
		Err(())
	}


	pub fn apply_alias(&mut self, alias: &HashMap<String, RegularExpression>) {
		let aliasid;
		match self {
			RegularExpression::Union { ref mut operands } => {
				for ref mut operand in operands {
					operand.apply_alias(alias);
				}
				return;
			},
			RegularExpression::Concatenation { ref mut operands } => {
				for ref mut operand in operands {
					operand.apply_alias(alias);
				}
				return;
			},
			RegularExpression::Iteration { ref mut operand } => {
				operand.apply_alias(alias);
				return;
			},
			RegularExpression::Match { ref mut operand } => {
				operand.apply_alias(alias);
				return;
			},
			RegularExpression::Alias { ref id } => {
				aliasid = id.clone();
			},
			_ => return
		}
		*self = alias.get(&aliasid).expect("No alias").clone();
	}
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum RegularExpressionNode {
	Expression(RegularExpression),
	Tuple,
	Group,
	GroupNegate,
	UnionOp,
	RangeOp,
}


#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct StateTransferGraph<D: Default = (), T: PartialEq = SingleToken> {
	pub vertices: Vec<Vertex<D>>,
	pub edges: Vec<Edge<T>>,
	pub start: usize,
	pub end: usize,
	pub ends: Vec<usize>,
}


impl<D: Default, T: PartialEq> StateTransferGraph<D, T> {
	pub fn new() -> Self {
		StateTransferGraph {
			vertices: Vec::default(),
			edges: Vec::default(),
			start: 0,
			end: 0,
			ends: Vec::default(),
		}
	}

	pub fn add_state(&mut self) -> usize{
		self.vertices.push(Vertex::new(D::default()));
		self.vertices.len() - 1
	}

	pub fn add_state_with_data(&mut self, data: D) -> usize{
		self.vertices.push(Vertex::new(data));
		self.vertices.len() - 1
	}

	pub fn add_state_after(&mut self, prev: usize, cost: T) -> usize {
		let state = self.add_state();
		self.add_transfer(prev, state, cost);
		state
	}

	pub fn add_transfer(&mut self, in_: usize, out_: usize, cost: T) -> usize {
		self.edges.push(Edge::new(in_, out_, cost));
		let edge = self.edges.len() - 1;
		self.vertices[in_].out_edges.push(edge);
		self.vertices[out_].in_edges.push(edge);
		edge
	}

	pub fn mark_as_start(&mut self, state: usize) {
		self.start = state;
	}

	pub fn mark_as_end(&mut self, state: usize) {
		self.end = state;
		if !self.ends.contains(&state) {
			self.ends.push(state)
		}
	}

	pub fn extend(&mut self, cross_state: usize, rhs: Self) -> usize {
		let mut map = Vec::new();
		for (i, _) in rhs.vertices.iter().enumerate() {
			map.push(if i == rhs.start { cross_state } else { self.add_state() });
		}
		for e in rhs.edges {
			self.add_transfer(map[e.in_vertex], map[e.out_vertex], e.cost);
		}
		map[rhs.end]
	}

	pub fn get_transition(&self, state: usize, token: T) -> Option<usize> {
		for i in &self.vertices[state].out_edges {
			if self.edges[*i].cost == token {
				return Some(self.edges[*i].out_vertex);
			}
		}
		None
	}
}


impl<D: Default + fmt::Display, T: Clone + PartialEq + fmt::Display> fmt::Display for StateTransferGraph<D, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		writeln!(f, "Vertices: \n\t{:}", self.vertices.iter().enumerate().map(|(i, x)| format!("{:} : {:?}\n{:}", i, x.out_edges.iter().map(|j| self.edges[*j].to_string()).collect::<Vec<String>>(), x)).collect::<Vec<String>>().join("\n\t"));
		writeln!(f, "Edges: \n\t{:}", self.edges.iter().enumerate().map(|(i, x)| format!("{:} : {:}", i, x)).collect::<Vec<String>>().join("\n\t"));
        writeln!(f, "start: {:?}", self.start);
        writeln!(f, "ends: {:?}", self.ends)
    }
}
use super::re::*;
use super::re::EPSILON_SINGLETOKEN;
use super::utils::*;

pub fn construct_nfa(re: &RegularExpression) -> StateTransferGraph {
	match re {
		&RegularExpression::Union {
			ref operands
		} => {
			let mut graph = StateTransferGraph::new();
			let state_start = graph.add_state();
			graph.mark_as_start(state_start);
			let mut states = Vec::new();
			for ref o in operands {
				let state_mid = graph.add_state_after(state_start, EPSILON_SINGLETOKEN);
				states.push(graph.extend(state_mid, construct_nfa(o)));
			}
			let state_end = graph.add_state();
			for s in states {
				graph.add_transfer(s, state_end, EPSILON_SINGLETOKEN);
			}
			graph.mark_as_end(state_end);
			graph
		},
		&RegularExpression::Concatenation {
			ref operands
		} => {
			let mut graph = StateTransferGraph::new();
			let mut state = graph.add_state();
			graph.mark_as_start(state);
			for ref o in operands {
				state = graph.extend(state, construct_nfa(o));
			}
			graph.mark_as_end(state);
			graph
		},
		&RegularExpression::Iteration {
			ref operand
		} => {
			let mut graph = StateTransferGraph::new();
			let state_start = graph.add_state();
			graph.mark_as_start(state_start);
			let state_start_inner = graph.add_state_after(state_start, EPSILON_SINGLETOKEN);
			let state_end_inner = graph.extend(state_start_inner, construct_nfa(operand));
			let state_end = graph.add_state_after(state_end_inner, EPSILON_SINGLETOKEN);
			graph.add_transfer(state_end_inner, state_start_inner, EPSILON_SINGLETOKEN);
			graph.add_transfer(state_start, state_end, EPSILON_SINGLETOKEN);
			graph.mark_as_end(state_end);
			graph
		},
		&RegularExpression::Epsilon => {
			let mut graph = StateTransferGraph::new();
			let state = graph.add_state();
			graph.mark_as_start(state);
			graph.mark_as_end(state);
			graph
		},
		&RegularExpression::Atomic {
			ref id
		} => {
			let mut graph = StateTransferGraph::new();
			let state = graph.add_state();
			graph.mark_as_start(state);
			let state = graph.add_state_after(state, id.clone());
			graph.mark_as_end(state);
			graph
		},
		_ => {
			panic!()
		}
	}
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum MatchResult {
	Ok,
	Unfinished, Err,
}

pub fn match_nfa<D: Default, T: PartialEq + Default + Clone, I: IntoIterator<Item=T>>(graph: &StateTransferGraph<D, T>, token: I) -> MatchResult {

	let epsilon_closure = |s: &VertexSet| -> VertexSet {
		let mut ret = s.clone();
		let mut flag = true;

		while flag {
			flag = false;
			// println!("loop here");
			for i in 0..s.len() {
				if !ret[i] { continue; }
				for e in &graph.vertices[i].out_edges {
					if graph.edges[*e].cost == T::default() {
						let v = graph.edges[*e].out_vertex;
						if !ret[v] {
							ret[v] = true;
							flag = true;
						}
					}
				}
			}
		}
		ret
	};


	let move_step = |s: &VertexSet, token: &T| -> VertexSet {
		let mut ret = VertexSet::with_capacity(s.len());
		ret.resize(s.len(), false);
		for i in 0..s.len() {
			if !s[i] { continue; }
			for e in &graph.vertices[i].out_edges {
				if &graph.edges[*e].cost == token {
					let v = graph.edges[*e].out_vertex;
					ret[v] = true;
				}		
			}
		}
		ret
	};

	let mut state = VertexSet::with_capacity(graph.vertices.len());
	state.resize(graph.vertices.len(), false);
	state[graph.start] = true;
	let mut state = epsilon_closure(&state);

	for c in token {
		state = move_step(&state, &c);
		if state.iter().all(|x| !x) {
			return MatchResult::Err;
		}
	}
	if state.iter().enumerate().filter(|(_, x)| **x).any(|(i, _)| graph.ends.contains(&i)) { MatchResult::Ok } else { MatchResult::Unfinished }
}
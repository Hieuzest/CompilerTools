use super::re::*;
use std::collections::HashMap;
use crate::utils::*;

trait FollowPosImpl {
	fn downward(&self, followpos: &mut Vec<Vec<bool>>, posmap: &mut Vec<SingleToken>) -> (bool, VertexSet, VertexSet);
}

impl FollowPosImpl for RegularExpression {
	fn downward(&self, followpos: &mut Vec<Vec<bool>>, posmap: &mut Vec<SingleToken>) -> (bool, VertexSet, VertexSet) {
		match self {
			RegularExpression::Epsilon => {
				// push_sets!(followpos);
				// posmap.push(EPSILON_SINGLETOKEN);
				(true, new_set!(followpos.len()), new_set!(followpos.len()))
			},
			RegularExpression::Atomic { id } => {
				let mut first = new_set!(followpos.len());
				first.push(true);
				let mut last = new_set!(followpos.len());
				last.push(true);
				push_sets!(followpos);
				posmap.push(*id);
				(false, first, last)                
			},
			RegularExpression::Union { ref operands } => {
				let mut arr = Vec::<(bool, VertexSet, VertexSet)>::new();
				for child in operands {
					arr.push(child.downward(followpos, posmap));
				}

				let mut nullable = false;
				let mut first = new_set!(followpos.len());
				let mut last = new_set!(followpos.len());

				for (cn, cf, cl) in &arr {
					nullable = nullable || *cn;
					cup_set!(first, cf);
					cup_set!(last, cl);
				}
				(nullable, first, last)    
			},
			RegularExpression::Concatenation { ref operands } => {
				let mut arr = Vec::<(bool, VertexSet, VertexSet)>::new();
				for child in operands {
					arr.push(child.downward(followpos, posmap));
				}

				let mut nullable = true;
				let mut first = new_set!(followpos.len());
				let mut last = new_set!(followpos.len());

				// evaluate followpos
				for i in 0..arr.len() - 1 {
					// for j in arr[i].2.iter().enumerate().filter_map(|(x, y)| if *y {Some(x)} else {None}) {
					for j in 0..arr[i].2.len() {
						if !arr[i].2[j] { continue; }
						let mut flag_null = true;
						for k in i+1..arr.len() {
							if !flag_null { break; }
							for l in 0..arr[k].1.len() {
								if !arr[k].1[l] { continue; }
								followpos[j][l] = true;
							}
							flag_null = arr[k].0;
						}
					}
				}

				for (cn, cf, _) in &arr {
					// println!("--- {:?}  {:?} {:}", self, first, cn);
					if nullable { cup_set!(first, cf); }
					nullable = nullable & cn;
					if !cn { break; }
				}
				// println!("--- {:?}  {:?}", self, first);

				arr.reverse();
				let mut last_null = true;
				for (cn, _, cl) in &arr {
					if last_null { cup_set!(last, cl); }
					last_null = *cn;
					if !cn { break; }
				}

				(nullable, first, last)    
			},
			RegularExpression::Iteration { ref operand } => {
				let (_, cf, cl) = operand.downward(followpos, posmap);
				
				// evaluate followpos
				for i in 0..cl.len() {
					if !cl[i] { continue; }
					for j in 0..cf.len() {
						if !cf[j] { continue; }
						followpos[i][j] = true;
					}
				}

				(true, cf, cl)
			},
			_ => panic!()
		}
	}
}



pub fn construct_dfa(re: &RegularExpression, charmap: &[SingleToken]) -> StateTransferGraph {
	let mut graph = StateTransferGraph::new();
	let mut followpos = Vec::<Vec<bool>>::new();
	let mut posmap = Vec::<SingleToken>::new();
	let re = RegularExpression::Concatenation { operands: vec![re.clone(), RegularExpression::Atomic { id: '$' } ] };
	// push_sets!(followpos);
	let (_, first, _) = re.downward(&mut followpos, &mut posmap);
	// println!("{:?}", followpos);

	let mut states: HashMap<VertexSet, usize> = HashMap::new();	
	let mut stack: Vec<VertexSet> = Vec::new();

	let state_start = graph.add_state();
	graph.mark_as_start(state_start);
	if first[followpos.len() - 1] { graph.mark_as_end(state_start); }
	states.insert(first.clone(), state_start);
	stack.push(first);

	while let Some(ref s) = stack.pop() {
		let state = *states.get(s).unwrap();

		for token in charmap {
			let mut ahead = new_set!(followpos.len());
			for i in 0..s.len() {
				if !s[i] || &posmap[i] != token { continue; }
				cup_set!(ahead, followpos[i]);
			}

			if !ahead.contains(&true) { continue; }

			let state_option = states.remove(&ahead);
			let state_rhs = if let Some(ind) = state_option {
				states.insert(ahead.clone(), ind);
				ind 
			} else {
				let ind = graph.add_state();
				states.insert(ahead.clone(), ind);
				if ahead[followpos.len() - 1] { graph.mark_as_end(ind) }
				stack.push(ahead);
				ind
			};
			graph.add_transfer(state, state_rhs, *token);
		}
	}
	graph
}


pub fn nfa_to_dfa<D: Default, T: PartialEq + Default + Clone>(nfa_graph: &StateTransferGraph<D, T>, charmap: &[T]) -> StateTransferGraph<D, T> {
	// let mut map: HashMap<VertexSet, bool> = HashMap::new();
	let mut states: HashMap<VertexSet, usize> = HashMap::new();	
	let mut stack: Vec<VertexSet> = Vec::new();
	let mut graph = StateTransferGraph::new();
	// let charmap = vec!['a', 'b'];

	let epsilon_closure = |s: &VertexSet| -> VertexSet {
		let mut ret = s.clone();
		let mut flag = true;

		while flag {
			flag = false;
			// println!("loop here");
			for i in 0..s.len() {
				if !ret[i] { continue; }
				for e in &nfa_graph.vertices[i].out_edges {
					if nfa_graph.edges[*e].cost == T::default() {
						let v = nfa_graph.edges[*e].out_vertex;
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
			for e in &nfa_graph.vertices[i].out_edges {
				if &nfa_graph.edges[*e].cost == token {
					let v = nfa_graph.edges[*e].out_vertex;
					ret[v] = true;
				}		
			}
		}
		ret
	};

	let mut start = Vec::with_capacity(nfa_graph.vertices.len());
	start.resize(nfa_graph.vertices.len(), false);
	start[nfa_graph.start] = true;
	let start = epsilon_closure(&start);
	let state_start = graph.add_state();

	graph.mark_as_start(state_start);
	states.insert(start.clone(), state_start);
	if start[nfa_graph.end] { graph.mark_as_end(state_start) }
	stack.push(start);

	while let Some(ref s) = stack.pop() {
		// if map.contains_key(s) { continue; }

		let state = *states.get(s).unwrap();
		// states.insert(s.clone(), state);
		for token in charmap {
			let ahead = move_step(s, token);
			let ahead = epsilon_closure(&ahead);

			// println!("ahead {:?} {:?} \n {:?} \n ", token, s, ahead);

			if !ahead.contains(&true) { continue; }

			let state_option = states.remove(&ahead);
			let state_rhs = if let Some(ind) = state_option {
				states.insert(ahead.clone(), ind);
				ind 
			} else {
				let ind = graph.add_state();
				states.insert(ahead.clone(), ind);
				if ahead[nfa_graph.end] { graph.mark_as_end(ind) }
				stack.push(ahead);
				ind
			};
			graph.add_transfer(state, state_rhs, token.clone());
		}
	}

	graph
}



pub fn minimize_dfa<D: Default, T: PartialEq + Default + Clone>(ori: &StateTransferGraph<D, T>, charmap: &[T]) -> StateTransferGraph<D, T> {
	let mut graph = StateTransferGraph::new();
	let mut group = Vec::<usize>::new();
	let mut groups: usize = 1;

	group.resize(ori.vertices.len(), 0);
	for end in &ori.ends {
		group[*end] = 1;
	}
	// println!("group: {:?}", group);
	let mut i = 0;
	while i <= groups {
		for token in charmap {
			let mut set = Vec::new();
			for j in 0..group.len() {
				if group[j] != i { continue; }
				set.push(ori.get_transition(j, token.clone()));
			}
			// println!("[{:} -> {:}] : {:?}", i, token, set);
			let mut k = 0;
			let new_group = groups + 1;
			for j in 0..group.len() {
				if group[j] != i { continue; }
				if set[k] != set[0] {
					groups = new_group;
					group[j] = groups;	
				}
				k += 1;
			}
		}
		i += 1;
	}
	// println!("GROUP: {:?}", group);
	for _ in 0..groups+1 {
		graph.add_state();
	}
	for i in 0..groups+1 {
		let states: Vec<usize> = group.iter().enumerate().filter_map(move |(state, g)| if *g==i { Some(state) } else { None }).collect();
		for token in charmap {
			if group[ori.start] == i { graph.mark_as_start(i); }
			if ori.ends.iter().any(|x| group[*x] == i) { graph.mark_as_end(i); }
			for state in &states {
				let trans = ori.get_transition(*state, token.clone());
				if let Some(rhs) = trans {
					graph.add_transfer(i, group[rhs], token.clone());
					break;
				}
			}
		}
	}
	graph
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum MatchResult {
	Ok, Unfinished, Err,
}

pub fn match_dfa<D: Default, T: PartialEq, I: IntoIterator<Item=T>>(graph: &StateTransferGraph<D, T>, token: I) -> MatchResult {
	let mut state = graph.start;
	for c in token {
		let mut flag = false;
		for e in &graph.vertices[state].out_edges {
			if graph.edges[*e].cost == c {
				state = graph.edges[*e].out_vertex;
				flag = true;
				break;
			}
		}
		if !flag { return MatchResult::Err; }
	}
	if graph.ends.contains(&state) { MatchResult::Ok } else { MatchResult::Unfinished }
}
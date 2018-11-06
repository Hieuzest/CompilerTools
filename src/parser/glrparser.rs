// use super::grammar::{Grammar, Production, Expression, Term, Associativity};
// use super::{NodeType, Node, NonTerminal};
use super::prelude::*;
use super::derivations;
use crate::lexer::re::StateTransferGraph;

use std::collections::HashMap;
use std::collections::HashSet;

use super::lrparser::{LRItem, LRItems, LRAction};


#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    pub msg: String,
    pub index: usize,
}


pub fn parse_with_graph(src: &[Token], graph: &StateTransferGraph<LRItems, Term>) -> Result<Node, ParseError> {
    if DEBUG!() && VERBOSE!() { println!("{:}", graph); }
    
    #[derive(Debug, Clone, PartialEq)]
    enum StackItem {
        State(usize), NonTerminal(String), Terminal(Token)
    }

    type MultiStack = Vec<(Vec<StackItem>, Vec<Production>)>;

    let mut last_stack = MultiStack::new();
    last_stack.push((vec![StackItem::State(graph.start)], Vec::new()));

    let mut next = 0;
    'outer: loop {
        let token = if next < src.len() { src[next].clone() } else { Token {
            type_ : FINISH_TOKEN.to_string(),
            value_: FINISH_TOKEN.to_string(),
            line_: 0,
        } };
        
        let mut curr_stack = MultiStack::new();

        // add all shift
        for (stack, derivations) in &last_stack {
            let curr_state = if let Some(&StackItem::State(s)) = stack.last() { s } else { panic!("State not on top of stack !") };
            // println!("Curr_state: {:?}, term: {:}, tr: {:?}", stack, token, graph.get_transition(curr_state, Term::from(&token)));

            if let Some(next_state) = graph.get_transition(curr_state, Term::from(&token)).or(graph.get_transition(curr_state, Term::terminal(token.type_.clone()))) {
                let mut stack = stack.clone();
                stack.push(StackItem::Terminal(token.clone()));
                stack.push(StackItem::State(next_state));
                curr_stack.push((stack, derivations.clone()));
            }
        }


        // handle all reduce

        if next < src.len() { next += 1; }
        let token = if next < src.len() { src[next].clone() } else { Token {
            type_ : FINISH_TOKEN.to_string(),
            value_: FINISH_TOKEN.to_string(),
            line_: 0,
        } };

        if DEBUG!() { 
            if VERBOSE!() {
                println!("#{:} States: {:?}, token: {:}", next - 1, curr_stack, token);
            } else {
                println!("#{:} State sum: {:?}, token: {:}", next - 1, last_stack.len(), token);                
            }
        }

        let mut state = 0;
        while state < curr_stack.len() {
            let (stack, derivations) = curr_stack.get(state).unwrap().clone();
            // if DEBUG!() && VERBOSE!() { println!("Search for reduced: States: {:?}, token: {:}", stack, token); }

            let curr_state = if let Some(&StackItem::State(s)) = stack.last() { s } else { panic!("State not on top of stack !") };

            for (item, ahead) in &*graph.vertices[curr_state].data {
                if item.pos == item.rule.expr.terms.len() {
                    // Check lookahead
                    // println!("Item: {:}, ahead: {:?}, token: {:}", item, ahead, token);
                    if ahead.len() > 0 && !ahead.contains(&Term::from(&token)) && !ahead.contains(&Term::terminal(token.type_.clone()))  { continue; }
        

                    let mut stack = stack.clone();
                    let mut derivations = derivations.clone();

                    for _ in 0..item.pos {
                        stack.pop();
                        stack.pop();
                    }

                    let curr_state = if let Some(&StackItem::State(s)) = stack.last() { s } else { panic!("Reduce: State not on top of stack !") };
                    stack.push(StackItem::NonTerminal(item.rule.name.clone()));


                    // if DEBUG!() && VERBOSE!() { println!("Reduced: Stack: {:?}", stack); }
                    if curr_state == graph.start && item.rule.name == FINISH_TOKEN.to_string() {
                        // Accept
                        // Assume the grammar is unambigious, so exit

                        last_stack = curr_stack;
                        break 'outer;
                    }
                    if let Some(next_state) = graph.get_transition(curr_state, Term::nonterminal(item.rule.name.as_str())) {
                        // goto
                        stack.push(StackItem::State(next_state));
                        // store derivation rule for this reduce
                        derivations.push(item.rule.clone());

                        if !curr_stack.iter().any(|(s, d)| s == &stack) { curr_stack.push((stack, derivations)); }
                    } else {
                        // Drop this
                        if DEBUG!() { println!("Dropped: {:}, {:}", curr_state, item.rule); }
                    }
                }
            }
            state += 1;
        }
        last_stack = curr_stack;
        // println!("next: {:}", next);
        if last_stack.is_empty() {
            return Err(ParseError {
                msg: format!("All cases failed."),
                index: next
            });
        }
    }
    let (_, mut derivations) = last_stack.pop().unwrap();
    derivations.reverse();
    if DEBUG!() { println!("Result : \n{:}", derivations.iter().map(|x| x.dump()).collect::<Vec<String>>().join("\n")); }
    Ok(derivations::build_rightmost(&derivations, src))
}
use super::prelude::*;

use std::collections::HashMap;


pub type LLTable = HashMap<String, HashMap<Term, Production>>;

#[derive(Debug, Clone, Eq, PartialEq, Default)]
struct TerminalSet {
    items: Vec<Term>
}

impl TerminalSet {
    fn len(&self) -> usize {
        self.items.len()
    }

    fn append(&mut self, rhs: &Self) -> bool {
        let mut ret = false;
        for item in &rhs.items {
            if !self.items.contains(item) { 
                self.items.push(item.clone());
                ret = true;
            }
        }
        ret
    }

    fn push(&mut self, rhs: Term) -> bool {
        if !self.items.contains(&rhs) {
            self.items.push(rhs);
            true
        } else {
            false
        }
    }

    fn append_epsilon(&mut self) -> bool {
        self.push(Term::terminal(EPSILON_TOKEN.to_string()))
    }
    
    fn remove_epsilon(&mut self) -> bool {
        if let Some(pos) = self.items.iter().position(|x| x == &Term::terminal(EPSILON_TOKEN.to_string())) {
            self.items.remove(pos);
            true
        } else {
            false
        }
    }

    fn contains_epsilon(&self) -> bool {
        self.items.contains(&Term::terminal(EPSILON_TOKEN.to_string()))
    }
}



pub fn generate_table(grammar: &Grammar) -> LLTable {
    let mut first: HashMap<Term, TerminalSet> = HashMap::new();
    let mut follow: HashMap<String, TerminalSet> = HashMap::new();

    let get_first = |terms: &[Term], first: &HashMap<Term, TerminalSet>| {
        let mut tset = TerminalSet::default();
        let mut flag = true;
        for term in terms {
            flag = false;
            let t = first.get(term).expect("Term not found");
            tset.append(t);
            tset.remove_epsilon();
            if !t.contains_epsilon() { break; }
            flag = true;
        }
        if flag { tset.append_epsilon(); }
        tset
    };

    // Compute first
    // println!("ts:{:?}", grammar.terminals());
    for terminal in grammar.terminals() {
        let mut tset = TerminalSet::default();
        tset.push(terminal.clone());
        first.insert(terminal, tset);
    }

    // println!("nts:{:?}", grammar.non_terminals());
    for nonterminal in grammar.non_terminals() {
        first.insert(Term::nonterminal(nonterminal), TerminalSet::default());
    }

    // Loop
    loop {
        let mut flag_move = false;
        for p in &grammar.productions {
            let tset = get_first(&p.expr.terms[..], &first);
            flag_move |= first.get_mut(&Term::nonterminal(p.name.as_str())).expect("2").append(&tset);
        }
        if !flag_move { break; }
    }

    // Compute follow
    for nonterminal in grammar.non_terminals() {
        follow.insert(nonterminal, TerminalSet::default());
    }
    follow.get_mut(&grammar.start_symbol).expect("3").push(Term::terminal(FINISH_TOKEN.to_string()));

    loop {
        let mut flag_move = false;
        for p in &grammar.productions {
            for i in 0..p.expr.terms.len() {
                if let Term::NonTerminal { name: t, .. } = &p.expr.terms[i] {
                    let mut tset = get_first(&p.expr.terms[i+1..p.expr.terms.len()], &first);
                    let fep = tset.contains_epsilon();
                    tset.remove_epsilon();
                    flag_move |= follow.get_mut(t).expect("4").append(&tset);
                    if fep {
                        let fx = follow.get_mut(&p.name).expect("5").clone();
                        flag_move |= follow.get_mut(t).expect("6").append(&fx);                    
                    }
                }
            }
        }
        if !flag_move { break; }
    }

    if DEBUG!() { println!("first:{:?}\nfollow:{:?}", first, follow); }

    let mut table = LLTable::default();
    for nonterminal in grammar.non_terminals() {
        table.insert(nonterminal, HashMap::default());
    }
    for p in &grammar.productions {
        for f in get_first(&p.expr.terms[..], &first).items {
            if f == Term::terminal(EPSILON_TOKEN.to_string()) {
                for f in &follow.get_mut(&p.name).unwrap().items {
                    if let Some(_) = table.get_mut(&p.name).unwrap().insert(f.clone(), p.clone()) {
                        if DEBUG!() {
                            println!("DETECTED CONFLICT: {:}, {:}", p.name, f);
                        }
                    }
                }
            } else {
                if let Some(_) = table.get_mut(&p.name).unwrap().insert(f.clone(), p.clone()) {
                    if DEBUG!() {
                        println!("DETECTED CONFLICT: {:}, {:}", p.name, f);
                    }     
                }
            }
        }
    }

    if DEBUG!() {
        for (t, m) in &table {
            println!("TABLE for {:}", t);
            for (n, p) in m {
                println!("\t{:}:\t\t{:}", n, p.dump());
            }
        }
    }
    table
}


#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    pub msg: String,
    pub index: usize,
}


pub fn parse(src: &[Token], grammar: &Grammar) -> Result<Node, Vec<ParseError>> {
    let table = generate_table(&grammar);
    let mut tree = Node {
        value: NodeType::InnerNode,
        childs: Vec::new(),
        index: 0
    }.zipper();
    let mut stack = Vec::new();
    let mut next = 0;
    let mut errs = Vec::new();
    // let mut errstate = false;

    stack.push(Term::Terminal{ type_: FINISH_TOKEN.to_string(), value: None });
    stack.push(Term::NonTerminal{ name: grammar.start_symbol.clone(), unwrap: false });
    while let Some(term) = stack.pop() {
        if DEBUG!() { println!("{:?}\n {:?} : {:?}", stack, term, if next == src.len() { FINISH_TOKEN.to_string() } else { format!("{:?}", src[next]) }); }
        match term {
            Term::NonTerminal { name, .. } => {
                if let Some(p) = table.get(&name).unwrap().get(&if next == src.len() { Term::terminal(FINISH_TOKEN) } else { Term::from(&src[next]) })
                    .or(table.get(&name).unwrap().get(&if next == src.len() { Term::terminal(FINISH_TOKEN) } else { Term::terminal(src[next].type_.as_str()) })) {
                    let node = Node {
                        value: NodeType::NonTerminal(NonTerminal {
                            type_: name.clone(),
                            value_: p.label.clone(),
                            rule_: p.clone(),
                        }),
                        childs: Vec::new(),
                        index: next,
                    };

                    tree.node.push(node);
                    let pos = tree.node.len() - 1;
                    tree = tree.child(pos);

                    if p.expr.terms.len() == 0 {
                        let mut f = if let NodeType::NonTerminal(NonTerminal { rule_, .. }) = &tree.node.value {
                            if tree.node.len() == rule_.expr.terms.len() { true } else { false }
                        } else { false };
                        while f {
                            tree = tree.parent();
                            f = if let NodeType::NonTerminal(NonTerminal { rule_, .. }) = &tree.node.value {
                                if tree.node.len() == rule_.expr.terms.len() { true } else { false }
                            } else { false };
                        }
                    }

                    stack.extend(p.expr.terms.clone().into_iter().rev());
                } else {
                    errs.push(ParseError{
                        msg: format!("No rule found for {:} : {:}, Expected {:?}", name, if next == src.len() { FINISH_TOKEN.to_string() } else { format!("{:?}", src[next]) }, table.get(&name).unwrap().keys()),
                        index: next
                    });
                }
            },
            Term::Terminal { type_: ref name, .. } => {
                if next == src.len() && FINISH_TOKEN == name {

                } else if term.match_token(&src[next]) {
                    let node = Node {
                        value: NodeType::Terminal(src[next].clone()),
                        childs: Vec::new(),
                        index: next,
                    };

                    tree.node.push(node);
                    
                    let mut f = if let NodeType::NonTerminal(NonTerminal { rule_, .. }) = &tree.node.value {
                        if tree.node.len() == rule_.expr.terms.len() { true } else { false }
                    } else { false };
                    while f {
                        tree = tree.parent();
                        f = if let NodeType::NonTerminal(NonTerminal { rule_, .. }) = &tree.node.value {
                            if tree.node.len() == rule_.expr.terms.len() { true } else { false }
                        } else { false };
                    }

                    next += 1;
                } else {
                    errs.push(ParseError{
                        msg: format!("Expected {:} but found {:}", name, if next == src.len() { FINISH_TOKEN.to_string() } else { format!("{:?}", src[next]) }),
                        index: next
                    });
                }
            },
            _ => panic!()
        }
    }
    if next == src.len() && errs.is_empty() {
        Ok(tree.finish().remove(0))
    } else if next == src.len() || !errs.is_empty() {
        Err(errs)
    } else {
        errs.push(ParseError {
            msg: "Program Too Long".to_string(),
            index: next
        });
        Err(errs)
    }
}
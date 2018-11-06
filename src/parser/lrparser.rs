use super::prelude::*;
use crate::lexer::re::StateTransferGraph;

use std::fmt;
use std::hash;
use std::cmp::PartialEq;
use std::ops::Deref;
use std::ops::DerefMut;
use std::collections::HashMap;
use std::collections::HashSet;




#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LRAction {
    Shift(usize), Reduce(Production)
}

impl fmt::Display for LRAction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LRAction::Shift(next) => write!(f, "Shift: {:}", next),
            LRAction::Reduce(rule) => write!(f, "Reduce: {:}", rule)
        }
    }
}

pub type LRTable = Vec<HashMap<Term, LRAction>>;


pub type LRAhead = HashSet<Term>;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LRItem {
    pub rule: Production,
    pub pos: usize,
}

impl hash::Hash for LRItem {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.rule.hash(state);
        self.pos.hash(state);
    }
}

impl fmt::Display for LRItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.pos == self.rule.expr.terms.len() {
            write!(f, "\t\tReduce: \t{:}, \tPos: {:}", self.rule, self.pos)
        } else {
            write!(f, "\t\tRule: \t{:}, \tPos: {:}", self.rule, self.pos)
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LRItems{
    items: HashMap<LRItem, LRAhead> 
}

impl LRItems {
    fn push(&mut self, item: LRItem, ahead: Term) -> bool {
        if let Some(v) = self.get_mut(&item) {                
            return v.insert(ahead);
        }        
        self.insert(item, set![ahead]);
        true
    }
    
    fn extend(&mut self, item: LRItem, ahead: LRAhead) -> bool {
        if let Some(v) = self.get_mut(&item) {
            let len = v.len();
            v.extend(ahead);
            return v.len() > len;
        }
        self.insert(item, ahead);
        true
    }

    fn extend_all(&mut self, rhs: Self) -> bool {
        let mut ret = false;
        for (item, ahead) in rhs.items.into_iter() {
            ret = self.extend(item, ahead) || ret;
        }
        ret
    }
}

impl fmt::Display for LRItems {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\n\t{:}", self.items.iter().map(|(x, y)| format!("{:} : {:?}", x, y)).collect::<Vec<String>>().join("\n\t"))
    }
}

impl PartialEq for LRItems {
    fn eq(&self, rhs: &Self) -> bool {
        self.keys().collect::<HashSet<_>>() == rhs.keys().collect::<HashSet<_>>()
    }
}


impl Deref for LRItems {
    type Target = HashMap<LRItem, LRAhead>;
    fn deref(&self) -> &Self::Target {
        &self.items
    }
}

impl DerefMut for LRItems {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.items
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Default)]
struct TerminalSet {
    items: HashSet<Term>
}

impl TerminalSet {
    fn len(&self) -> usize {
        self.items.len()
    }

    fn append(&mut self, rhs: &Self) -> bool {
        let mut ret = false;
        for item in &rhs.items {
            ret = self.items.insert(item.clone()) || ret;
        }
        ret
    }

    fn push(&mut self, rhs: Term) -> bool {
        self.items.insert(rhs)
    }

    fn append_epsilon(&mut self) -> bool {
        self.push(Term::terminal(EPSILON_TOKEN.to_string()))
    }
    
    fn remove_epsilon(&mut self) -> bool {
        self.items.remove(&Term::terminal(EPSILON_TOKEN.to_string()))
    }

    fn contains_epsilon(&self) -> bool {
        self.items.contains(&Term::terminal(EPSILON_TOKEN.to_string()))
    }
}



pub fn construct_lr_0(grammar: &Grammar) -> StateTransferGraph<LRItems, Term> {
    let mut graph: StateTransferGraph<LRItems, Term> = StateTransferGraph::new();
    let closure = |s: &LRItems| -> LRItems {
        let mut ret = s.clone();
        let mut flag = true;
        while flag {
            let mut set = LRItems::default();
            for (&LRItem { ref rule, ref pos, .. }, _) in &*ret {
                if pos < &rule.expr.terms.len() {
                    if let Term::NonTerminal { name, .. } = &rule.expr.terms[*pos] {
                        for p in &grammar.productions {
                            if &p.name != name { continue; }
                            set.extend(LRItem {
                                rule: p.clone(),
                                pos: 0,
                                ..Default::default()
                            }, LRAhead::default());
                        }
                    }
                }
            }
            flag = ret.extend_all(set);
        }
        ret
    };

    let goto = |s: &LRItems, t: Term| -> LRItems {
        let mut ret = LRItems::default();
        for (&LRItem { ref rule, ref pos, .. }, _) in s.deref() {
            if pos < &rule.expr.terms.len() {
                if rule.expr.terms[*pos] == t {
                    ret.extend_all(closure(&LRItems { items: map![LRItem { rule: rule.clone(), pos: pos + 1, ..Default::default() } => LRAhead::default()] }));
                }
            }
        }
        ret
    };

    let start = LRItems{ items: map![LRItem {
        rule: Production {
            name: FINISH_TOKEN.to_string(),
            label: String::new(),
            expr: Expression {
                terms: vec![Term::NonTerminal {
                    name: grammar.start_symbol.clone(),
                    unwrap: true,
                }]
            },
            ..Default::default()
        },
        pos: 0,
        ..Default::default()
    } => LRAhead::default() ]};
    let start = closure(&start);
    // start.remove(0);
    let start = graph.add_state_with_data(start);
    graph.mark_as_start(start);
    let mut state = 0;
    while state < graph.vertices.len() {
        {
            let ss = graph.vertices[state].data.clone();
            for (s, _) in &*ss {
                if s.rule.name == grammar.start_symbol && s.pos == s.rule.expr.terms.len() {
                    graph.mark_as_end(state);
                }
            }
        }
        for t in grammar.symbols() {
            let g = goto(&graph.vertices[state].data, t.clone());
            if !g.is_empty() {
                let p = if let Some(p) = graph.vertices.iter().position(|x| x.data == g) { p } else { graph.add_state_with_data(g) };
                if let None = graph.get_transition(state, t.clone()) {
                    graph.add_transfer(state, p, t.clone());
                }
            }
        }
        // println!("current state: {:?}", graph);
        state += 1;
    }
    graph
}


pub fn construct_lalr_1(grammar: &Grammar) -> StateTransferGraph<LRItems, Term> {
    // Initialize first , follow
    let mut first: HashMap<Term, TerminalSet> = HashMap::new();
    let mut follow: HashMap<String, TerminalSet> = HashMap::new();

    let get_first = |terms: &[Term], first: &HashMap<Term, TerminalSet>| {
        let mut tset = TerminalSet::default();
        let mut flag = true;
        for term in terms {
            flag = false;
            let t = first.get(term).expect(&format!("Term not found : {:} from {:?}", term, terms));
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
    {
        let mut tset = TerminalSet::default();
        tset.push(Term::terminal(FINISH_TOKEN));
        first.insert(Term::terminal(FINISH_TOKEN), tset);
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
                    flag_move |= follow.get_mut(t).expect(&format!("Follow not found : {:}", t)).append(&tset);
                    if fep {
                        let fx = follow.get_mut(&p.name).expect("5").clone();
                        flag_move |= follow.get_mut(t).expect("6").append(&fx);                    
                    }
                }
            }
        }
        if !flag_move { break; }
    }

    if DEBUG!() { println!("first: {:?}\nfollow: {:?}", first, follow); }
    // Initialize LRGraph
    let mut graph: StateTransferGraph<LRItems, Term> = StateTransferGraph::new();
    let closure = |s: &LRItems| -> LRItems {
        let mut ret = s.clone();
        let mut rset = ret.clone();
        let mut flag = true;
        while flag {
            let mut set = LRItems::default();
            for (&LRItem { ref rule, ref pos }, ahead) in &*rset {
                if pos < &rule.expr.terms.len() {
                    if let Term::NonTerminal { name, .. } = &rule.expr.terms[*pos] {
                        for ahead in ahead {
                            let f = if *pos == rule.expr.terms.len() - 1 {
                                get_first(&[ahead.clone()], &first)
                            } else {
                                get_first(&[&rule.expr.terms[*pos+1..], &[ahead.clone()]].concat(), &first)
                            };
                            for p in &grammar.productions {
                                if &p.name != name { continue; }
                                set.extend(LRItem {
                                    rule: p.clone(),
                                    pos: 0,
                                }, f.items.clone());
                            }
                        }
                    }
                }
            }
            rset = set.clone();
            flag = ret.extend_all(set);
        }
        ret
    };

    let goto = |s: &LRItems, t: Term| -> LRItems {
        let mut ret = LRItems::default();
        for (&LRItem { ref rule, ref pos }, ahead) in s.deref() {
            if pos < &rule.expr.terms.len() && rule.expr.terms[*pos] == t {
                ret.extend_all(closure(&LRItems { items: map![LRItem { rule: rule.clone(), pos: pos + 1} => ahead.clone()] }));
            }
        }
        ret
    };

    let start = LRItems { items: map![LRItem {
        rule: Production {
            name: FINISH_TOKEN.to_string(),
            label: String::new(),
            expr: Expression {
                terms: vec![Term::NonTerminal {
                    name: grammar.start_symbol.clone(),
                    unwrap: true,
                }]
            },
            ..Default::default()
        },
        pos: 0,
    } => set![Term::terminal(FINISH_TOKEN)]] };

    let start = graph.add_state_with_data(closure(&start));
    graph.mark_as_start(start);
    // let mut state = 0;
    let mut stack = Vec::new();
    stack.push(0);
    // use a stack to allow re-initialization of one state
    while let Some(state) = stack.pop() {
        {
            let ss = graph.vertices[state].data.clone();
            for (s, _) in &*ss {
                if s.rule.name == FINISH_TOKEN.to_string() && s.pos == s.rule.expr.terms.len() {
                    graph.mark_as_end(state);
                }
            }
        }
        for t in grammar.symbols() {
            let g = goto(&graph.vertices[state].data, t.clone());
            if !g.is_empty() {
                let p = if let Some(p) = graph.vertices.iter().position(|x| x.data == g) { 
                    if graph.vertices[p].data.extend_all(g) { if !stack.contains(&p) { stack.push(p); } }
                    p 
                } else {
                    let p = graph.add_state_with_data(g);
                    if !stack.contains(&p) { stack.push(p); }
                    p
                };
                if let None = graph.get_transition(state, t.clone()) {
                    graph.add_transfer(state, p, t.clone());
                }
            }
        }
        if DEBUG!() && VERBOSE!() { println!("current state: {:} : {:}, {:}", state, stack.len(), graph.vertices.len()); }
    }

    graph
}


#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    pub msg: String,
    pub index: usize,
}

pub fn parse(src: &[Token], grammar: &Grammar) -> Result<Node, ParseError> {
    let graph = construct_lalr_1(grammar);
    let table = construct_table(&graph).unwrap();
    parse_with_table(src, &table)
}


pub fn parse_with_graph(src: &[Token], graph: &StateTransferGraph<LRItems, Term>) -> Result<Node, ParseError> {
    if DEBUG!() && VERBOSE!() { println!("{:}", graph); }
    
    #[derive(Debug)]
    enum StackItem {
        State(usize), NonTerminal(String), Terminal(Token)
    }
    let mut stack = Vec::<StackItem>::new();
    stack.push(StackItem::State(graph.start));

    let mut stack_out = Vec::new();

    let mut next = 0;
    'outer: loop {
        let token = if next < src.len() { src[next].clone() } else { Token {
            type_ : FINISH_TOKEN.to_string(),
            value_: FINISH_TOKEN.to_string(),
            line_: 0,
        } };
        let curr_state = if let Some(&StackItem::State(s)) = stack.last() { s } else { panic!("State not on top of stack !") };

        if DEBUG!() { println!("#{:} Step: [{:}, {:}]", next, curr_state, token); }
        // Shift
        if let Some(next_state) = graph.get_transition(curr_state, Term::from(token.clone())).or(graph.get_transition(curr_state, Term::terminal(token.type_.clone()))) {
            let mut conflict = false;
            // Check shift-reduce conflict
            let mut shift_precedence = None;
            for (item, _) in &*graph.vertices[curr_state].data {
                if item.pos < item.rule.expr.terms.len() && item.rule.expr.terms[item.pos].match_token(&token) {
                    if shift_precedence == None || shift_precedence.unwrap() > item.rule.precedence { shift_precedence = Some(item.rule.precedence); }
                }
            }
            let shift_precedence = shift_precedence.unwrap();

            for (item, ahead) in &*graph.vertices[curr_state].data {
                if item.pos == item.rule.expr.terms.len() {
                    // Check lookahead
                    if ahead.len() > 0 && !ahead.contains(&Term::from(&token)) && !ahead.contains(&Term::terminal(token.type_.clone())) { continue; }

                    if DEBUG!() && VERBOSE!() { println!("SHIFT_REDUCE CONFLICT"); }
                    if item.rule.precedence < shift_precedence || (item.rule.precedence == shift_precedence && item.rule.associativity == Associativity::Left) {
                        conflict = true;
                        break
                    }
                }
            }

            if !conflict {
                stack.push(StackItem::Terminal(token.clone()));
                stack.push(StackItem::State(next_state));
                stack_out.push(Node {
                    value: NodeType::Terminal(token.clone()),
                    childs: Vec::new(),
                    index: next
                });
                next += 1;
                if DEBUG!() { 
                    if VERBOSE!() { println!("Shifted: {:?}", stack); }
                    else { println!("Shifted to {:}", next_state); }
                }
                continue 'outer;
            }
        }

        // Reduce
        let mut reduce_item: Option<LRItem> = None;

        for (item, ahead) in &*graph.vertices[curr_state].data {
            if item.pos == item.rule.expr.terms.len() {
                // Check lookahead
                if ahead.len() > 0 && !ahead.contains(&Term::from(&token)) && !ahead.contains(&Term::terminal(token.type_.clone())) { continue; }

                if reduce_item == None || reduce_item.as_ref().unwrap().rule.precedence > item.rule.precedence { 
                    if DEBUG!() && VERBOSE!() && reduce_item != None { println!("REDUCE_REDUCE CONFLICT"); }
                    reduce_item = Some(item.clone());
                }
            }
        }

        if let Some(item) = reduce_item {

            if DEBUG!() { println!("Reduce by: {:}", item.rule); }
            let mut node = Node {
                value: NodeType::NonTerminal(NonTerminal {
                    type_: item.rule.name.clone(),
                    value_: item.rule.label.clone(),
                    rule_: item.rule.clone(),
                }),
                childs: Vec::new(),
                index: next
            };
            for _ in 0..item.pos {
                stack.pop();
                stack.pop();

                let n = stack_out.pop().expect("Not enough node to reduce");
                node.index = n.index;
                node.childs.insert(0, n);
            }

            stack_out.push(node);

            let curr_state = if let Some(&StackItem::State(s)) = stack.last() { s } else { panic!("Reduce: State not on top of stack !") };
            stack.push(StackItem::NonTerminal(item.rule.name.clone()));
            if DEBUG!() && VERBOSE!() { println!("Reduced: Stack: {:?}", stack); }
            if curr_state == graph.start && item.rule.name == FINISH_TOKEN.to_string() {
                break 'outer;
            }
            if let Some(next_state) = graph.get_transition(curr_state, Term::nonterminal(item.rule.name.as_str())) {
                stack.push(StackItem::State(next_state));
            } else {
                panic!("No transition found");
            }

        } else {
            panic!("No action")
        }
    }

    if DEBUG!() { println!("Result : {:?}", stack); }
    Ok(stack_out.pop().unwrap().childs.remove(0))
}


pub fn parse_with_table(src: &[Token], table: &LRTable) -> Result<Node, ParseError> {
    // if DEBUG!() { println!("{:}", graph); }
    
    #[derive(Debug)]
    enum StackItem {
        State(usize), NonTerminal(String), Terminal(Token)
    }
    let mut stack = Vec::<StackItem>::new();
    stack.push(StackItem::State(0));

    let mut stack_out = Vec::new();

    let mut next = 0;
    'outer: loop {
        let token = if next < src.len() { src[next].clone() } else { Token {
            type_ : FINISH_TOKEN.to_string(),
            value_: FINISH_TOKEN.to_string(),
            line_: 0,
        } };
        let curr_state = if let Some(&StackItem::State(s)) = stack.last() { s } else { panic!("State not on top of stack !") };

        if DEBUG!() { println!("#{:} Step: [{:}, {:}]", next, curr_state, token.type_); }
        // Shift

        match table[curr_state].get(&Term::from(&token)).or(table[curr_state].get(&Term::terminal(token.type_.as_str()))) {
            Some(LRAction::Shift(next_state)) => {
                stack.push(StackItem::Terminal(token.clone()));
                stack.push(StackItem::State(*next_state));
                stack_out.push(Node {
                    value: NodeType::Terminal(token.clone()),
                    childs: Vec::new(),
                    index: next
                });
                next += 1;
                if DEBUG!() { 
                    if VERBOSE!() { println!("Shifted: {:?}", stack); }
                    else { println!("Shifted to {:}", next_state); }
                }
            },
            Some(LRAction::Reduce(rule)) => {
                if DEBUG!() { println!("Reduce by: {:}", rule); }
                let mut node = Node {
                    value: NodeType::NonTerminal(NonTerminal {
                        type_: rule.name.clone(),
                        value_: rule.label.clone(),
                        rule_: rule.clone(),
                    }),
                    childs: Vec::new(),
                    index: next
                };
                for _ in 0..rule.expr.terms.len() {
                    stack.pop();
                    stack.pop();

                    let n = stack_out.pop().expect("Not enough node to reduce");
                    node.index = n.index;
                    node.childs.insert(0, n);
                }

                stack_out.push(node);

                let curr_state = if let Some(&StackItem::State(s)) = stack.last() { s } else { panic!("Reduce: State not on top of stack !") };
                stack.push(StackItem::NonTerminal(rule.name.clone()));
                if DEBUG!() && VERBOSE!() { println!("Reduced: Stack: {:?}", stack); }
                if curr_state == 0 && rule.name == FINISH_TOKEN.to_string() {
                    break 'outer;
                }
                if let Some(LRAction::Shift(next_state)) = table[curr_state].get(&Term::nonterminal(rule.name.as_str())) {
                    stack.push(StackItem::State(*next_state));
                } else {
                    return Err(ParseError {
                        msg: format!("No transition found after reduced: {:}", rule),
                        index: next,
                    });
                }
            },
            None => {
                return Err(ParseError {
                    msg: format!("No action found for token: {:} when state: {:}", token, curr_state),
                    index: next,
                });
            }
        }
    }

    if DEBUG!() { println!("Result : {:?}", stack); }
    Ok(stack_out.pop().unwrap().childs.remove(0))
}


pub fn construct_table(graph: &StateTransferGraph<LRItems, Term>) -> Result<LRTable, String> {
    if DEBUG!() && VERBOSE!() { println!("{:}", graph); }
    let mut table = LRTable::default();
    for s in 0..graph.vertices.len() {
        let mut map = HashMap::<Term, (Production, LRAction)>::new();
        for (item, ahead) in &*graph.vertices[s].data {
            if item.pos < item.rule.expr.terms.len() {
                let f = match map.get(&item.rule.expr.terms[item.pos]) {
                    Some((rule, action @ LRAction::Reduce(..))) => {
                        if DEBUG!() { println!("DETECTED CONFLICT on {:} {:} : {:} {:}", s, item.rule.expr.terms[item.pos], LRAction::Shift(graph.get_transition(s, item.rule.expr.terms[item.pos].clone()).expect("No transition for item")), action); }
                        item.rule.precedence < rule.precedence || (item.rule.precedence == rule.precedence && item.rule.associativity == Associativity::Right)
                    },
                    Some((rule, _)) => {
                        item.rule.precedence < rule.precedence || (item.rule.precedence == rule.precedence && item.rule.associativity == Associativity::Right)
                    },
                    None => true
                };
                if f {
                    map.insert(item.rule.expr.terms[item.pos].clone(), (item.rule.clone(), LRAction::Shift(graph.get_transition(s, item.rule.expr.terms[item.pos].clone()).expect("No transition for item"))));
                }
            } else {
                for ahead in ahead {
                    let f = match map.get(ahead) {
                        Some((rule, action)) => {
                            if DEBUG!() { println!("DETECTED CONFLICT on {:} {:} : {:} {:}", s, ahead, LRAction::Reduce(item.rule.clone()), action); }
                            item.rule.precedence < rule.precedence || (item.rule.precedence == rule.precedence && item.rule.associativity == Associativity::Left)
                        },
                        None => true
                    };
                    if f {
                        map.insert(ahead.clone(), (item.rule.clone(), LRAction::Reduce(item.rule.clone())));
                    }
                }
            }
        }
        table.push(map.into_iter().map(|(t, (_, a))| (t, a)).collect());
    }
    if DEBUG!() {        
        for (i, map) in table.iter().enumerate() {
            println!("# {:}", i);
            for (k, v) in map {
                println!("\t{:} : {:}", k, v);
            }
        }
    }
    Ok(table)
}
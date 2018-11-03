use super::grammar::*;
use super::*;
use std::collections::HashMap;
use super::utils::*;


pub trait Transform {

    fn inverse(&self) -> Box<dyn Transform>;
}


#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Substitution {
    rule: Production,
    source: Production,
    target: Production,
    pos: usize,
}

impl Transform for Substitution {
    fn inverse(&self) -> Box<dyn Transform> {
        Box::new(self.clone())
    }
}

impl Substitution {

    fn new(rule: Production, source: Production, pos: usize) -> Self {
        let mut target = source.clone();
        target.label = target.label + "#" + &rule.name + "#" + &rule.label;
        target.expr.terms.remove(pos);
        let slice = target.expr.terms.split_off(pos);
        target.expr.terms.extend(rule.expr.terms.clone());
        target.expr.terms.extend(slice);
        Self {
            rule: rule,
            source: source,
            pos: pos,
            target: target
        }
    }

    fn apply_to_grammar(&self, mut grammar: Grammar) -> Grammar {
        grammar.productions = grammar.productions.into_iter().map(|x| if x == self.source { self.target.clone() } else { x }).collect();
        grammar
    }

    fn apply_to_grammar_inverse(&self, mut grammar: Grammar) -> Grammar {
        grammar.productions = grammar.productions.into_iter().map(|x| if x == self.target { self.source.clone() } else { x }).collect();
        grammar
    }

    fn apply_to_parser_tree(&self, mut node: Node) -> Node {
        if let NodeType::NonTerminal(NonTerminal { ref rule_, .. }) = node.value {
            if rule_ == &self.source {
                let set = node.childs.remove(self.pos);
                let slice = node.childs.split_off(self.pos);
                node.childs.extend(set.childs);
                node.childs.extend(slice);
            }
        }
        for _ in 0..node.childs.len() {
            let t = node.childs.pop().unwrap();
            node.childs.insert(0, self.apply_to_parser_tree(t));
        }
        node
    }

    pub fn apply_to_parser_tree_inverse(&self, mut node: Node) -> Node {
        if let NodeType::NonTerminal(NonTerminal { ref mut rule_, .. }) = node.value {
            // println!("Comparsing\n{:?}\n{:?}\n", rule_, self.target);
            if rule_ == &self.target {
                // println!("Found {:?}", self);
                let mut new_node = Node {
                    value: NodeType::NonTerminal(NonTerminal {
                        rule_: self.rule.clone(),
                        type_: self.rule.name.clone(),
                        ..Default::default()
                    }),
                    index: 0,
                    childs: Vec::new()
                };
                new_node.childs = node.childs.drain(self.pos..self.pos + self.rule.expr.terms.len()).collect();
                new_node.index = new_node.childs[0].index;
                node.childs.insert(self.pos, new_node);
                *rule_ = self.source.clone();
                // println!("Now: {:?}", self.pos..self.pos + rule_.expr.alters[0].terms.len());
            }
        
        node.childs = node.childs.into_iter().map(|t| self.apply_to_parser_tree_inverse(t)).collect();
        }
        node
    }

}


pub fn convert_to_formal_grammar(mut grammar: Grammar) -> Grammar {
    let mut stack: Vec<Production> = grammar.productions.into_iter().rev().collect();
    grammar.productions = Vec::new();
    while let Some(mut p) = stack.pop() {
        let name = p.name.clone() + "$" + &p.label;
        let pp = p.clone();
        p.expr.terms = p.expr.terms.into_iter().enumerate().map(|(i, y)| 
        match y {
            Term::Group { expr, .. } => {
                stack.push(Production {
                    name: name.clone() + "$group#" + &i.to_string(),
                    label: "#0".to_string(),
                    expr: *expr,
                    associativity: pp.associativity,
                    precedence: pp.precedence
                });
                Term::NonTerminal {
                    name: name.clone() + "$group#" + &i.to_string(),
                    unwrap: true
                }
            },
            Term::Optional { expr, .. } => {
                stack.push(Production {
                    name: name.clone() + "$optionalterm#" + &i.to_string(),
                    label: "#0".to_string(),
                    expr: *expr,
                    associativity: pp.associativity,
                    precedence: pp.precedence
                });
                stack.push(Production {
                    name: name.clone() + "$optional#" + &i.to_string(),
                    label: String::from("epsilon"),
                    expr: Expression {
                        terms: Vec::new(),
                    },
                    associativity: pp.associativity,
                    precedence: pp.precedence
                });
                stack.push(Production {
                    name: name.clone() + "$optional#" + &i.to_string(),
                    label: String::from("main"),
                    expr: Expression {
                            terms: vec![Term::NonTerminal {
                                name: name.clone() + "$optionalterm#" + &i.to_string(),
                                unwrap: true
                            }]
                    },
                    associativity: pp.associativity,
                    precedence: pp.precedence
                });
                Term::NonTerminal {
                    name: name.clone() + "$optional#" + &i.to_string(),
                    unwrap: true
                }
            },
            Term::Repetition { expr, .. } => {
                stack.push(Production {
                    name: name.clone() + "$repetitionterm#" + &i.to_string(),
                    label: "#0".to_string(),
                    expr: *expr,
                    associativity: pp.associativity,
                    precedence: pp.precedence
                });
                stack.push(Production {
                    name: name.clone() + "$repetition#" + &i.to_string(),
                    label: String::from("epsilon"),
                    expr: Expression {
                        terms: Vec::new(),
                    },
                    associativity: pp.associativity,
                    precedence: pp.precedence
                });
                stack.push(Production {
                    name: name.clone() + "$repetition#" + &i.to_string(),
                    label: String::from("main"),
                    expr: Expression {
                            terms: vec![Term::NonTerminal {
                                name: name.clone() + "$repetitionterm#" + &i.to_string(),
                                unwrap: true
                            }, Term::NonTerminal {
                                name: name.clone() + "$repetition#" + &i.to_string(),
                                unwrap: true
                            }]
                    },
                    associativity: pp.associativity,
                    precedence: pp.precedence
                });
                Term::NonTerminal {
                    name: name.clone() + "$repetition#" + &i.to_string(),
                    unwrap: true
                }
            },
            _ => y
        }).collect();

        grammar.productions.push(p);

    } 
    grammar
}

pub fn elimate_undirect_left_recursion(mut grammar: Grammar) -> Grammar {
    let non_terminals = grammar.non_terminals();
    for i in 0..non_terminals.len() {
        for j in 0..i {
            let mut tfs = Vec::new();
            for p in &grammar.productions {
                if p.name != non_terminals[i] { continue; }
                if let Some(&Term::NonTerminal { ref name, .. }) = p.expr.terms.get(0) {
                    if non_terminals.iter().position(|x| x==name).expect("Unexpected nonterminal") == j {
                        for pr in &grammar.productions {
                            if pr.name != non_terminals[j] { continue; }
                            let tf = Substitution::new(pr.clone(), p.clone(), 0);
                            tfs.push(tf);
                        }
                    }
                }            
            }
            for tf in tfs {
                // grammar = tf.apply_to_grammar(grammar);
                if DEBUG!() { println!("Sub:\n{:}\n{:}\n{:}", tf.rule.dump(), tf.source.dump(), tf.target.dump()); }
                grammar.productions.push(tf.target.clone());
                let pos = grammar.productions.iter().position(|x| x==&tf.source);
                if let Some(pos) = pos {
                    grammar.productions.remove(pos);
                }
                grammar.add_transform(tf);
            }
            grammar = elimate_left_recursion(grammar);
        }

    }
    grammar
}

pub fn elimate_left_recursion(mut grammar: Grammar) -> Grammar {
    let mut set = Vec::new();
    for nt in grammar.non_terminals() {
        let mut alpha = Vec::new();
        let mut beta = Vec::new();
        let new_name = nt.clone() + "##";
        let mut precedence = None;

        for p in &grammar.productions {
            if p.name != nt { continue; }
            if precedence == None || precedence.unwrap() > p.precedence { precedence = Some(p.precedence); }
            if let Some(&Term::NonTerminal { ref name, .. }) = p.expr.terms.get(0) {
                if name == &nt {
                    alpha.push(Production {
                        name: new_name.clone(),
                        label: p.label.clone(),
                        expr: Expression {
                            terms: p.expr.terms.clone().into_iter().skip(1).collect()
                        },
                    associativity: p.associativity,
                    precedence: p.precedence
                    });
                    continue;
                }
            }
            beta.push(p.clone());
        }
        if alpha.is_empty() { 
            set.extend(beta);
            continue;
        }

        for x in alpha.iter_mut().chain(beta.iter_mut()) {
            x.expr.terms.push(Term::NonTerminal {
                name: new_name.clone(),
                unwrap: false,
            });
        }

        set.extend(beta);
        set.extend(alpha);
        set.push(Production {
            name: new_name.clone(),
            label: "epsilon".to_string(),
            expr: Expression {
                terms: Vec::new(),
            },
            associativity: Associativity::default(),
            precedence: if precedence == None { 0 } else { precedence.unwrap() },
        });
    }

    grammar.productions = set;
    grammar
}




pub fn retrieve_left_recursion(node: Node) -> Node {
    let mut in_stack = Vec::new();
    let mut out_stack = Vec::new();

    in_stack.push(Some(node));

    while let Some(node) = in_stack.pop() {
        if let Some(mut node) = node {

            // Do some operation
            /*******************/
            let mut flag = false;
            let mut flag_epsilon = false;
            if let Some(n) = node.childs.last() {
                if let NodeType::NonTerminal(NonTerminal { type_, .. }) = &n.value {
                    // If epsilon, remove it
                    if type_.ends_with("##") {
                        if n.childs.len() == 0 {
                            // flag = false;
                            flag_epsilon = true;
                        } else {
                            flag = true;
                        }
                    }
                }
            }
            if flag_epsilon {
                node.childs.pop();
                if let NodeType::NonTerminal(NonTerminal { ref mut rule_, .. }) = node.value {
                    rule_.expr.terms.pop();
                }
            }
            if flag {
                let mut extra_node = node.childs.pop().unwrap();
                if let NodeType::NonTerminal(NonTerminal { ref mut rule_, .. }) = node.value {
                    rule_.expr.terms.pop();
                }
                if let NodeType::NonTerminal(NonTerminal { ref mut type_, ref mut rule_, .. }) = &mut extra_node.value {
                    *type_ = type_[0..type_.len() - 2].to_string();
                    rule_.name = rule_.name[0..rule_.name.len() - 2].to_string();
                    rule_.expr.terms.insert(0, Term::NonTerminal {
                        name: rule_.name.clone(),
                        unwrap: false,
                    });
                }
                extra_node.childs.insert(0, node);
                in_stack.push(Some(extra_node));
                continue;
            }
            /*******************/

            in_stack.push(None);
            in_stack.extend(node.childs.into_iter().map(|x| Some(x)));
            node.childs = Vec::new();
            out_stack.push(Some(node));
            out_stack.push(None);
        } else {
            let mut set = Vec::new();
            while let Some(Some(child)) = out_stack.pop() {
                set.push(child);
            }
            let mut node = out_stack.pop().unwrap().unwrap();
            node.childs = set;
            out_stack.push(Some(node));
        }
    }
    out_stack.pop().unwrap().unwrap()
}


pub fn retrieve_undirect_left_recursion(mut node: Node, grammar: &Grammar) -> Node {
    for tf in &grammar.transforms {
        node = tf.apply_to_parser_tree_inverse(node);
    }
    node
}

pub fn left_factor(mut grammar: Grammar) -> Grammar {
    let mut map = HashMap::<String, HashMap<Term, Vec<Production>>>::new();
    let mut set = Vec::new();
    for nt in grammar.non_terminals() {
        map.insert(nt, HashMap::new());
    }
    for p in grammar.productions {
        if p.expr.terms.len() == 0 {
            set.push(p);
            continue;
        }
        if let Some(ps) = map.get_mut(&p.name).unwrap().get_mut(&p.expr.terms[0]) {
            ps.push(p);
            continue;
        }
        map.get_mut(&p.name).unwrap().insert(p.expr.terms[0].clone(), vec![p]);
    }
    grammar.productions = set;
    for (n, pm) in map {
        for (i, (t, mut ps)) in pm.into_iter().enumerate() {
            if ps.len() == 1 {
                grammar.productions.push(ps.remove(0));
            } else {
                grammar.productions.push(Production {
                    name: n.clone(),
                    label: "#".to_string() + &i.to_string(),
                    expr: Expression {
                        terms: vec![t, Term::NonTerminal {
                            name: n.clone() + "$#" + &i.to_string(),
                            unwrap: true,
                        }]
                    },
                    ..Default::default()
                });
                grammar.productions.extend(ps.into_iter().map(|mut p| {
                    p.name = n.clone() + "$#" + &i.to_string();
                    p.expr.terms.remove(0);
                    p
                }));
            }
        }
    }
    grammar
}


pub fn retrieve_unwrap(node: Node) -> Node {
    let mut in_stack = Vec::new();
    let mut out_stack = Vec::new();

    in_stack.push(Some(node));

    while let Some(node) = in_stack.pop() {
        if let Some(mut node) = node {
            in_stack.push(None);
            in_stack.extend(node.childs.into_iter().map(|x| Some(x)));
            node.childs = Vec::new();
            out_stack.push(Some(node));
            out_stack.push(None);
        } else {
            let mut set = Vec::new();
            while let Some(Some(child)) = out_stack.pop() {
                set.push(child);
            }
            let mut node = out_stack.pop().unwrap().unwrap();
            node.childs = set;

            // Do operation after

            if let NodeType::NonTerminal(NonTerminal { rule_, .. }) = &node.value {
                node.childs = node.childs.into_iter().zip(&rule_.expr.terms).flat_map(|(n, t)| {
                    if let Term::NonTerminal { unwrap, .. } = t {
                        if *unwrap {
                            n.childs
                        } else { vec![n] }
                    } else { vec![n] }
                }).collect()
            };

            out_stack.push(Some(node));
        }
    }
    out_stack.pop().unwrap().unwrap()
}

pub fn elimate_epsilon(mut grammar: Grammar) -> Grammar {
    loop {
        let mut tfs = Vec::new();
        for rule in &grammar.productions {
            if rule.expr.terms.len() == 0 {
                for r in &grammar.productions {
                    if let Some(p) = r.expr.terms.iter().position(|x| if let Term::NonTerminal { name, .. } = x { name == &rule.name } else { false }) {
                        tfs.push(Substitution::new(r.clone(), rule.clone(), p));
                    }
                }
                break;
            }
        }

        for tf in &tfs {
            grammar = tf.apply_to_grammar(grammar);
        }
    }
    grammar
}
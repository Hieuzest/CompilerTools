use super::prelude::*;

pub type Derivations = Vec<Production>;

pub fn build_leftmost(derivations: &Derivations, tokens: &[Token]) -> Node {
    let mut tree = Node::new(NodeType::NonTerminal(NonTerminal::from(&derivations[0]))).zipper();
    let mut next = 0;
    for derivation in derivations {
        if DEBUG!() { println!("Derivation: {:}; token_ind: {:}", derivation, next); }
        if let NodeType::NonTerminal(NonTerminal { ref mut type_, ref mut value_, ref mut rule_ }) = &mut tree.node.value {
            if type_ != &derivation.name { panic!("Unexpected derivation rule !") }
            *value_ = derivation.label.clone();
            *rule_ = derivation.clone();
            tree.node.childs.extend(derivation.expr.terms.iter().map(|x| {
                if let Term::Terminal { .. } = x {
                    next += 1;
                    Node::new(NodeType::Terminal(tokens[next - 1].clone()))
                } else if let Term::NonTerminal { name, .. } = x {
                    Node::new(NodeType::NonTerminal(NonTerminal {
                        type_: name.clone(),
                        ..Default::default()
                    }))
                } else {
                    panic!()
                }
            }));

            // Move to next nonterminal
            let mut f = false;
            if tree.node.childs.len() > 0 {
                tree = tree.child(0);
                f = true;
            }
            while !f || if let NodeType::Terminal(_) = &tree.node.value { true } else { false } {
                if tree.check_parent() {
                    let pos = tree.index_in_parent + 1;
                    tree = tree.parent();
                    if pos < tree.node.len() {
                        tree = tree.child(pos);
                        f = true;
                    } else {
                        f = false;
                    }
                } else {
                    break;
                }
            }
        } else {
            panic!("Built failed !");          
        }
    }
    // println!("\n\n{:?}\n\n", tree);
    tree.finish()
}


pub fn build_rightmost(derivations: &Derivations, tokens: &[Token]) -> Node {
    let mut tree = Node::new(NodeType::NonTerminal(NonTerminal::from(&derivations[0]))).zipper();
    let mut next = 0;
    for derivation in derivations {
        if DEBUG!() { println!("Derivation: {:}; token_ind: {:}", derivation, next); }
        if let NodeType::NonTerminal(NonTerminal { ref mut type_, ref mut value_, ref mut rule_ }) = &mut tree.node.value {
            if type_ != &derivation.name { panic!("Unexpected derivation rule !") }
            *value_ = derivation.label.clone();
            *rule_ = derivation.clone();
            tree.node.childs.extend(derivation.expr.terms.iter().map(|x| {
                if let Term::Terminal { .. } = x {
                    next += 1;
                    Node::new(NodeType::Terminal(tokens[next - 1].clone()))
                } else if let Term::NonTerminal { name, .. } = x {
                    Node::new(NodeType::NonTerminal(NonTerminal {
                        type_: name.clone(),
                        ..Default::default()
                    }))
                } else {
                    panic!()
                }
            }));

            // Move to next nonterminal
            let mut f = false;
            if tree.node.childs.len() > 0 {
                let pos = tree.node.childs.len() - 1;
                tree = tree.child(pos);
                f = true;
            }
            while !f || if let NodeType::Terminal(_) = &tree.node.value { true } else { false } {
                if tree.check_parent() {
                    let pos = tree.index_in_parent;
                    tree = tree.parent();
                    if pos > 0 {
                        tree = tree.child(pos - 1);
                        f = true;
                    } else {
                        f = false;
                    }
                } else {
                    break;
                }
            }
        } else {
            panic!("Built failed !");          
        }
    }
    // println!("\n\n{:?}\n\n", tree);
    tree.finish()
}
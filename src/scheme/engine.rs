use super::beam::*;
use super::env::*;
use super::symbol::*;
use std::iter;
use crate::utils::*;

macro_rules! cont {
    ($val: expr, $env: expr, $level: expr) => {
        Datum::Continuation{ expr: $val.clone(), env: $env.clone(), level: $level }.wrap()
    };
}

macro_rules! cont_expr {
    ($cont: expr) => {
        if let Datum::Continuation { ref expr, .. } = *$cont.clone().borrow() {
            expr.clone()
        } else {
            Err(RuntimeError::new("ice: non-continuation in call stack"))?
        }
    };
}

macro_rules! cont_env {
    ($cont: expr) => {
        if let Datum::Continuation { ref env, .. } = *$cont.clone().borrow() {
            env.clone()
        } else {
            Err(RuntimeError::new("ice: non-continuation in call stack"))?
        }
    };
}

macro_rules! cont_level {
    ($cont: expr) => {
        if let Datum::Continuation { ref level, .. } = *$cont.clone().borrow() {
            level.clone()
        } else {
            Err(RuntimeError::new("ice: non-continuation in call stack"))?
        }
    };
}

pub fn proc_eval(term: Value, env: Env) -> Result<Value, RuntimeError> {
    let mut stack_operands: Vec<Value> = Vec::new();
    // let mut stack_operators: Vec<Value> = Vec::new();
    let mut continuations: Vec<Value> = Vec::new();


    continuations.push(cont!(term, env, 0));
    // stack_operators.push(term);


    while let Some(cont) = continuations.pop() {
        let cc = cont_expr!(cont);
        let env = cont_env!(cont);
        let level = cont_level!(cont);

        match *cc.clone().borrow() {
            Datum::Pair(ref a, ref d) if level > 0 => {
                if !a.borrow().is_holder() {
                    let cc = List::clone(cc.clone());
                    cc.borrow_mut().set_car(a.clone());
                    continuations.push(cont!(cc, env, level));

                    let mut list = List::new();
                    for li in List::from(d.clone()) {
                        if let ListItem::Item(val) = li {
                            list = list.chain(iter::once(ListItem::Item(SymbolTable::holder()))).collect();
                            continuations.push(cont!(val, env, level));
                        } else if let ListItem::Ellipsis(val) = li {
                            list = list.chain(iter::once(ListItem::Ellipsis(SymbolTable::holder()))).collect();
                            continuations.push(cont!(val, env, level));
                        }
                    }
                    cc.borrow_mut().set_cdr(list.into())?;
                } else {
                    let len = cc.borrow().len();
                    let mut operands = List::new();
                    for i in 0..len {
                        operands = operands.chain(iter::once(ListItem::Item(stack_operands.pop().unwrap()))).collect();
                    }
                    stack_operands.push(cc.clone());
                }
            },
            _ if level > 0 => {
                stack_operands.push(cc.clone());
            },
            Datum::Pair(ref a, ref d) => {
                match *a.clone().borrow() {
                    Datum::SpecialForm(SpecialForm::Begin) => {
                        let cc = List::clone(cc.clone());
                        let mut list = cc.borrow().cdr()?.clone();
                        loop {
                            println!("Begin_iter {:?}", list.borrow());
                            let aa = if let Ok(aa) = list.clone().borrow().car() { aa } else { break };
                            if !aa.borrow().is_holder() {
                                // stack_operators.push(cc.clone());
                                continuations.push(cont!(cc, env, level));
                                // stack_operators.push(aa.clone());
                                continuations.push(cont!(aa, env, level));
                                list.borrow_mut().set_car(SymbolTable::holder());
                                break;
                            }
                            list = list.clone().borrow().cdr()?;
                        }
                        if list.borrow().is_nil() {
                            let len = d.borrow().len();
                            let ret = stack_operands.pop().unwrap();
                            for _ in 0..len-1 {
                                stack_operands.pop().unwrap();
                            }
                            stack_operands.push(ret);
                        }
                    },
                    Datum::SpecialForm(SpecialForm::Quasiquote) => {
                        
                    },
                    Datum::SpecialForm(SpecialForm::If) => {
                        if !d.borrow().car()?.borrow().is_holder() {
                            // stack_operators.push(cc.clone());
                            continuations.push(cont!(cc, env, level));
                            // stack_operators.push(d.borrow().car()?);
                            continuations.push(cont!(d.borrow().car()?, env, level));
                            d.borrow_mut().set_car(SymbolTable::holder());
                        } else {
                            let test = stack_operands.pop().unwrap();
                            if test.borrow().is_true() {
                                // stack_operators.push(d.borrow().cdr()?.borrow().car()?);
                                continuations.push(cont!(d.borrow().cdr()?.borrow().car()?, env, level));
                            } else {
                                if let Ok(f_term) = d.borrow().cdr()?.borrow().cdr()?.borrow().car() {
                                    // stack_operators.push(f_term);
                                    continuations.push(cont!(f_term, env, level));
                                } else {
                                    // stack_operators.push(SymbolTable::unspecified());
                                    continuations.push(cont!(SymbolTable::unspecified(), env, level));
                                }
                            }
                        }
                    },
                    Datum::SpecialForm(SpecialForm::Define) => {
                        if !d.borrow().cadr()?.borrow().is_holder() {
                            // stack_operators.push(cc.clone());
                            let cc = List::clone(cc.clone());
                            continuations.push(cont!(cc, env, level));
                            // stack_operators.push(d.borrow().cadr()?);
                            continuations.push(cont!(d.borrow().cadr()?, env, level));
                            cc.borrow().cdr()?.borrow().cdr()?.borrow_mut().set_car(SymbolTable::holder());
                        } else {
                            let id = if let Datum::Symbol(ref id) = *d.borrow().car()?.borrow() { id.clone() } else { return Err(RuntimeError::new("symbol not specified in define")) };
                            let val = stack_operands.pop().unwrap();
                            env.borrow_mut().put(id, val);
                            stack_operands.push(d.borrow().car()?);
                        }
                    },
                    Datum::SpecialForm(SpecialForm::Lambda) => {
                        stack_operands.push(Datum::Lambda(LambdaExpression {
                            formals: d.borrow().car()?.clone(),
                            expr: d.borrow().cadr()?.clone(),
                            env: env.clone()
                        }).wrap());
                    },
                    Datum::SpecialForm(SpecialForm::SyntaxRules) => {
                        let literals = d.borrow().car()?;
                        let mut rules = List::new();
                        for rule in List::from(d.borrow().cdr()?) {
                            if let ListItem::Item(rule) = rule {
                                let tf = Datum::TransformerSpec {
                                    pattern: rule.borrow().car()?,
                                    template: rule.borrow().cadr()?
                                }.wrap();
                                rules = rules.chain(iter::once(ListItem::Item(tf))).collect();
                            }
                        }
                        stack_operands.push(Datum::Syntax {
                            literals: literals,
                            rules: rules.into()
                        }.wrap());
                    },
                    Datum::SpecialForm(SpecialForm::DefineSyntax) => {
                        if !d.borrow().cadr()?.borrow().is_holder() {
                            // stack_operators.push(cc.clone());
                            let cc = List::clone(cc.clone());
                            continuations.push(cont!(cc, env, level));
                            // stack_operators.push(d.borrow().cadr()?);
                            continuations.push(cont!(d.borrow().cadr()?, env, level));
                            cc.borrow().cdr()?.borrow().cdr()?.borrow_mut().set_car(SymbolTable::holder());
                        } else {
                            let id = if let Datum::Symbol(ref id) = *d.borrow().car()?.borrow() { id.clone() } else { return Err(RuntimeError::new("symbol not specified in define")) };
                            let val = stack_operands.pop().unwrap();
                            env.borrow_mut().put(id, val);
                            stack_operands.push(d.borrow().car()?);
                        }
                    },          
                    Datum::Syntax { ref literals, ref rules } => {
                        let mut flag = false;
                        for tf in List::from(rules) {
                            if let ListItem::Item(val) = tf {
                                if let Datum::TransformerSpec { ref pattern, ref template } = *val.borrow() {
                                    if check_syntax_rule(pattern.clone(), d.clone(), literals.clone())? {
                                        let env = Environment::forward(env.clone());
                                        eval_pattern_match(pattern.clone(), d.clone(), env.clone());
                                        continuations.push(cont!(template, env, level));
                                        flag = true;
                                        break;
                                    }
                                }
                            }
                        }
                        if !flag {
                            Err(RuntimeError::new(format!("unexpected syntax pattern {:?}", a.borrow())))?
                        }
                    },
                    Datum::Symbol(_) | Datum::Pair(_, _) => {
                        let operator = a.clone();
                        // *a = SymbolTable::holder();
                        // stack_operators.push(cc.clone());
                        let cc = List::clone(cc.clone());
                        cc.borrow_mut().set_car(SymbolTable::holder());
                        continuations.push(cont!(cc, env, level));
                        // stack_operators.push(operator.clone());
                        continuations.push(cont!(operator, env, level));
                    },
                    Datum::Holder => {
                        let operator = stack_operands.pop().unwrap();
                        match *operator.clone().borrow() {
                            Datum::Builtin(_) | Datum::Lambda(_) => {
                                // stack_operators.push(cc.clone());
                                let cc = List::clone(cc.clone());
                                cc.borrow_mut().set_car(operator.clone());
                                continuations.push(cont!(cc, env, level));
                                // *a = operator.clone();

                                let mut list = List::new();
                                for li in List::from(d.clone()) {
                                    if let ListItem::Item(val) = li {
                                        list = list.chain(iter::once(ListItem::Item(SymbolTable::holder()))).collect();
                                        continuations.push(cont!(val, env, level));
                                    } else if let ListItem::Ellipsis(val) = li {
                                        list = list.chain(iter::once(ListItem::Ellipsis(SymbolTable::holder()))).collect();
                                        continuations.push(cont!(val, env, level));
                                    }
                                }
                                cc.borrow_mut().set_cdr(list.into())?;
                                // let mut list = d.clone();
                                // loop {
                                //     let aa = if let Ok(aa) = list.clone().borrow().car() { aa } else { break };
                                //     // stack_operators.push(aa.clone());
                                //     continuations.push(cont!(aa, env));
                                //     list.borrow_mut().set_car(SymbolTable::holder());
                                //     list = list.clone().borrow().cdr()?;
                                // }
                            },
                            Datum::SpecialForm(_) | Datum::Syntax { .. } => {
                                // *a = operator.clone();
                                // stack_operators.push(cc.clone());
                                let cc = List::clone(cc.clone());
                                cc.borrow_mut().set_car(operator.clone())?;
                                continuations.push(cont!(cc, env, level));
                            },
                            _ => {
                                panic!("TODO")
                            }
                        }
                    },
                    Datum::Builtin(ref func) => {
                        let len = d.borrow().len();
                        let mut operands = List::new();
                        for i in 0..len {
                            operands = operands.chain(iter::once(ListItem::Item(stack_operands.pop().unwrap()))).collect();
                        }
                        stack_operands.push(func(operands.into())?);
                    },
                    Datum::Lambda(LambdaExpression { ref formals, ref expr, env: ref lambda_env }) => {
                        let expr = List::clone(expr.clone());
                        let len = d.borrow().len();
                        let mut operands = List::new();
                        for i in 0..len {
                            operands = operands.chain(iter::once(ListItem::Item(stack_operands.pop().unwrap()))).collect();
                        }
                        // Extend environment
                        let env = Environment::forward_with_name(lambda_env.clone(), format!("{:?}", a.borrow()));
                        // if DEBUG!() { println!("Evaling lambda: params: {:?}, expr: {:?}\n\t{:?}", formals.borrow(), expr.borrow(), lambda_env.borrow()); }
                        eval_pattern_match(formals.clone(), operands.into(), env.clone()).map_err(|_| RuntimeError::new("precedure params not match"))?;
                        // eval_begin(expr.clone(), lambda_env.clone())
                      
                        // stack_operators.push(expr.clone());
                        continuations.push(cont!(expr, env, level));
                    },
                    _ => {
                        Err(RuntimeError::new(format!("{:?} is not applicable", a.borrow())))?
                    }
                }
            },
            Datum::Symbol(ref id) => {
                stack_operands.push(env.borrow().find(id)?);
            },
            _ => {
                stack_operands.push(cc.clone());                
            }
        }
        println!("Loop : {:?}, {:?}", continuations, stack_operands);
    }

    stack_operands.pop().ok_or(RuntimeError::new("ice: unexpected return value"))

}


pub fn eval(term: Value, env: Env) -> Result<Value, RuntimeError> {
    let ret = eval_inner(term.clone(), env.clone())?;
    if DEBUG!() { println!("Evaluating {:?} \n\t\t{:?} \n\t=>\t{:?}", env.borrow(), term.borrow(), ret.borrow()); }
    Ok(ret)
}
pub fn eval_inner(term: Value, env: Env) -> Result<Value, RuntimeError> {
    match *term.borrow() {
        Datum::Symbol(ref s) => {
            env.borrow().find(&s)
        },
        // Datum::SpecialForm(sf) => Ok(Datum::SpecialForm(*sf)),
        Datum::Abbreviation(AbbrevPrefix::Quote, ref val) => Ok(val.clone()),
        Datum::Abbreviation(AbbrevPrefix::Quasiquote, ref val) => eval_quasiquote(val.clone(), env.clone()),
        Datum::Abbreviation(AbbrevPrefix::Unquote, _) | Datum::Abbreviation(AbbrevPrefix::UnquoteSplicing, _) => Err(RuntimeError::new("unexpected unquote outside quasiquote")),
        Datum::Pair(ref a, ref d) => {
            let operator = eval(a.clone(), env.clone())?;
            match *operator.clone().borrow() {
                // Special Forms

                Datum::SpecialForm(SpecialForm::Apply) => {
                    let expr = Datum::Pair(d.borrow().car()?, eval(d.borrow().cdr()?.borrow().car()?, env.clone())?).wrap();
                    eval(expr, env.clone())
                },
        //         Datum::SpecialForm(SpecialForm::Eval) => {
        //             let ret = eval(&items[1], env)?;
        //             eval(&ret, env)
        //         },
                Datum::SpecialForm(SpecialForm::Begin) => {
                    eval_begin(d.clone(), Environment::forward_with_name(env.clone(), "begin"))
                },
                Datum::SpecialForm(SpecialForm::Define) => {
                    if let Datum::Symbol(ref id) = *d.borrow().car()?.borrow() {
                        let value = eval_begin(d.borrow().cdr()?, env.clone())?;
                        env.borrow_mut().put(id.clone(), value);
                        Ok(d.borrow().car()?)
                    } else if let Datum::Pair(ref ad, ref dd) = *d.borrow().car()?.borrow() {
                        let id = if let Datum::Symbol(ref id) = *ad.clone().borrow() { id.clone() } else { return Err(RuntimeError::new("precedure name not specified in define")) };
                        let lambda = Datum::Lambda(LambdaExpression {
                            formals: dd.clone(),
                            expr: d.borrow().cdr()?.clone(),
                            env: env.clone()
                        }).wrap();
                        env.borrow_mut().put(id.clone(), lambda);
                        Ok(ad.clone())
                    } else {
                        Err(RuntimeError::new("unknown syntax for define"))
                    }
                },
        //         Datum::SpecialForm(SpecialForm::DefineSyntax) => {
        //             let id = if let Datum::Symbol(id) = &items[1] { id.clone() } else { panic!("Define syntax error") };
        //             let formals: Vec<String> = if let Datum::List(l) = &items[2] {
        //                 l.clone().into_iter().map(|x| if let Datum::Symbol(id) = x { id } else { panic!("Not formal") }).collect()
        //             } else { panic!("Not formal") };
        //             let syntax = Datum::Syntax(SyntaxRule {
        //                 formals: formals,
        //                 expr: Box::new(Datum::List(
        //                     vec![Datum::SpecialForm(SpecialForm::Begin)]
        //                         .into_iter()
        //                         .chain(items.iter().skip(3).map(|x| x.clone()))
        //                         .collect()
        //                     ))
        //             });
        //             env.put(id, syntax);
        //             Ok(Datum::new())
        //         },
                Datum::SpecialForm(SpecialForm::Set) => {
                    if let Datum::Symbol(ref id) = *d.borrow().car()?.borrow() {
                        let value = eval(d.borrow().cadr()?, env.clone())?;
                        env.borrow_mut().set(id, value)
                    } else {
                        Err(RuntimeError::new("variable not specified in set!"))
                    }
                },
                Datum::SpecialForm(SpecialForm::SetCar) => {
                    let list = eval(d.borrow().car()?, env.clone())?;
                    let value = eval(d.borrow().cadr()?, env.clone())?;
                    let pair = list.replace(Datum::Nil);
                    if let Datum::Pair(a, d) = pair {
                        list.replace(Datum::Pair(value, d));
                        Ok(SymbolTable::unspecified())
                    } else {
                        list.replace(pair);
                        Err(RuntimeError::new("variable not pair in set-car!"))
                    }
                },
                Datum::SpecialForm(SpecialForm::SetCdr) => {
                    let list = eval(d.borrow().car()?, env.clone())?;
                    let value = eval(d.borrow().cadr()?, env.clone())?;
                    let pair = list.replace(Datum::Nil);
                    if let Datum::Pair(a, d) = pair {
                        list.replace(Datum::Pair(a, value));
                        Ok(SymbolTable::unspecified())
                    } else {
                        list.replace(pair);
                        Err(RuntimeError::new(format!("variable not pair in set-cdr! : {:?}", list.borrow())))
                    }
                },         
                Datum::SpecialForm(SpecialForm::Lambda) => {
                    Ok(Datum::Lambda(LambdaExpression {
                        formals: d.borrow().car()?.clone(),
                        expr: d.borrow().cdr()?.clone(),
                        env: env.clone()
                    }).wrap())
                },
                Datum::SpecialForm(SpecialForm::And) => {
                    eval_and(d.clone(), env.clone())
                },
                Datum::SpecialForm(SpecialForm::Or) => {
                    eval_or(d.clone(), env.clone())
                },
                Datum::SpecialForm(SpecialForm::Cond) => {
                    let mut formals = d.clone();
                    while let Datum::Pair(ref ad, ref dd) = *formals.clone().borrow().car()?.borrow() {
                        let test = eval(ad.clone(), env.clone())?;
                        if test.borrow().is_true() {
                            return eval_begin(dd.clone(), env.clone());
                        } else if let Datum::SpecialForm(SpecialForm::Else) = *test.clone().borrow() {
                            return eval_begin(dd.clone(), env.clone());
                        } else {
                            formals = formals.clone().borrow().cdr()?;
                            if let Datum::Nil = *formals.borrow() { break; }
                        }
                    }
                    Err(RuntimeError::new("cond not exhausted"))
                },
                Datum::SpecialForm(SpecialForm::If) => {
                    if eval(d.borrow().car()?, env.clone())?.borrow().is_true() {
                        eval(d.borrow().cdr()?.borrow().car()?, env.clone())
                    } else if let Ok(ref false_term) = d.borrow().cdr()?.borrow().cdr()?.borrow().car() {
                        eval(false_term.clone(), env.clone())
                    } else {
                        Ok(SymbolTable::unspecified())
                    }
                },
                Datum::SpecialForm(SpecialForm::Quote) => {
                    Ok(d.clone())
                },
                // Datum::SpecialForm(SpecialForm::Unquote) => {
                //     Ok(d.clone())
                // },
                Datum::SpecialForm(SpecialForm::Letstar) | Datum::SpecialForm(SpecialForm::Letrec) => {
                    let env = Environment::forward(env.clone());
                    let mut formals = d.borrow().car()?;
                    while let Datum::Pair(ref ad, ref dd) = *formals.clone().borrow().car()?.borrow() {
                        let id = if let Datum::Symbol(ref id) = *ad.clone().borrow() { id.clone() } else { return Err(RuntimeError::new("formal name not specified in let")) };
                        let val = if let Datum::Nil = *dd.borrow() { SymbolTable::unspecified() } else { eval(dd.borrow().car()?, env.clone())? };
                        env.borrow_mut().put(id, val);
                        formals = formals.clone().borrow().cdr()?;
                        if let Datum::Nil = *formals.borrow() { break; }
                    }
                    eval_begin(d.borrow().cdr()?, env.clone())
                },
                Datum::SpecialForm(SpecialForm::Let) => {
                    let let_env = Environment::forward(env.clone());
                    let mut formals = d.borrow().car()?;
                    while let Datum::Pair(ref ad, ref dd) = *formals.clone().borrow().car()?.borrow() {
                        let id = if let Datum::Symbol(ref id) = *ad.clone().borrow() { id.clone() } else { return Err(RuntimeError::new("formal name not specified in let")) };
                        let val = if let Datum::Nil = *dd.borrow() { SymbolTable::unspecified() } else { eval(dd.borrow().car()?, env.clone())? };
                        let_env.borrow_mut().put(id, val);
                        formals = formals.clone().borrow().cdr()?;
                        if let Datum::Nil = *formals.borrow() { break; }
                    }
                    eval_begin(d.borrow().cdr()?, let_env.clone())
                },
        //         // Precudure Call
                Datum::Lambda(LambdaExpression { ref formals, ref expr, env: ref lambda_env }) => {
                    let operands = eval_list(d.clone(), env.clone())?;
                    let lambda_env = Environment::forward_with_name(lambda_env.clone(), format!("{:?}", operator.borrow()));
                    if DEBUG!() { println!("Evaling lambda: params: {:?}, expr: {:?}\n\t{:?}", formals.borrow(), expr.borrow(), lambda_env.borrow()); }
                    eval_pattern_match(formals.clone(), operands, lambda_env.clone()).map_err(|_| RuntimeError::new("precedure params not match"))?;
                    eval_begin(expr.clone(), lambda_env.clone())
                },
        //         Datum::Syntax(SyntaxRule { ref formals, ref expr }) => {
        //             let operands: Vec<Datum> = items.iter().skip(1).map(|x| x.clone()).collect();
        //             Ok(enter_env!(env, {
        //                 for (f, d) in formals.iter().zip(operands.iter()) {
        //                     env.put(f.clone(), d.clone());
        //                 }
        //                 let ret = eval(expr, env)?;
        //                 eval(&ret, env)?
        //             }))
        //         },
                Datum::Builtin(ref func) => {
                    let operands = eval_list(d.clone(), env.clone())?;
                    func(operands)
                },
                _ => {
                    Err(RuntimeError::new(format!("{:?} is not applicable", operator.borrow())))
                }
            }
        },
        _ => Ok(term.clone())
    }
}

pub fn eval_list(term: Value, env: Env) -> Result<Value, RuntimeError> {
    let mut ret = List::new();
    let mut list = List::from(term);
    while let Some(next) = list.next() {
        if let ListItem::Item(x) = next {
            ret = ret.chain(iter::once(ListItem::Item(eval(x, env.clone())?))).collect();
        } else if let ListItem::Ellipsis(x) = next {
            ret = ret.chain(iter::once(ListItem::Ellipsis(eval(x, env.clone())?))).collect();
        }
    }
    Ok(ret.into())
}

pub fn eval_begin(term: Value, env: Env) -> Result<Value, RuntimeError> {
    let mut ret = SymbolTable::unspecified();
    let mut list = List::from(term);
    while let Some(next) = list.next() {
        if let ListItem::Item(x) = next {
            ret = eval(x, env.clone())?;
        } else {
            Err(RuntimeError::new("Unexpected form in begin"))?
        }
    }
    Ok(ret)
}

pub fn eval_and(term: Value, env: Env) -> Result<Value, RuntimeError> {
    let mut ret = SymbolTable::bool(true);
    let mut list = List::from(term);
    while let Some(next) = list.next() {
        if let ListItem::Item(x) = next {
            ret = eval(x, env.clone())?;            
            if ret.borrow().is_false() {
                break;
            }
        } else {
            Err(RuntimeError::new("Unexpected form in and"))?
        }
    }
    Ok(ret)
}

pub fn eval_or(term: Value, env: Env) -> Result<Value, RuntimeError> {
    let mut ret = SymbolTable::bool(false);
    let mut list = List::from(term);
    while let Some(next) = list.next() {
        if let ListItem::Item(x) = next {
            ret = eval(x.clone(), env.clone())?;
            if ret.borrow().is_true() {
                break;
            }
        } else {
            Err(RuntimeError::new("Unexpected form in or"))?
        }
    }
    Ok(ret)
}

pub fn eval_pattern_match(pattern: Value, params: Value, env: Env) -> Result<(), RuntimeError> {
    match *pattern.borrow() {
        Datum::Symbol(ref id) => {
            if DEBUG!() { println!("Pattern matching: Binding {:} to {:?}", id, params.borrow()); }
            env.borrow_mut().put(id.clone(), params.clone());
            // println!("Env is {:?}", env);
            Ok(())
        },
        Datum::Pair(ref a, ref d) => {
            eval_pattern_match(a.clone(), params.borrow().car()?.clone(), env.clone())?;
            eval_pattern_match(d.clone(), params.borrow().cdr()?.clone(), env.clone())?;
            Ok(())
        },
        Datum::Nil => {
            Ok(())
        }
        _ => {
            Err(RuntimeError::new("Unexpected pattern matching"))
        }
    }
}

pub fn eval_quasiquote(term: Value, env: Env) -> Result<Value, RuntimeError> {
    let mut ret = List::new();
    let mut list = List::from(term);
    while let Some(next) = list.next() {
        if let ListItem::Item(x) = next {
            match *x.borrow() {
                Datum::Abbreviation(AbbrevPrefix::Unquote, ref val) => {
                    ret = ret.chain(iter::once(ListItem::Item(
                        eval(val.clone(), env.clone())?
                    ))).collect();
                },
                Datum::Abbreviation(AbbrevPrefix::UnquoteSplicing, ref val) => {
                    ret = ret.chain(List::from(
                        eval(val.clone(), env.clone())?
                    )).collect();
                },
                _ => {
                    // if let Datum::Pair(ref a, ref d) = *x.borrow() {
                    //     if let Datum::Symbol(ref id) = *a.borrow() {
                    //         if id == "unquote" {
                    //             ret = ret.chain(iter::once(ListItem::Item(
                    //                 eval(d.clone(), env.clone())?
                    //             ))).collect();
                    //             continue;
                    //         } else if id == "unquote-splicing" {
                    //             ret = ret.chain(List::from(
                    //                 eval(d.clone(), env.clone())?
                    //             )).collect();
                    //             continue;
                    //         }
                    //     }
                    // }
                    ret = ret.chain(iter::once(ListItem::Item(
                        x.clone()
                    ))).collect();
                }
            }
        } else if let ListItem::Ellipsis(x) = next {
            match *x.borrow() {
                Datum::Abbreviation(AbbrevPrefix::Unquote, ref val) => {
                    ret = ret.chain(iter::once(ListItem::Ellipsis(
                        eval(val.clone(), env.clone())?
                    ))).collect();
                },
                Datum::Abbreviation(AbbrevPrefix::UnquoteSplicing, ref val) => {
                    Err(RuntimeError::new(",@ in unexpected context"))?
                },
                _ => {
                    ret = ret.chain(iter::once(ListItem::Ellipsis(
                        x.clone()
                    ))).collect();
                }
            }
        }
    }
    Ok(ret.into())
}

pub fn check_syntax_rule(pattern: Value, params: Value, literals: Value) -> Result<bool, RuntimeError> {
    match *pattern.borrow() {
        Datum::Symbol(ref id) => {
            // env.borrow_mut().put(id.clone(), params.clone());
            // println!("Env is {:?}", env);
            // find if literals contains id, then params must be params
            for r in List::from(literals) {
                if let ListItem::Item(val) = r {
                    if let Datum::Symbol(ref r) = *val.borrow() {
                        if r == id {
                            if let Datum::Symbol(ref rr) = *params.borrow() {
                                return Ok(id == rr);
                            } else {
                                return Ok(false);
                            }
                        }
                    }
                }
            }
            Ok(true)
        },
        Datum::Pair(ref a, ref d) => {
            Ok(check_syntax_rule(a.clone(), params.borrow().car()?.clone(), literals.clone())?
            && check_syntax_rule(d.clone(), params.borrow().cdr()?.clone(), literals.clone())?)
        },
        Datum::Nil => {
            Ok(params.borrow().is_nil())
        }
        _ => {
            Ok(false)
            // Err(RuntimeError::new("Unexpected pattern matching"))
        }
    }
}

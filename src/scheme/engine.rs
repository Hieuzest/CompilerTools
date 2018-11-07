use super::beam::*;
use super::env::*;
use super::symbol::*;
use std::iter;
use crate::utils::*;


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
        // Datum::Abbreviation(AbbrevPrefix::Template, d) => {
        //     if let Datum::List(l) = d.as_ref() {
        //         Ok(Datum::List(l.iter().flat_map(|x| {match x {
        //             Datum::Abbreviation(AbbrevPrefix::Comma, dd) => {
        //                 vec![eval(dd, env).expect(&format!("Error occured when eval {:?}", dd))]
        //             },
        //             Datum::Abbreviation(AbbrevPrefix::CommaSplicing, dd) => {
        //                 if let Datum::List(l) = eval(dd, env).expect(&format!("Error occured when eval {:?}", dd)) {
        //                     l.iter().map(|xx| eval(xx, env).expect(&format!("Error occured when eval {:?}", xx))).collect::<Vec<Datum>>()
        //                 } else {
        //                     panic!("Comma splicing on non-list")
        //                 }
        //             },
        //             Datum::Abbreviation(AbbrevPrefix::Template, _) => vec![eval(&x, env).expect(&format!("Error occured when eval {:?}", x))],
        //             _ => vec![x.clone()]
        //         }}).collect()))
        //     } else {
        //         Ok(*d.clone())
        //     }
        // },
        // Datum::Abbreviation(AbbrevPrefix::Comma, _) | Datum::Abbreviation(AbbrevPrefix::CommaSplicing, _) => Err(RuntimeError::new(format!("Unquote : {:?}", term))) ,
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
                        Err(RuntimeError::new("variable not pair in set-cdr!"))
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
                            return eval(dd.borrow().car()?, env.clone());
                        } else if let Datum::SpecialForm(SpecialForm::Else) = *test.clone().borrow() {
                            return eval(dd.borrow().car()?, env.clone());
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
    // println!("Eval list: {:?}", term);
    // match *term.borrow() {
    //     Datum::Symbol(ref id) => {
    //         env.borrow().find(id)
    //     },
    //     Datum::Pair(ref a, ref d) => {
    //         Ok(Datum::Pair(eval(a.clone(), env.clone())?, eval_list(d.clone(), env.clone())?).wrap())
    //     },
    //     _ => Ok(term.clone())
    // }
    let mut ret = SymbolTable::nil();
    let mut last = SymbolTable::nil();
    let mut list = List::from(term);
    while let Some(next) = list.next() {
        if let ListItem::Item(x) = next {
            if ret.clone().borrow().is_nil() {
                let v = eval(x, env.clone())?;
                ret = Datum::Pair(v, SymbolTable::nil()).wrap();
                last = ret.clone();
            } else {
                let v = eval(x, env.clone())?;
                let d = Datum::Pair(v, SymbolTable::nil()).wrap();
                last.borrow_mut().set_cdr(d.clone());
                last = d;
            }
        } else if let ListItem::Ellipsis(x) = next {
            let v = eval(x, env.clone())?;
            last.borrow_mut().set_cdr(v);
        }
    }
    Ok(ret)
}

pub fn eval_begin(term: Value, env: Env) -> Result<Value, RuntimeError> {
    // println!("Eval begin: {:?}", term);
    // match *term.borrow() {
    //     Datum::Symbol(ref id) => {
    //         env.borrow().find(id)
    //     },
    //     Datum::Pair(ref a, ref d) => {
    //         let a = eval(a.clone(), env.clone())?;
    //         if let Datum::Nil = *d.clone().borrow() { Ok(a) } else { eval_begin(d.clone(), env.clone()) }
    //     },
    //     _ => Ok(term.clone())
    // }
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
    // match *term.borrow() {
    //     Datum::Symbol(ref id) => {
    //         if let Datum::Boolean(ref b) = *env.borrow().find(id)?.borrow() {
    //             Ok(SymbolTable::bool(*b))
    //         } else { Err(RuntimeError::new("Non-bool expr in and")) }
    //     },
    //     Datum::Pair(ref a, ref d) => {
    //         if eval(a.clone(), env.clone())?.borrow().is_true() {
    //             eval_and(d.clone(), env.clone())
    //         } else {
    //             Ok(SymbolTable::bool(false))
    //         }
    //     },
    //     Datum::Nil => {
    //         Ok(SymbolTable::bool(true))
    //     },
    //     _ => Err(RuntimeError::new("Non-bool expr in and"))
    // }
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

    // match *term.borrow() {
    //     Datum::Symbol(ref id) => {
    //         if let Datum::Boolean(ref b) = *env.borrow().find(id)?.borrow() {
    //             Ok(SymbolTable::bool(*b))
    //         } else { Err(RuntimeError::new("Non-bool expr in and")) }
    //     },
    //     Datum::Pair(ref a, ref d) => {
    //         if eval(a.clone(), env.clone())?.borrow().is_true() {
    //             Ok(SymbolTable::bool(true))
    //         } else {
    //             eval_or(d.clone(), env.clone())                
    //         }
    //     },
    //     Datum::Nil => {
    //         Ok(SymbolTable::bool(false))
    //     },
    //     _ => Err(RuntimeError::new("Non-bool expr in or"))
    // }
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
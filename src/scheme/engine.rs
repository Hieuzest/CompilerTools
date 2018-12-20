use super::beam::*;
use super::env::*;
use super::symbol::*;
use std::iter;
use std::rc::*;
use crate::utils::*;

macro_rules! DATUM {
    ($expr: expr) => {
        *$expr.clone().borrow()
    };
}

macro_rules! CONT {
    ($cont: expr) => {
        Box::new($cont.clone())
    };
}

pub fn eval(src: Value, env: Env) -> Result<Value, RuntimeError> {
    let mut cont = Continuation::Return;
    let mut expr = SymbolTable::nil();
    cont = Continuation::EvaluateBegin(src, env.clone(), CONT!(cont));
    'outer: loop {
        if VERBOSE!() { println!("Ref: {} Step: {:?} \n\t Cont {:?}", Rc::strong_count(&expr), expr.borrow(), cont); }

        match cont.clone() {
            Continuation::Return => {
                return Ok(expr);
            },
            // Continuation::EvaluateCallCC(ref cont_) => {
            //     cont = Continuation::EvaluateApply(expr.clone(), cont_.clone());
            //     expr = List::one(Datum::Continuation(*cont_.clone()).wrap()).into();
            // },
            Continuation::EvaluateExpression(ref expr_, ref env, ref level, ref cont_) => {
                cont = *cont_.clone();
                match *expr_.clone().borrow() {
                    Datum::Symbol(ref id) if *level == 0 => {
                        expr = env.borrow().find(id).map_err(|e| if env.borrow().find_syntax(id).is_ok() { RuntimeError::new(format!("Keyword cannot be used as expression : {:?}", id)) } else { e })?;
                    },
                    Datum::Pair(ref a, ref d) if *level == 0 => {
                        cont = Continuation::EvaluateList(expr_.clone(), env.clone(), CONT!(cont));
                        cont = Continuation::EvaluateOperator(a.clone(), env.clone(), CONT!(cont));
                        continue 'outer;
                    },
                    Datum::Pair(ref a, ref d) => {
                        cont = Continuation::QuasiquoteList(List::new().into(), expr_.borrow().cdr()?, env.clone(), *level, CONT!(cont));
                        cont = Continuation::EvaluateExpression(a.clone(), env.clone(), *level, CONT!(cont));
                    },
                    _ => {
                        expr = expr_.clone();
                    }
                }
            },
            Continuation::EvaluateOperator(ref expr_, ref env, ref cont_) => {
                cont = *cont_.clone();
                match *expr_.borrow() {
                    Datum::Symbol(ref id) => {
                        expr = env.borrow().find_syntax(id).or(env.borrow().find(id))?;
                    },
                    Datum::Pair(ref a, ref d) => {
                        cont = Continuation::EvaluateList(expr_.clone(), env.clone(), CONT!(cont));
                        cont = Continuation::EvaluateOperator(a.clone(), env.clone(), CONT!(cont));
                        continue 'outer;
                    }
                    _ => {
                        error!("Unsupported operator : {:?}", expr_.borrow())?
                    }
                }
            },
            Continuation::EvaluateList(ref expr_, ref env, ref cont_) => {
                cont = *cont_.clone();

                match *expr.clone().borrow() {
                    Datum::Builtin(_) | Datum::BuiltinExt(_) | Datum::Lambda(_) => {
                        cont = Continuation::EvaluateApply(expr.clone(), CONT!(cont));
                        cont = Continuation::EvaluateProcedure(List::new().into(), expr_.borrow().cdr()?.clone(), env.clone(), CONT!(cont));
                    },
                    Datum::Continuation(ref cont_) => {
                        cont = cont_.clone();
                        cont = Continuation::EvaluateExpression(expr_.borrow().cadr()?, env.clone(), 0, CONT!(cont));
                    },
                    Datum::SpecialForm(SpecialForm::Begin) => {
                        expr = SymbolTable::nil();
                        if let Continuation::EvaluateBegin(ref e, _, ref c) = cont {
                            if e.borrow().is_nil() {
                                cont = Continuation::EvaluateBegin(expr_.borrow().cdr()?.clone(), env.clone(), c.clone());
                            } else {
                                cont = Continuation::EvaluateBegin(expr_.borrow().cdr()?.clone(), env.clone(), CONT!(cont));
                            }
                        } else {
                            cont = Continuation::EvaluateBegin(expr_.borrow().cdr()?.clone(), env.clone(), CONT!(cont));
                        }
                    },
                    Datum::SpecialForm(SpecialForm::Define) => {
                        cont = Continuation::EvaluateDefine(expr_.borrow().cadr()?.clone(), env.clone(), CONT!(cont));
                        cont = Continuation::EvaluateExpression(expr_.borrow().cdr()?.borrow().cadr()?, env.clone(), 0, CONT!(cont));

                    },
                    Datum::SpecialForm(SpecialForm::DefineSyntax) => {
                        cont = Continuation::EvaluateDefineSyntax(expr_.borrow().cadr()?.clone(), env.clone(), CONT!(cont));
                        cont = Continuation::EvaluateOperator(expr_.borrow().cdr()?.borrow().cadr()?, env.clone(), CONT!(cont));

                    },
                    Datum::SpecialForm(SpecialForm::If) => {
                        cont = Continuation::EvaluateIf(expr_.borrow().cdr()?.clone(), env.clone(), CONT!(cont));
                        cont = Continuation::EvaluateExpression(expr_.borrow().cadr()?, env.clone(), 0, CONT!(cont));
                    },
                    Datum::SpecialForm(SpecialForm::Set) => {
                        cont = Continuation::EvaluateSet(expr_.borrow().cadr()?.clone(), env.clone(), CONT!(cont));
                        cont = Continuation::EvaluateExpression(expr_.borrow().cdr()?.borrow().cadr()?, env.clone(), 0, CONT!(cont));
                    },
                    Datum::SpecialForm(SpecialForm::SetSyntax) => {
                        cont = Continuation::EvaluateSetSyntax(expr_.borrow().cadr()?.clone(), env.clone(), CONT!(cont));
                        cont = Continuation::EvaluateOperator(expr_.borrow().cdr()?.borrow().cadr()?, env.clone(), CONT!(cont));
                    },
                    Datum::SpecialForm(SpecialForm::SyntaxRules) => {
                        expr = Datum::Syntax(SyntaxRules {
                            literals: expr_.borrow().cadr()?,
                            rules: expr_.borrow().cdr()?.borrow().cdr()?,
                            env: env.clone(),
                        }).wrap();
                    },
                    Datum::SpecialForm(SpecialForm::Lambda) => {
                        expr = Datum::Lambda(LambdaExpression {
                            formals: expr_.borrow().cadr()?,
                            expr: expr_.borrow().cdr()?.borrow().cadr()?,
                            env: env.clone()
                        }).wrap();
                    },
                    Datum::SpecialForm(SpecialForm::CurrEnv) => {
                        expr = Datum::Environment(env.clone()).wrap();
                    },
                    Datum::SpecialForm(SpecialForm::Quote) => {
                        expr = expr_.borrow().cadr()?;
                    },
                    Datum::SpecialForm(SpecialForm::Quasiquote) => {
                        cont = Continuation::EvaluateExpression(expr_.borrow().cadr()?, env.clone(), 1, CONT!(cont));
                    },
                    Datum::SpecialForm(SpecialForm::Unquote) => {
                        error!("Unexpected unquote")?
                    },
                    Datum::SpecialForm(SpecialForm::UnquoteSplicing) => {
                        error!("Unexpected unquote-splicing")?
                    },
                    Datum::SpecialForm(SpecialForm::CurrEnv) => {
                        expr = Datum::Environment(env.clone()).wrap();
                    },
                    Datum::SpecialForm(SpecialForm::StandardEnv) => {
                        expr = Datum::Environment(Environment::new()).wrap();
                    },
                    Datum::Syntax(SyntaxRules { ref literals, ref rules, env: ref env_ }) => {
                        let rules = List::from(rules.clone());
                        let mut flag_ok = false;
                        for li in rules {
                            if let ListItem::Item(rule) = li {
                                if let Ok(true) = check_syntax_rule(rule.borrow().car()?.borrow().cdr()?, expr_.borrow().cdr()?, literals.clone()) {
                                    // println!("Syntax checked : {:?} {:?} {:?}", rule.borrow(), rule.borrow().car()?.borrow().cdr()?, expr_.borrow().cdr()?);
                                    flag_ok = true;
                                    let null_env = Environment::null();
                                    eval_pattern_match(rule.borrow().car()?.borrow().cdr()?, expr_.borrow().cdr()?, null_env.clone())?;
                                    cont = Continuation::EvaluateExpression(eval_template(rule.borrow().cadr()?, null_env.clone())?, env.clone(), 0, CONT!(cont));
                                    break;
                                }
                            }
                        }
                        if !flag_ok { error!("syntex not matched : {:?}", expr.borrow())? }
                    },
                    _ => {
                        error!("{:?} is not applicable", expr_.borrow())?
                    }
                }
            },
            Continuation::EvaluateApply(ref expr_, ref cont_) => {
                cont = *cont_.clone();
                match *expr_.clone().borrow() {
                    Datum::Builtin(ref func) => {
                        expr = func(expr.clone())?;
                    },
                    Datum::Lambda(LambdaExpression { ref formals, expr: ref expr_, env: ref env_ }) => {
                        let env = Environment::forward(env_.clone());
                        eval_pattern_match(formals.clone(), expr.clone(), env.clone())?;
                        cont = Continuation::EvaluateExpression(expr_.clone(), env.clone(), 0, CONT!(cont));
                    },
                    Datum::BuiltinExt(SpecialProcedure::Apply) => {
                        cont = Continuation::EvaluateApply(expr.borrow().car()?, CONT!(cont));
                        expr = expr.clone().borrow().cadr()?;
                    },
                    Datum::BuiltinExt(SpecialProcedure::Eval) => {
                        if let Datum::Environment(ref env) = *expr.borrow().cadr()?.borrow() {
                            cont = Continuation::EvaluateExpression(expr.borrow().car()?, env.clone(), 0, CONT!(cont));
                        } else {
                            error!("Expected environment in eval")?
                        }
                    },
                    Datum::BuiltinExt(SpecialProcedure::CallCC) => {
                        let e = Datum::Continuation(cont.clone()).wrap();
                        cont = Continuation::EvaluateApply(expr.borrow().car()?, CONT!(cont));
                        expr = List::one(e).into();
                    },
                    // Datum::BuiltinExt(SpecialProcedure::CurrEnv) => {
                    //     // There is no env !
                    //     // expr = Datum::Environment(env.clone()).wrap();
                    // },
                    // catch eval, apply, call/cc ...
                    _ => {
                        error!("unapplicable passed to apply : {:?}", expr_.borrow())?
                    }
                }
            },
            Continuation::EvaluateProcedure(ref car, ref cdr, ref env, ref cont_) => {
                cont = *cont_.clone();
                if cdr.borrow().is_nil() {
                    let mut list = List::from(car);
                    list.extend(iter::once(ListItem::Item(expr.clone())));
                    list.next();
                    expr = list.into();
                } else {
                    let mut list = List::from(car);
                    list.extend(iter::once(ListItem::Item(expr.clone())));
                    if cdr.borrow().is_pair() {
                        cont = Continuation::EvaluateProcedure(list.into(), cdr.borrow().cdr()?.clone(), env.clone(), CONT!(cont));
                        cont = Continuation::EvaluateExpression(cdr.borrow().car()?.clone(), env.clone(), 0, CONT!(cont));
                    } else {
                        cont = Continuation::EvaluateProcedureSplicing(list.into(), SymbolTable::nil(), env.clone(), CONT!(cont));
                        cont = Continuation::EvaluateExpression(cdr.clone(), env.clone(), 0, CONT!(cont));

                    }
                }
            },
            Continuation::EvaluateProcedureSplicing(ref car, ref cdr, ref env, ref cont_) => {
                cont = *cont_.clone();
                let mut list = List::from(car);
                list.extend(iter::once(ListItem::Ellipsis(expr.clone())));
                list.next();
                expr = list.into();
            },
            Continuation::EvaluateBegin(ref expr_, ref env, ref cont_) => {
                cont = *cont_.clone();
                if !expr_.borrow().is_nil() {
                    cont = Continuation::EvaluateBegin(expr_.borrow().cdr()?.clone(), env.clone(), CONT!(cont));
                    cont = Continuation::EvaluateExpression(expr_.borrow().car()?.clone(), env.clone(), 0, CONT!(cont));
                }
            },
            Continuation::EvaluateSet(ref expr_, ref env, ref cont_) => {
                cont = *cont_.clone();
                if let Datum::Symbol(ref id) = *expr_.borrow() {
                    env.borrow_mut().set(id, expr.clone())?;
                } else {
                    error!("expected symbol in set : {:?}", expr_.borrow())?
                }
                expr = expr_.clone();
            },
            Continuation::EvaluateSetSyntax(ref expr_, ref env, ref cont_) => {
                cont = *cont_.clone();
                if let Datum::Symbol(ref id) = *expr_.borrow() {
                    env.borrow_mut().set_syntax(id, expr.clone())?;
                } else {
                    error!("expected symbol in set-syntax : {:?}", expr_.borrow())?
                }
                expr = expr_.clone();
            },
            Continuation::EvaluateDefine(ref expr_, ref env, ref cont_) => {
                cont = *cont_.clone();
                if let Datum::Symbol(ref id) = *expr_.borrow() {
                    env.borrow_mut().put(id.clone(), expr.clone());
                } else {
                    error!("expected symbol in define : {:?}", expr_.borrow())?
                }
                expr = expr_.clone();
            },
            Continuation::EvaluateDefineSyntax(ref expr_, ref env, ref cont_) => {
                cont = *cont_.clone();
                if let Datum::Symbol(ref id) = *expr_.borrow() {
                    env.borrow_mut().put_syntax(id.clone(), expr.clone());
                } else {
                    error!("expected symbol in define : {:?}", expr_.borrow())?
                }
                expr = expr_.clone();
            },
            Continuation::EvaluateIf(ref expr_, ref env, ref cont_) => {
                cont = *cont_.clone();
                if expr.clone().borrow().is_true() {
                    expr = expr_.borrow().cadr()?;
                } else {
                    if let Ok(e) = expr_.borrow().cdr()?.borrow().cadr() {
                        expr = e;
                    } else {
                        expr = SymbolTable::unspecified();
                    }
                }
                cont = Continuation::EvaluateExpression(expr.clone(), env.clone(), 0, CONT!(cont));
            },
            Continuation::QuasiquoteList(ref car, ref cdr, ref env, ref level, ref cont_) => {
                cont = *cont_.clone();
                if cdr.borrow().is_nil() {
                    let mut list = List::from(car);
                    list.extend(iter::once(ListItem::Item(expr.clone())));
                    expr = list.into();
                    continue 'outer;
                } else if cdr.borrow().is_pair() && cdr.borrow().len() == 1 {
                    if let Datum::Symbol(ref id) = *expr.clone().borrow() {
                        if let Ok(val) = env.borrow().find_syntax(id) {
                            match *val.borrow() {
                                Datum::SpecialForm(SpecialForm::Quasiquote) => {
                                    let mut list = List::from(car);
                                    list.extend(iter::once(ListItem::Item(expr.clone())));
                                    cont = Continuation::QuasiquoteList(list.into(), cdr.borrow().cdr()?.clone(), env.clone(), *level, CONT!(cont));
                                    cont = Continuation::EvaluateExpression(cdr.borrow().car()?.clone(), env.clone(), *level + 1, CONT!(cont));
                                    continue 'outer;
                                },
                                Datum::SpecialForm(SpecialForm::Unquote) => {
                                    if *level == 1 {
                                         cont = Continuation::QuasiquoteListSplicing(car.clone(), cdr.borrow().cdr()?.clone(), env.clone(), *level, CONT!(cont));
                                    } else {
                                        let mut list = List::from(car);
                                        list.extend(iter::once(ListItem::Item(expr.clone())));
                                        cont = Continuation::QuasiquoteList(list.into(), cdr.borrow().cdr()?.clone(), env.clone(), *level, CONT!(cont));
                                    }
                                    cont = Continuation::EvaluateExpression(cdr.borrow().car()?.clone(), env.clone(), *level - 1, CONT!(cont));
                                    continue 'outer;
                                },
                                Datum::SpecialForm(SpecialForm::UnquoteSplicing) => {
                                    if *level == 1 {
                                        cont = cont.splicing()?;
                                    } else {
                                        let mut list = List::from(car);
                                        list.extend(iter::once(ListItem::Item(expr.clone())));
                                        cont = Continuation::QuasiquoteList(list.into(), cdr.borrow().cdr()?.clone(), env.clone(), *level, CONT!(cont));
                                    }
                                    cont = Continuation::EvaluateExpression(cdr.borrow().car()?.clone(), env.clone(), *level - 1, CONT!(cont));
                                    continue 'outer;
                                },
                                _ => {}
                            }
                        }
                    }
                } else if !cdr.borrow().is_pair() {
                    let mut list = List::from(car);
                    list.extend(iter::once(ListItem::Item(expr.clone())));
                    cont = Continuation::QuasiquoteListSplicing(list.into(), SymbolTable::nil(), env.clone(), *level, CONT!(cont));
                    cont = Continuation::EvaluateExpression(cdr.clone(), env.clone(), *level, CONT!(cont));
                    continue 'outer;
                }
                let mut list = List::from(car);
                list.extend(iter::once(ListItem::Item(expr.clone())));
                cont = Continuation::QuasiquoteList(list.into(), cdr.borrow().cdr()?.clone(), env.clone(), *level, CONT!(cont));
                cont = Continuation::EvaluateExpression(cdr.borrow().car()?.clone(), env.clone(), *level, CONT!(cont));                
            },
            Continuation::QuasiquoteListSplicing(ref car, ref cdr, ref env, ref level, ref cont_) => {
                cont = *cont_.clone();
                if cdr.borrow().is_nil() {
                    let mut list = List::from(car);
                    list.extend(List::from(expr.clone()));
                    expr = list.into();
                } else {
                    let mut list = List::from(car);
                    list.extend(List::from(expr.clone()));
                    cont = Continuation::QuasiquoteList(list.into(), cdr.borrow().cdr()?.clone(), env.clone(), *level, CONT!(cont));
                    cont = Continuation::EvaluateExpression(cdr.borrow().car()?.clone(), env.clone(), *level, CONT!(cont));
                }
            },
            _ => {
                error!("Unknown continuation passed : {:?}", cont)?
            }
        }

        
    }
}

pub fn eval_pattern_match(pattern: Value, params: Value, env: Env) -> Result<(), RuntimeError> {
    let mut stack = vec![(pattern, params)];
    while let Some((pattern, params)) = stack.pop() {
        match *pattern.borrow() {
            Datum::Symbol(ref id) => {
                if DEBUG!() { println!("Pattern matching: Binding {:} to {:?}", id, params.borrow()); }
                env.borrow_mut().put(id.clone(), params.clone());
                // println!("Env is {:?}", env);
            },
            Datum::Pair(ref a, ref d) => {
                stack.push((a.clone(), params.borrow().car()?.clone()));
                stack.push((d.clone(), params.borrow().cdr()?.clone()));
            },
            Datum::Nil => {
            }
            _ => {
                error!("Unexpected pattern matching : binding {:?} to {:?}", pattern, params)?
            }
        }
    }
    Ok(())
}

pub fn check_syntax_rule(pattern: Value, template: Value, literals: Value) -> Result<bool, RuntimeError> {
    let mut stack = vec![(pattern, template)];
    while let Some((pattern, template)) = stack.pop() {
        match *pattern.borrow() {
            Datum::Symbol(ref id) => {
                // find if literals contains id, then params must be params
                for r in List::from(&literals) {
                    if let ListItem::Item(val) = r {
                        if let Datum::Symbol(ref r) = *val.borrow() {
                            if r == id {
                                if let Datum::Symbol(ref rr) = *template.borrow() {
                                   if id != rr { return Ok(false) }
                                } else {
                                    return Ok(false);
                                }
                            }
                        }
                    }
                }
            },
            Datum::Pair(ref a, ref d) => {
                stack.push((a.clone(), template.borrow().car()?.clone()));
                stack.push((d.clone(), template.borrow().cdr()?.clone()));
            },
            Datum::Nil => {
                if !template.borrow().is_nil() { return Ok(false) }
            }
            _ => {
                return Ok(false);
            }
        }
    }
    Ok(true)
}


pub fn eval_template(expr: Value, env:Env) -> Result<Value, RuntimeError> {
    match *expr.borrow() {
        Datum::Symbol(ref id) => {
            if DEBUG!() { println!("Template matching: Binding {:} to {:?}", id, expr.borrow()); }
            Ok(env.borrow().find(id).unwrap_or(expr.clone()))
        },
        Datum::Pair(ref a, ref d) => {
            Ok(Datum::Pair(eval_template(a.clone(), env.clone())?, eval_template(d.clone(), env.clone())?).wrap())
        },
        _ => {
            Ok(expr.clone())
        }
    }
}
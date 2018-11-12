use super::beam::*;
use super::env::*;
use super::symbol::*;
use std::iter;
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
    #[derive(Debug)]
    enum Action {
        Shift, Reduce
    }
    let mut action = Action::Shift;
    let mut cont = Continuation::Return;
    let mut level: usize = 0;
    let mut expr = src;
    let mut env = env;
    'outer: loop {
        if VERBOSE!() { println!("Step: {:?}  {} \t{:?} {:?} \n\t Cont {:?}", action, level, expr.borrow(), env.borrow(), cont); }
        match action {
            Action::Shift if level == 0 => {
                match *expr.clone().borrow() {
                    Datum::Symbol(ref id) => {
                        action = Action::Reduce;
                        expr = env.borrow().find_syntax(id).or(env.borrow().find(id))?;
                    },
                    Datum::Pair(ref a, ref d) => {
                        cont = Continuation::EvaluateList(expr.clone(), env.clone(), level, CONT!(cont));
                        expr = a.clone();
                    },
                    _ => {
                        action = Action::Reduce;
                    }
                }
            },
            Action::Shift => {
                match *expr.clone().borrow() {
                    Datum::Pair(ref a, ref d) => {
                        cont = Continuation::ConstructList(List::new().into(), expr.borrow().cdr()?, env.clone(), level, CONT!(cont));
                        expr = a.clone();
                    },
                    _ => {
                        action = Action::Reduce;
                    }
                }
            },
            Action::Reduce => {
                match cont.clone() {
                    Continuation::Return => {
                        return Ok(expr);
                    },
                    Continuation::EvaluateCallCC(ref cont_) => {
                        cont = Continuation::EvaluateApply(expr.clone(), level.clone(), cont_.clone());
                        expr = List::one(Datum::Continuation(*cont_.clone()).wrap()).into();
                    },
                    Continuation::EvaluateList(ref expr_, ref env_, ref level_, ref cont_) => {
                        env = env_.clone();
                        level = level_.clone();
                        cont = *cont_.clone();
                        action = Action::Shift;
                        match *expr.clone().borrow() {
                            Datum::Builtin(_) | Datum::Lambda(_) => {
                                cont = Continuation::EvaluateApply(expr.clone(), level.clone(), CONT!(cont));
                                cont = Continuation::EvaluateProcedure(List::new().into(), expr_.borrow().cdr()?.clone(), env.clone(), level.clone(), CONT!(cont));
                            },
                            Datum::Continuation(ref cont_) => {
                                expr = expr_.borrow().cadr()?;
                                cont = cont_.clone();
                            },
                            Datum::SpecialForm(SpecialForm::Begin) => {
                                if let Continuation::EvaluateBegin(ref e, _, _, ref c) = cont {
                                    if e.borrow().is_nil() {
                                        cont = Continuation::EvaluateBegin(expr_.borrow().cdr()?.clone(), env.clone(), level.clone(), c.clone());
                                    } else {
                                        cont = Continuation::EvaluateBegin(expr_.borrow().cdr()?.clone(), env.clone(), level.clone(), CONT!(cont));
                                    }
                                } else {
                                    cont = Continuation::EvaluateBegin(expr_.borrow().cdr()?.clone(), env.clone(), level.clone(), CONT!(cont));
                                }
                            },
                            Datum::SpecialForm(SpecialForm::Define) => {
                                expr = expr_.borrow().cdr()?.borrow().cadr()?;
                                cont = Continuation::EvaluateDefine(expr_.borrow().cadr()?.clone(), env.clone(), level.clone(), CONT!(cont));
                            },
                            Datum::SpecialForm(SpecialForm::DefineSyntax) => {
                                expr = expr_.borrow().cdr()?.borrow().cadr()?;
                                cont = Continuation::EvaluateDefineSyntax(expr_.borrow().cadr()?.clone(), env.clone(), level.clone(), CONT!(cont));
                            },
                            Datum::SpecialForm(SpecialForm::If) => {
                                expr = expr_.borrow().cadr()?;
                                cont = Continuation::EvaluateIf(expr_.borrow().cdr()?.clone(), env.clone(), level.clone(), CONT!(cont));
                            },
                            Datum::SpecialForm(SpecialForm::Set) => {
                                expr = expr_.borrow().cdr()?.borrow().cadr()?;
                                cont = Continuation::EvaluateSet(expr_.borrow().cadr()?.clone(), env.clone(), level.clone(), CONT!(cont));
                            },
                            Datum::SpecialForm(SpecialForm::SyntaxRules) => {
                                expr = Datum::Syntax(SyntaxRules {
                                    literals: expr_.borrow().cadr()?,
                                    rules: expr_.borrow().cdr()?.borrow().cdr()?,
                                    env: env.clone(),
                                }).wrap();
                                action = Action::Reduce;
                            },
                            Datum::SpecialForm(SpecialForm::Lambda) => {
                                expr = Datum::Lambda(LambdaExpression {
                                    formals: expr_.borrow().cadr()?,
                                    expr: expr_.borrow().cdr()?.borrow().cadr()?,
                                    env: env.clone()
                                }).wrap();
                                action = Action::Reduce;
                            },
                            Datum::SpecialForm(SpecialForm::CallCC) => {
                                expr = expr_.borrow().cadr()?;
                                cont = Continuation::EvaluateCallCC(CONT!(cont));
                            },
                            Datum::SpecialForm(SpecialForm::Quote) => {
                                expr = expr_.borrow().cadr()?;
                                action = Action::Reduce;
                            },
                            Datum::SpecialForm(SpecialForm::Quasiquote) => {
                                expr = expr_.borrow().cadr()?;
                                level += 1;
                            },
                            Datum::SpecialForm(SpecialForm::Unquote) => {
                                error!("Unexpected unquote")?
                            },
                            Datum::SpecialForm(SpecialForm::UnquoteSplicing) => {
                                error!("Unexpected unquote-splicing")?
                            },
                            Datum::Syntax(SyntaxRules { ref literals, ref rules, ref env }) => {
                                let rules = List::from(rules.clone());
                                let mut flag_ok = false;
                                for li in rules {
                                    if let ListItem::Item(rule) = li {
                                        if let Ok(true) = check_syntax_rule(rule.borrow().car()?.borrow().cdr()?, expr_.borrow().cdr()?, literals.clone()) {
                                            // println!("Syntax checked : {:?} {:?} {:?}", rule.borrow(), rule.borrow().car()?.borrow().cdr()?, expr_.borrow().cdr()?);
                                            flag_ok = true;
                                            let null_env = Environment::null();
                                            eval_pattern_match(rule.borrow().car()?.borrow().cdr()?, expr_.borrow().cdr()?, null_env.clone())?;
                                            expr = eval_template(rule.borrow().cadr()?, null_env.clone())?;
                                            action = Action::Shift;
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
                    Continuation::EvaluateApply(ref expr_, ref level_, ref cont_) => {
                        level = level_.clone();
                        cont = *cont_.clone();
                        match *expr_.clone().borrow() {
                            Datum::Builtin(ref func) => {
                                expr = func(expr.clone())?;
                                action = Action::Reduce;
                            },
                            Datum::Lambda(LambdaExpression { ref formals, expr: ref expr_, env: ref env_ }) => {
                                env = Environment::forward(env_.clone());
                                eval_pattern_match(formals.clone(), expr.clone(), env.clone())?;
                                action = Action::Shift;
                                expr = expr_.clone();
                            },
                            _ => {
                                error!("unapplicable passed to apply : {:?}", expr.borrow())?
                            }
                        }
                    },
                    Continuation::EvaluateProcedure(ref car, ref cdr, ref env_, ref level_, ref cont_) => {
                        env = env_.clone();
                        level = level_.clone();
                        cont = *cont_.clone();                        
                        if cdr.borrow().is_nil() {
                            action = Action::Reduce;
                            let mut list = List::from(car);
                            list.extend(iter::once(ListItem::Item(expr.clone())));
                            list.next();
                            expr = list.into();
                        } else {
                            action = Action::Shift;
                            let mut list = List::from(car);
                            list.extend(iter::once(ListItem::Item(expr.clone())));
                            if cdr.borrow().is_pair() {
                                expr = cdr.borrow().car()?.clone();
                                cont = Continuation::EvaluateProcedure(list.into(), cdr.borrow().cdr()?.clone(), env.clone(), level.clone(), CONT!(cont));
                            } else {
                                expr = cdr.clone();
                                cont = Continuation::EvaluateProcedureSplicing(list.into(), SymbolTable::nil(), env.clone(), level.clone(), CONT!(cont));
                            }
                        }
                    },
                    Continuation::EvaluateProcedureSplicing(ref car, ref cdr, ref env_, ref level_, ref cont_) => {
                        env = env_.clone();
                        level = level_.clone();
                        cont = *cont_.clone();                        
                        action = Action::Reduce;
                        let mut list = List::from(car);
                        list.extend(iter::once(ListItem::Ellipsis(expr.clone())));
                        list.next();
                        expr = list.into();
                    },
                    Continuation::EvaluateBegin(ref expr_, ref env_, ref level_, ref cont_) => {
                        env = env_.clone();
                        level = level_.clone();
                        cont = *cont_.clone();                        
                        if expr_.borrow().is_nil() {
                            action = Action::Reduce;
                        } else {
                            action = Action::Shift;
                            expr = expr_.borrow().car()?.clone();
                            cont = Continuation::EvaluateBegin(expr_.borrow().cdr()?.clone(), env.clone(), level.clone(), CONT!(cont));                            
                        }
                    },
                    Continuation::EvaluateSet(ref expr_, ref env_, ref level_, ref cont_) => {
                        env = env_.clone();
                        level = level_.clone();
                        cont = *cont_.clone();
                        action = Action::Reduce;
                        // Check for non-syntax
                        if let Datum::Symbol(ref id) = *expr_.borrow() {
                            env.borrow_mut().set(id, expr.clone())?;
                        } else {
                            error!("expected symbol in set : {:?}", expr_.borrow())?
                        }
                        expr = expr_.clone();
                    },
                    Continuation::EvaluateDefine(ref expr_, ref env_, ref level_, ref cont_) => {
                        env = env_.clone();
                        level = level_.clone();
                        cont = *cont_.clone();
                        action = Action::Reduce;
                        // Check for non-syntax
                        if let Datum::Symbol(ref id) = *expr_.borrow() {
                            env.borrow_mut().put(id.clone(), expr.clone());
                        } else {
                            error!("expected symbol in define : {:?}", expr_.borrow())?
                        }
                        expr = expr_.clone();
                    },
                    Continuation::EvaluateDefineSyntax(ref expr_, ref env_, ref level_, ref cont_) => {
                        env = env_.clone();
                        level = level_.clone();
                        cont = *cont_.clone();
                        action = Action::Reduce;
                        // Check for syntax
                        if let Datum::Symbol(ref id) = *expr_.borrow() {
                            env.borrow_mut().put_syntax(id.clone(), expr.clone());
                        } else {
                            error!("expected symbol in define : {:?}", expr_.borrow())?
                        }
                        expr = expr_.clone();
                    },
                    Continuation::EvaluateIf(ref expr_, ref env_, ref level_, ref cont_) => {
                        env = env_.clone();
                        level = level_.clone();
                        cont = *cont_.clone();
                        action = Action::Shift;
                        if expr.clone().borrow().is_true() {
                            expr = expr_.borrow().cadr()?;
                        } else {
                            if let Ok(e) = expr_.borrow().cdr()?.borrow().cadr() {
                                expr = e;
                            } else {
                                expr = SymbolTable::unspecified();
                            }
                        }
                    },
                    Continuation::EvaluateSyntax(ref expr_, ref env_, ref level_, ref cont_) => {
                        env = env_.clone();
                        level = level_.clone();
                        cont = *cont_.clone();
                        if expr.clone().borrow().is_true() {
                            action = Action::Shift;
                            expr = expr_.borrow().cadr()?;
                        } else {
                            if let Ok(e) = expr_.borrow().cdr()?.borrow().cadr() {
                                expr = e;
                            } else {
                                expr = SymbolTable::unspecified();
                            }
                        }
                    },
                    Continuation::ConstructList(ref car, ref cdr, ref env_, ref level_, ref cont_) => {
                        env = env_.clone();
                        level = level_.clone();
                        cont = *cont_.clone();
                        if cdr.borrow().is_nil() {
                            action = Action::Reduce;
                            let mut list = List::from(car);
                            list.extend(iter::once(ListItem::Item(expr.clone())));
                            expr = list.into();
                            continue 'outer;
                        } else if car.borrow().len() == 0 && cdr.borrow().len() == 1 {
                            if let Datum::Symbol(ref id) = *expr.clone().borrow() {
                                if let Ok(val) = env.borrow().find_syntax(id) {
                                    match *val.borrow() {
                                        Datum::SpecialForm(SpecialForm::Quasiquote) => {
                                            action = Action::Shift;
                                            let mut list = List::from(car);
                                            list.extend(iter::once(ListItem::Item(expr.clone())));
                                            expr = cdr.borrow().car()?.clone();
                                            cont = Continuation::ConstructList(list.into(), cdr.borrow().cdr()?.clone(), env.clone(), level.clone(), CONT!(cont));
                                            level = level + 1;
                                            continue 'outer;
                                        },
                                        Datum::SpecialForm(SpecialForm::Unquote) => {
                                            action = Action::Shift;
                                            if level == 1 {
                                                // cont = Continuation::ConstructList(car.clone(), cdr.borrow().cdr()?.clone(), env.clone(), level.clone(), CONT!(cont));
                                                // using outer continuation
                                            } else {
                                                let mut list = List::from(car);
                                                list.extend(iter::once(ListItem::Item(expr.clone())));
                                                cont = Continuation::ConstructList(list.into(), cdr.borrow().cdr()?.clone(), env.clone(), level.clone(), CONT!(cont));                                    
                                            }
                                            expr = cdr.borrow().car()?.clone();
                                            level = level - 1;
                                            continue 'outer;
                                        },
                                        Datum::SpecialForm(SpecialForm::UnquoteSplicing) => {
                                            action = Action::Shift;
                                            if level == 1 {
                                                // transform outer continuation
                                                cont = cont.splicing()?;
                                            } else {
                                                let mut list = List::from(car);
                                                list.extend(iter::once(ListItem::Item(expr.clone())));
                                                cont = Continuation::ConstructList(list.into(), cdr.borrow().cdr()?.clone(), env.clone(), level.clone(), CONT!(cont));                                    
                                            }
                                            expr = cdr.borrow().car()?.clone();
                                            level = level - 1;
                                            continue 'outer;
                                        },
                                        _ => {}
                                    }
                                }
                            }
                        }
                        action = Action::Shift;
                        let mut list = List::from(car);
                        list.extend(iter::once(ListItem::Item(expr.clone())));
                        expr = cdr.borrow().car()?.clone();
                        cont = Continuation::ConstructList(list.into(), cdr.borrow().cdr()?.clone(), env.clone(), level.clone(), CONT!(cont));
                    },
                    Continuation::ConstructListSplicing(ref car, ref cdr, ref env_, ref level_, ref cont_) => {
                        env = env_.clone();
                        level = level_.clone();
                        cont = *cont_.clone();
                        if cdr.borrow().is_nil() {
                            action = Action::Reduce;
                            let mut list = List::from(car);
                            list.extend(List::from(expr.clone()));
                            expr = list.into();
                        } else {
                            action = Action::Shift;
                            let mut list = List::from(car);
                            list.extend(List::from(expr.clone()));
                            expr = cdr.borrow().car()?.clone();
                            cont = Continuation::ConstructList(list.into(), cdr.borrow().cdr()?.clone(), env.clone(), level.clone(), CONT!(cont));                            
                        }
                    },
                    _ => {
                        error!("Unknown continuation passed : {:?}", cont)?
                    }
                }
            }
        }
        
    }
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
            error!("Unexpected pattern matching : binding {:?} to {:?}", pattern, params)
        }
    }
}

pub fn check_syntax_rule(pattern: Value, template: Value, literals: Value) -> Result<bool, RuntimeError> {
    match *pattern.borrow() {
        Datum::Symbol(ref id) => {
            // env.borrow_mut().put(id.clone(), params.clone());
            // println!("Env is {:?}", env);
            // find if literals contains id, then params must be params
            for r in List::from(literals) {
                if let ListItem::Item(val) = r {
                    if let Datum::Symbol(ref r) = *val.borrow() {
                        if r == id {
                            if let Datum::Symbol(ref rr) = *template.borrow() {
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
            Ok(check_syntax_rule(a.clone(), template.borrow().car()?.clone(), literals.clone())?
            && check_syntax_rule(d.clone(), template.borrow().cdr()?.clone(), literals.clone())?)
        },
        Datum::Nil => {
            Ok(template.borrow().is_nil())
        }
        _ => {
            Ok(false)
            // Err(RuntimeError::new("Unexpected pattern matching"))
        }
    }
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
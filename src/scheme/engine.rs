use super::beam::*;
use super::env::*;
use super::symbol::*;
use std::iter;
use crate::utils::*;

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
    loop {
        println!("Step: {:?} \t{:?} \n\t{:?}", action, expr.borrow(), cont);
        match action {
            Action::Shift if level == 0 => {
                match *expr.clone().borrow() {
                    Datum::Symbol(ref id) => {
                        action = Action::Reduce;
                        expr = env.borrow().find(id)?;
                    },
                    Datum::Pair(ref a, ref d) => {
                        cont = Continuation::EvaluateList(expr.clone(), env.clone(), level, CONT!(cont));
                        expr = a.clone();
                    },
                    Datum::Abbreviation(AbbrevPrefix::Quote, ref val) => {
                        expr = val.clone();
                        action = Action::Reduce;
                    },
                    Datum::Abbreviation(AbbrevPrefix::Quasiquote, ref val) => {
                        expr = val.clone();
                        level += 1;
                    },
                    Datum::Abbreviation(AbbrevPrefix::Unquote, ref val) => {
                        expr = val.clone();
                        if level == 0 { Err(RuntimeError::new("Unexpected unquote"))? }
                        level -= 1;
                    },
                    _ => {
                        action = Action::Reduce;
                    }
                }
            },
            Action::Shift => {
                match *expr.clone().borrow() {
                    Datum::Pair(ref a, ref d) => {
                        cont = Continuation::EvaluateList(expr.clone(), env.clone(), level, CONT!(cont));
                        expr = a.clone();
                    },
                    Datum::Abbreviation(AbbrevPrefix::Quote, ref val) => {
                        expr = val.clone();
                        action = Action::Reduce;
                    },
                    Datum::Abbreviation(AbbrevPrefix::Quasiquote, ref val) => {
                        expr = val.clone();
                        level += 1;
                    },
                    Datum::Abbreviation(AbbrevPrefix::Unquote, ref val) => {
                        expr = val.clone();
                        if level == 0 { Err(RuntimeError::new("Unexpected unquote"))? }
                        level -= 1;
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
                        cont = Continuation::EvaluateApply(expr.clone(), env.clone(), level.clone(), cont_.clone());
                        expr = Datum::Pair(Datum::Continuation(*cont_.clone()).wrap(), SymbolTable::nil()).wrap();
                    },
                    Continuation::EvaluateList(ref expr_, ref env_, ref level_, ref cont_) => {
                        env = env_.clone();
                        level = level_.clone();
                        cont = *cont_.clone();
                        action = Action::Shift;                
                        match *expr.clone().borrow() {
                            Datum::Builtin(_) | Datum::Lambda(_) => {
                                cont = Continuation::EvaluateApply(expr.clone(), env.clone(), level.clone(), CONT!(cont));
                                cont = Continuation::EvaluateProcedure(List::new().into(), expr_.borrow().cdr()?.clone(), env.clone(), level.clone(), CONT!(cont));
                            },
                            Datum::Continuation(ref cont_) => {
                                expr = expr_.borrow().cadr()?;
                                cont = cont_.clone();
                            },
                            Datum::SpecialForm(SpecialForm::Begin) => {
                                cont = Continuation::EvaluateBegin(expr_.borrow().cdr()?.clone(), env.clone(), level.clone(), CONT!(cont));
                            },
                            Datum::SpecialForm(SpecialForm::Define) => {
                                expr = expr_.borrow().cdr()?.borrow().cadr()?;
                                cont = Continuation::EvaluateDefine(expr_.borrow().cadr()?.clone(), env.clone(), level.clone(), CONT!(cont));
                            },
                            Datum::SpecialForm(SpecialForm::If) => {
                                expr = expr_.borrow().cadr()?;
                                cont = Continuation::EvaluateIf(expr_.borrow().cdr()?.clone(), env.clone(), level.clone(), CONT!(cont));
                            },
                            Datum::SpecialForm(SpecialForm::SyntaxRules) => {
                                expr = Datum::Syntax(SyntaxRules {
                                    literals: expr_.borrow().cadr()?,
                                    rules: expr_.borrow().cdr()?.borrow().cdr()?
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
                                expr = expr_.borrow().cadr()?;
                                if level == 0 { Err(RuntimeError::new("Unexpected unquote"))? }
                                level -= 1;
                            },
                            Datum::SpecialForm(SpecialForm::UnquoteSplicing) => {
                                expr = expr_.borrow().cadr()?;
                                if level == 0 { Err(RuntimeError::new("Unexpected unquote"))? }
                                level -= 1;
                            },
                            Datum::Syntax(SyntaxRules { ref literals, ref rules }) => {
                                let rules = List::from(rules.clone());
                                let mut flag_ok = false;
                                for li in rules {
                                    if let ListItem::Item(rule) = li {
                                        if let Ok(true) = check_syntax_rule(rule.borrow().car()?.borrow().cdr()?, expr_.borrow().cdr()?, literals.clone()) {
                                            flag_ok = true;
                                            let null_env = Environment::null();
                                            eval_pattern_match(rule.borrow().car()?.borrow().cdr()?, expr_.borrow().cdr()?, null_env.clone())?;
                                            expr = eval_template(rule.borrow().cadr()?, null_env.clone())?;
                                            action = Action::Shift;
                                            break;
                                        }
                                    }
                                }
                                if !flag_ok { Err(RuntimeError::new("syntex not matched"))? }
                            },
                            _ => {
                                Err(RuntimeError::new(format!("{:?} is not applicable", expr_)))?
                            }
                        }
                    },
                    Continuation::EvaluateApply(ref expr_, ref env_, ref level_, ref cont_) => {
                        env = Environment::forward(env_.clone());
                        level = level_.clone();
                        cont = *cont_.clone();
                        match *expr_.clone().borrow() {
                            Datum::Builtin(ref func) => {
                                expr = func(expr.clone())?;
                                action = Action::Reduce;
                            },
                            Datum::Lambda(LambdaExpression { ref formals, expr: ref expr_, ref env }) => {
                                eval_pattern_match(formals.clone(), expr.clone(), env.clone());
                                action = Action::Shift;
                                expr = expr_.clone();
                            },
                            _ => {
                                Err(RuntimeError::new(format!("unapplicable passed to apply : {:?}", expr.borrow())))?
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
                            list = list.chain(iter::once(ListItem::Item(expr.clone()))).collect();
                            list.next();
                            expr = list.into();
                        } else {
                            action = Action::Shift;
                            let mut list = List::from(car);
                            list = list.chain(iter::once(ListItem::Item(expr.clone()))).collect();
                            expr = cdr.borrow().car()?.clone();
                            cont = Continuation::EvaluateProcedure(list.into(), cdr.borrow().cdr()?.clone(), env.clone(), level.clone(), CONT!(cont));                            
                        }
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
                    Continuation::EvaluateDefine(ref expr_, ref env_, ref level_, ref cont_) => {
                        env = env_.clone();
                        level = level_.clone();
                        cont = *cont_.clone();
                        action = Action::Reduce;
                        if let Datum::Symbol(ref id) = *expr_.borrow() {
                            env.borrow_mut().put(id.clone(), expr.clone());
                        } else {
                            Err(RuntimeError::new("xpected symbol in define"))?
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
                    _ => {
                        Err(RuntimeError::new(format!("Unknown continuation passed : {:?}", cont)))?
                    }
                }
            }
        }
        
    }
}

// pub fn eval_list(term: Value, env: Env) -> Result<Value, RuntimeError> {
//     let mut ret = List::new();
//     let mut list = List::from(term);
//     while let Some(next) = list.next() {
//         if let ListItem::Item(x) = next {
//             ret = ret.chain(iter::once(ListItem::Item(eval(x, env.clone())?))).collect();
//         } else if let ListItem::Ellipsis(x) = next {
//             ret = ret.chain(iter::once(ListItem::Ellipsis(eval(x, env.clone())?))).collect();
//         }
//     }
//     Ok(ret.into())
// }

// pub fn eval_begin(term: Value, env: Env) -> Result<Value, RuntimeError> {
//     let mut ret = SymbolTable::unspecified();
//     let mut list = List::from(term);
//     while let Some(next) = list.next() {
//         if let ListItem::Item(x) = next {
//             ret = eval(x, env.clone())?;
//         } else {
//             Err(RuntimeError::new("Unexpected form in begin"))?
//         }
//     }
//     Ok(ret)
// }

// pub fn eval_and(term: Value, env: Env) -> Result<Value, RuntimeError> {
//     let mut ret = SymbolTable::bool(true);
//     let mut list = List::from(term);
//     while let Some(next) = list.next() {
//         if let ListItem::Item(x) = next {
//             ret = eval(x, env.clone())?;            
//             if ret.borrow().is_false() {
//                 break;
//             }
//         } else {
//             Err(RuntimeError::new("Unexpected form in and"))?
//         }
//     }
//     Ok(ret)
// }

// pub fn eval_or(term: Value, env: Env) -> Result<Value, RuntimeError> {
//     let mut ret = SymbolTable::bool(false);
//     let mut list = List::from(term);
//     while let Some(next) = list.next() {
//         if let ListItem::Item(x) = next {
//             ret = eval(x.clone(), env.clone())?;
//             if ret.borrow().is_true() {
//                 break;
//             }
//         } else {
//             Err(RuntimeError::new("Unexpected form in or"))?
//         }
//     }
//     Ok(ret)
// }

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

// pub fn eval_quasiquote(term: Value, env: Env) -> Result<Value, RuntimeError> {
//     let mut ret = List::new();
//     let mut list = List::from(term);
//     while let Some(next) = list.next() {
//         if let ListItem::Item(x) = next {
//             match *x.borrow() {
//                 Datum::Abbreviation(AbbrevPrefix::Unquote, ref val) => {
//                     ret = ret.chain(iter::once(ListItem::Item(
//                         eval(val.clone(), env.clone())?
//                     ))).collect();
//                 },
//                 Datum::Abbreviation(AbbrevPrefix::UnquoteSplicing, ref val) => {
//                     ret = ret.chain(List::from(
//                         eval(val.clone(), env.clone())?
//                     )).collect();
//                 },
//                 _ => {
//                     // if let Datum::Pair(ref a, ref d) = *x.borrow() {
//                     //     if let Datum::Symbol(ref id) = *a.borrow() {
//                     //         if id == "unquote" {
//                     //             ret = ret.chain(iter::once(ListItem::Item(
//                     //                 eval(d.clone(), env.clone())?
//                     //             ))).collect();
//                     //             continue;
//                     //         } else if id == "unquote-splicing" {
//                     //             ret = ret.chain(List::from(
//                     //                 eval(d.clone(), env.clone())?
//                     //             )).collect();
//                     //             continue;
//                     //         }
//                     //     }
//                     // }
//                     ret = ret.chain(iter::once(ListItem::Item(
//                         x.clone()
//                     ))).collect();
//                 }
//             }
//         } else if let ListItem::Ellipsis(x) = next {
//             match *x.borrow() {
//                 Datum::Abbreviation(AbbrevPrefix::Unquote, ref val) => {
//                     ret = ret.chain(iter::once(ListItem::Ellipsis(
//                         eval(val.clone(), env.clone())?
//                     ))).collect();
//                 },
//                 Datum::Abbreviation(AbbrevPrefix::UnquoteSplicing, ref val) => {
//                     Err(RuntimeError::new(",@ in unexpected context"))?
//                 },
//                 _ => {
//                     ret = ret.chain(iter::once(ListItem::Ellipsis(
//                         x.clone()
//                     ))).collect();
//                 }
//             }
//         }
//     }
//     Ok(ret.into())
// }

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
        Datum::Nil => {
            Ok(expr.clone())
        }
        _ => {
            Err(RuntimeError::new("Unexpected pattern matching"))
        }
    }
}
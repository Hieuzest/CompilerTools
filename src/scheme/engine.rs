use super::beam::*;
use super::env::*;
use std::iter;


pub fn eval(term: &Datum, env: &mut Enviroment) -> Result<Datum, RuntimeError> {
    let ret = eval_inner(term, env)?;
    println!("Evaluating {:?} => {:?}", term, ret);
    Ok(ret)
}
pub fn eval_inner(term: &Datum, env: &mut Enviroment) -> Result<Datum, RuntimeError> {
    match term {
        Datum::Symbol(s) => {
            env.find(&s)
        },
        Datum::SpecialForm(sf) => Ok(Datum::SpecialForm(*sf)),
        Datum::Abbreviation(AbbrevPrefix::Quote, d) => Ok(*d.clone()),
        Datum::Abbreviation(AbbrevPrefix::Template, d) => {
            if let Datum::List(l) = d.as_ref() {
                Ok(Datum::List(l.iter().flat_map(|x| {match x {
                    Datum::Abbreviation(AbbrevPrefix::Comma, dd) => {
                        vec![eval(dd, env).expect(&format!("Error occured when eval {:?}", dd))]
                    },
                    Datum::Abbreviation(AbbrevPrefix::CommaSplicing, dd) => {
                        if let Datum::List(l) = eval(dd, env).expect(&format!("Error occured when eval {:?}", dd)) {
                            l.iter().map(|xx| eval(xx, env).expect(&format!("Error occured when eval {:?}", xx))).collect::<Vec<Datum>>()
                        } else {
                            panic!("Comma splicing on non-list")
                        }
                    },
                    Datum::Abbreviation(AbbrevPrefix::Template, _) => vec![eval(&x, env).expect(&format!("Error occured when eval {:?}", x))],
                    _ => vec![x.clone()]
                }}).collect()))
            } else {
                Ok(*d.clone())
            }
        },
        Datum::Abbreviation(AbbrevPrefix::Comma, _) | Datum::Abbreviation(AbbrevPrefix::CommaSplicing, _) => Err(RuntimeError::new(format!("Unquote : {:?}", term))) ,
        Datum::List(items) => {
            let operator = eval(items.get(0).expect("Null list"), env)?;
            match operator {
                // Special Forms
                // define
                // lambda
                // cond
                Datum::SpecialForm(SpecialForm::Apply) => {
                    let mut l = if let Datum::List(l) = eval(&items[2], env)? { l } else { panic!("The second argument of apply is not a list") };
                    l.insert(0, items[1].clone());
                    eval(&Datum::List(l), env)
                },
                Datum::SpecialForm(SpecialForm::Eval) => {
                    let ret = eval(&items[1], env)?;
                    eval(&ret, env)
                },
                Datum::SpecialForm(SpecialForm::Begin) => {
                    let mut operands: Vec<Datum> = items.iter().skip(1).map(|x| eval(x, env).expect(&format!("Error occured when eval {:?}", x))).collect();
                    Ok(operands.pop().expect("Null begin"))
                },
                Datum::SpecialForm(SpecialForm::Define) => {
                    if let Datum::Symbol(id) = &items[1] {
                        let value = eval(&items[2], env)?;
                        env.put(id.clone(), value);
                    } else if let Datum::List(l) = &items[1] {
                        let id = if let Datum::Symbol(id) = &l[0] { id.clone() } else { panic!("Define lambda error") };
                        let formals: Vec<String> = l.clone().into_iter().skip(1).map(|x| if let Datum::Symbol(id) = x { id } else { panic!("Not formal") }).collect();
                        let lambda = Datum::Lambda(LambdaExpression {
                            formals: formals,
                            expr: Box::new(Datum::List(
                                vec![Datum::SpecialForm(SpecialForm::Begin)]
                                    .into_iter()
                                    .chain(items.iter().skip(2).map(|x| x.clone()))
                                    .collect()
                                ))
                        });
                        env.put(id, lambda);
                    }
                    Ok(Datum::new())
                },
                Datum::SpecialForm(SpecialForm::DefineSyntax) => {
                    let id = if let Datum::Symbol(id) = &items[1] { id.clone() } else { panic!("Define syntax error") };
                    let formals: Vec<String> = if let Datum::List(l) = &items[2] {
                        l.clone().into_iter().map(|x| if let Datum::Symbol(id) = x { id } else { panic!("Not formal") }).collect()
                    } else { panic!("Not formal") };
                    let syntax = Datum::Syntax(SyntaxRule {
                        formals: formals,
                        expr: Box::new(Datum::List(
                            vec![Datum::SpecialForm(SpecialForm::Begin)]
                                .into_iter()
                                .chain(items.iter().skip(3).map(|x| x.clone()))
                                .collect()
                            ))
                    });
                    env.put(id, syntax);
                    Ok(Datum::new())
                },
                Datum::SpecialForm(SpecialForm::Set) => {
                    if let Datum::Symbol(ref id) = &items[1] {
                        let value = eval(&items[2], env)?;
                        env.set(&id, value)?;
                    }
                    Ok(Datum::new())
                },
                Datum::SpecialForm(SpecialForm::Lambda) => {
                    let formals: Vec<String> = 
                        if let Datum::List(l) = items[1].clone() {
                            l.into_iter().map(|x| if let Datum::Symbol(id) = x { id } else { panic!("Not formal") }).collect()
                        } else { panic!("Not formals") };
                    Ok(Datum::Lambda(LambdaExpression {
                        formals: formals,
                        expr: Box::new(Datum::List(
                            vec![Datum::SpecialForm(SpecialForm::Begin)]
                                .into_iter()
                                .chain(items.iter().skip(2).map(|x| x.clone()))
                                .collect()
                            ))
                    }))
                },
                Datum::SpecialForm(SpecialForm::And) => {
                    let ret = items.iter().skip(1).all(|x| if let Datum::Boolean(true) = eval(x, env).expect(&format!("Error occured when eval {:?}", x)) { true } else { false });
                    Ok(Datum::Boolean(ret))
                },
                Datum::SpecialForm(SpecialForm::Or) => {
                    let ret = items.iter().skip(1).any(|x| if let Datum::Boolean(true) = eval(x, env).expect(&format!("Error occured when eval {:?}", x)) { true } else { false });
                    Ok(Datum::Boolean(ret))
                },
                Datum::SpecialForm(SpecialForm::Cond) => {
                    let ret = items.iter().skip(1).find_map(|x| if let Datum::List(l) = x {
                        if let Datum::Boolean(true) = eval(&l[0], env).expect(&format!("Error occured when eval {:?}", x)) {
                            Some(eval(&l[1], env).expect(&format!("Error occured when eval {:?}", x)))
                        } else { None }
                    } else { None });
                    Ok(ret.expect(&format!("Unexhausted cond: {:?}", term)))
                },
                Datum::SpecialForm(SpecialForm::If) => {
                    if let Datum::Boolean(true) = eval(&items[1], env)? {
                        eval(&items[2], env)
                    } else if items.len() == 4 {
                        eval(&items[3], env)
                    } else {
                        Ok(Datum::new())
                    }
                },
                Datum::SpecialForm(SpecialForm::Quote) => {
                    Ok(items[1].clone())
                },
                Datum::SpecialForm(SpecialForm::Letstar) | Datum::SpecialForm(SpecialForm::Letrec) => {
                    Ok(enter_env!(env, {
                        if let Datum::List(l) = &items[1] {
                            l.iter().for_each(|x| if let Datum::List(l) = x {
                                let ret = eval(&l[1], env).expect(&format!("Error occured when eval {:?}", l[1]));
                                env.put(if let Datum::Symbol(id) = &l[0] { id.clone() } else { panic!("Not formal in let") }, ret);
                            } else { panic!("Not formal in let") } );
                        } else { panic!("Not formal in let"); }
                        let mut operands: Vec<Datum> = items.iter().skip(2).map(|x| eval(x, env).expect(&format!("Error occured when eval {:?}", x))).collect();
                        operands.pop().expect("Null let")
                    }))
                },
                Datum::SpecialForm(SpecialForm::Let) => {
                    Ok(enter_env!(env, {
                        if let Datum::List(l) = &items[1] {
                            l.iter()
                            .map(|x| if let Datum::List(l) = x {
                                let id = if let Datum::Symbol(id) = &l[0] { id.clone() } else { panic!("Not formal in let") };
                                let val = eval(&l[1], env).expect(&format!("Error occured when eval {:?}", l[1]));
                                (id, val)
                            } else { panic!("Not formal in let") })
                            .collect::<Vec<(String, Datum)>>()
                            .into_iter()
                            .for_each(|(id, val)| env.put(id, val));
                        } else { panic!("Not formal in let"); }
                        let mut operands: Vec<Datum> = items.iter().skip(2).map(|x| eval(x, env).expect(&format!("Error occured when eval {:?}", x))).collect();
                        operands.pop().expect("Null let")
                    }))
                },
                // Precudure Call
                Datum::Lambda(LambdaExpression { ref formals, ref expr }) => {
                    let operands: Vec<Datum> = items.iter().skip(1).map(|x| eval(x, env).expect(&format!("Error occured when eval {:?}", x))).collect();
                    Ok(enter_env!(env, {
                        for (f, d) in formals.iter().zip(operands.iter()) {
                            env.put(f.clone(), d.clone());
                        }
                        eval(expr, env)?
                    }))
                },
                Datum::Syntax(SyntaxRule { ref formals, ref expr }) => {
                    let operands: Vec<Datum> = items.iter().skip(1).map(|x| x.clone()).collect();
                    Ok(enter_env!(env, {
                        for (f, d) in formals.iter().zip(operands.iter()) {
                            env.put(f.clone(), d.clone());
                        }
                        let ret = eval(expr, env)?;
                        eval(&ret, env)?
                    }))
                },
                Datum::Builtin(func) => {
                    let operands: Vec<Datum> = items.iter().skip(1).map(|x| eval(x, env).expect(&format!("Error occured when eval {:?}", x))).collect();
                    func(operands)
                },
                _ => {
                    println!("Evaluating {:?} Error", operator);
                    panic!("Not s-expr")
                }
            }
        },
        _ => { Ok(term.clone()) }
    }
}
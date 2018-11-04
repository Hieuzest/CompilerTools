use super::beam::*;
use super::env::*;


pub fn eval(term: &Datum, env: &mut Enviroment) -> Result<Datum, RuntimeError> {
    let ret = eval_inner(term, env)?;
    println!("Evaluating {:?} => {:?}", term, ret);
    Ok(ret)
}
pub fn eval_inner(term: &Datum, env: &mut Enviroment) -> Result<Datum, RuntimeError> {
    match term {
        Datum::Identifier(s) => {
            env.find(&s)
        },
        Datum::List(items) => {
            let operator = eval(items.get(0).expect("Null list"), env)?;
            match operator {
                // Special Forms
                // define
                // lambda
                // cond
                Datum::SpecialForm(SpecialForm::Begin) => {
                    let mut operands: Vec<Datum> = items.iter().skip(1).map(|x| eval(x, env).expect(&format!("Error occured when eval {:?}", x))).collect();
                    Ok(operands.pop().expect("Null begin"))
                },
                Datum::SpecialForm(SpecialForm::Define) => {
                    if let Datum::Identifier(ref id) = &items[1] {
                        let value = eval(&items[2], env)?;
                        env.put(id.clone(), value);
                    }
                    Ok(Datum::new())
                },
                Datum::SpecialForm(SpecialForm::Set) => {
                    if let Datum::Identifier(ref id) = &items[1] {
                        let value = eval(&items[2], env)?;
                        env.set(&id, value)?;
                    }
                    Ok(Datum::new())
                },
                Datum::SpecialForm(SpecialForm::Lambda) => {
                    let formals: Vec<String> = 
                        if let Datum::List(l) = items[1].clone() {
                            l.into_iter().map(|x| if let Datum::Identifier(id) = x { id } else { panic!("Not formal") }).collect()
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
                Datum::Builtin(func) => {
                    let operands: Vec<Datum> = items.iter().skip(1).map(|x| eval(x, env).expect(&format!("Error occured when eval {:?}", x))).collect();
                    func(operands)
                },
                _ => {
                    println!("{:?}", operator);
                    panic!("Not s-expr")
                }
            }
        },
        Datum::SpecialForm(sf) => Ok(Datum::SpecialForm(*sf)),
        _ => { Ok(term.clone()) }
    }
}
use super::beam::*;
use super::core;
use crate::utils::*;


use std::collections::HashMap;

macro_rules! enter_env {
    ($env: expr, $b: block) => {{
        $env.forward();
        let ret_ = $b;
        $env.downward();
        ret_
    }};
}


macro_rules! builtins(
    { $($key:expr => $value:expr),+ } => {
        {
            let mut m = ::std::collections::HashMap::new();
            $(
                m.insert($key.to_string(), Datum::Builtin(Box::new($value)));
            )+
            m
        }
     };
);

#[derive(Debug, Default, Clone)]
pub struct Enviroment {
    datas: Option<HashMap<String, Datum>>,
    parent: Option<Box<Enviroment>>,
}

impl Enviroment {

    pub fn new() -> Self {
        let m = builtins![
            "=" => core::eq,
            "<" => core::lt,
            "<=" => core::le,
            "+" => core::add,
            "-" => core::sub,
            "*" => core::mul,
            "/" => core::div,
            "car" => core::car,
            "cdr" => core::cdr,
            "list" => core::list
        ];

        Enviroment {
            datas: Some(m),
            parent: Some(Box::new(Enviroment::default())),
            ..Default::default()
        }
    }

    fn datas(&self) -> &HashMap<String, Datum> {
        self.datas.as_ref().expect("ICE: Env data none")
    }

    fn datas_mut(&mut self) -> &mut HashMap<String, Datum> {
        self.datas.as_mut().expect("ICE: Env data none")
    }

    pub fn put(&mut self, name: String, data: Datum) {
        self.datas_mut().insert(name, data);
    }

    pub fn set(&mut self, name: &String, data: Datum) -> Result<Datum, RuntimeError> {
        if let None = self.datas {
            Err(RuntimeError::new(format!("No variable named {:}", name)))
        } else if let Some(d) = self.datas_mut().get_mut(name) {
            let old = d.clone();
            *d = data;
            Ok(old)
        } else if let Some(p) = &mut self.parent {
            p.set(name, data)
        } else {
            Err(RuntimeError::new(format!("No variable named {:}", name)))
        }        
    }

    pub fn find(&self, name: &String) -> Result<Datum, RuntimeError> {
        if let None = self.datas {
            Err(RuntimeError::new(format!("No variable named {:}", name)))
        } else if let Some(d) = self.datas().get(name) {
            Ok(d.clone())
        } else if let Some(p) = &self.parent {
            p.find(name)
        } else {
            Err(RuntimeError::new(format!("No variable named {:}", name)))
        }
    }

    pub fn forward(&mut self) {
        let e = Enviroment {
            datas: self.datas.take(),
            parent: self.parent.take(),
        };
        self.datas.replace(HashMap::new());
        self.parent.replace(Box::new(e));
    }

    pub fn downward(&mut self) {
        let datas = self.parent.as_mut().expect("ICE: Env data none").datas.take().expect("ICE: Env data none");
        let parent = self.parent.as_mut().expect("ICE: Env higheset").parent.take().expect("ICE: Env higheset");
        self.datas.replace(datas);
        self.parent.replace(parent);
    }

}   
use super::beam::*;
use super::env::*;
use std::sync::{Once, ONCE_INIT};
use std::rc::Rc;
use std::cell::RefCell;
use std::mem;
use std::thread;

use std::collections::HashMap;

#[derive(Clone)]
pub struct SymbolTable {
    sym_table: Rc<RefCell<HashMap<String, Value>>>,
    nil: Value,
    unspecified: Value,
    holder: Value,
    bool_t: Value,
    bool_f: Value,
}

impl SymbolTable {
    fn new() -> Self {
        SymbolTable {
            sym_table: Rc::new(RefCell::new(HashMap::new())),
            nil: Datum::Nil.wrap(),
            unspecified: Datum::Unspecified.wrap(),
            holder: Datum::Holder.wrap(),
            bool_t: Datum::Boolean(true).wrap(),
            bool_f: Datum::Boolean(false).wrap()
        }
    }

    pub fn nil() -> Value {
        SymbolTable::singleton().nil.clone()
    }

    pub fn unspecified() -> Value {
        SymbolTable::singleton().unspecified.clone()
    }

    pub fn holder() -> Value {
        SymbolTable::singleton().holder.clone()
    }

    fn symbol_(&mut self, id: impl Into<String>) -> Value {
        let id = id.into();
        if let Some(symbol) = self.sym_table.borrow().get(&id) {
            return symbol.clone();
        }
        let symbol = Datum::Symbol(id.clone()).wrap();
        self.sym_table.borrow_mut().insert(id, symbol.clone());
        symbol
    }

    pub fn symbol(id: impl Into<String>) -> Value {
        SymbolTable::singleton().symbol_(id)
    }

    pub fn bool(b: bool) -> Value {
        if b {
            SymbolTable::singleton().bool_t.clone()
        } else {
            SymbolTable::singleton().bool_f.clone()
        }
    }

    pub fn number(n: f64) -> Value {
        Datum::Number(n).wrap()
    }

    pub fn character(c: char) -> Value {
        Datum::Character(c).wrap()
    }

    pub fn string(s: String) -> Value {
        Datum::String(s).wrap()
    }

    pub fn singleton() -> SymbolTable {
        static mut SINGLETON: *const SymbolTable = 0 as *const _;
        static ONCE: Once = ONCE_INIT;
        unsafe {
            ONCE.call_once(|| {
                let singleton = SymbolTable::new();
                SINGLETON = mem::transmute(Box::new(singleton));
            });
            (*SINGLETON).clone()
        }
    }


}
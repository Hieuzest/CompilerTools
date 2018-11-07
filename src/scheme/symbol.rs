use super::beam::*;
use super::env::*;

use std::collections::HashMap;

pub struct SymbolTable {
    table: HashMap<String, Value>
}

impl SymbolTable {
    pub fn new() -> Self {
        SymbolTable {
            table: HashMap::new()
        }
    }

    pub fn get(&mut self, id: impl Into<String>) -> Value {
        let id = id.into();
        if let Some(symbol) = self.table.get(&id) {
            return symbol.clone();
        }
        let symbol = Datum::Symbol(id.clone()).wrap();
        self.table.insert(id, symbol.clone());
        symbol
    }
}
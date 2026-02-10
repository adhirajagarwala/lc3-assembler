use std::collections::HashMap;

// TODO-LOW: Consider using BTreeMap instead of HashMap + Vec for automatic ordering without duplication
#[derive(Debug, Clone)]
pub struct SymbolTable {
    map: HashMap<String, u16>,
    order: Vec<String>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            order: Vec::new(),
        }
    }

    pub fn insert(&mut self, label: String, address: u16) {
        if !self.map.contains_key(&label) {
            self.order.push(label.clone());
        }
        self.map.insert(label, address);
    }

    pub fn get(&self, label: &str) -> Option<u16> {
        self.map.get(label).copied()
    }

    pub fn contains(&self, label: &str) -> bool {
        self.map.contains_key(label)
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn print_table(&self) {
        println!("//\tSymbol Name\tAddress");
        println!("//\t-----------\t-------");
        for label in &self.order {
            let addr = self.map[label];
            println!("//\t{}\t\tx{:04X}", label, addr);
        }
    }

    // TODO-LOW: iter() uses map[label] which can panic - use .get() or .expect()
    pub fn iter(&self) -> impl Iterator<Item = (&str, u16)> {
        self.order.iter().map(move |label| (label.as_str(), self.map[label]))
    }
}

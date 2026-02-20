use std::collections::HashMap;

// TODO-LOW: Consider using BTreeMap instead of HashMap + Vec for automatic ordering without duplication
#[derive(Debug, Clone)]
pub struct SymbolTable {
    map: HashMap<String, u16>,
    order: Vec<String>,
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            order: Vec::new(),
        }
    }

    pub fn insert(&mut self, label: String, address: u16) {
        use std::collections::hash_map::Entry;
        // Entry API: single hash lookup instead of contains_key + insert (two lookups).
        // On a new label (Vacant), clone the key to maintain insertion order in `order`.
        // On an existing label (Occupied), just update the value without touching `order`.
        match self.map.entry(label) {
            Entry::Vacant(e) => {
                self.order.push(e.key().clone());
                e.insert(address);
            }
            Entry::Occupied(mut e) => {
                *e.get_mut() = address;
            }
        }
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

    pub fn iter(&self) -> impl Iterator<Item = (&str, u16)> {
        self.order.iter().map(move |label| {
            (
                label.as_str(),
                self.map
                    .get(label)
                    .copied()
                    .expect("Label in order but not in map"),
            )
        })
    }
}

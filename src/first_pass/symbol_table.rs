/// Insertion-ordered symbol table mapping labels to addresses.
///
/// Uses a single `Vec<(String, u16)>` instead of the previous `HashMap + Vec`
/// approach, eliminating string duplication. Lookups are O(n) linear scans,
/// which is perfectly adequate for LC-3 programs (typically <50 labels).
#[derive(Debug, Clone)]
pub struct SymbolTable {
    entries: Vec<(String, u16)>,
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn insert(&mut self, label: String, address: u16) {
        // If the label already exists, update its address in place.
        // Otherwise, append a new entry to preserve insertion order.
        if let Some(entry) = self.entries.iter_mut().find(|(l, _)| l == &label) {
            entry.1 = address;
        } else {
            self.entries.push((label, address));
        }
    }

    pub fn get(&self, label: &str) -> Option<u16> {
        self.entries
            .iter()
            .find(|(l, _)| l == label)
            .map(|(_, addr)| *addr)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn print_table(&self) {
        println!("//\tSymbol Name\tAddress");
        println!("//\t-----------\t-------");
        for (label, addr) in &self.entries {
            println!("//\t{label}\t\tx{addr:04X}");
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, u16)> {
        self.entries.iter().map(|(l, a)| (l.as_str(), *a))
    }
}

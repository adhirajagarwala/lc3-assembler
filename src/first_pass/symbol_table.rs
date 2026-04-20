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

    pub fn iter(&self) -> impl Iterator<Item = (&str, u16)> {
        self.entries.iter().map(|(l, a)| (l.as_str(), *a))
    }

    /// Return entries sorted alphabetically by label name.
    pub fn sorted_by_name(&self) -> Vec<(&str, u16)> {
        let mut v: Vec<(&str, u16)> = self.iter().collect();
        v.sort_by(|a, b| a.0.cmp(b.0));
        v
    }

    /// Return entries sorted by address (ascending).
    pub fn sorted_by_address(&self) -> Vec<(&str, u16)> {
        let mut v: Vec<(&str, u16)> = self.iter().collect();
        v.sort_by_key(|(_, addr)| *addr);
        v
    }
}

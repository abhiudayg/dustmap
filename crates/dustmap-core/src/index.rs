use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntryId(pub u32);

pub const ROOT_ID: EntryId = EntryId(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntryKind {
    File,
    Directory,
    Symlink,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Coverage {
    Complete,
    Partial,
    #[default]
    NotScanned,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub id: EntryId,
    pub parent: EntryId,
    pub name: String,
    pub kind: EntryKind,
    pub logical_bytes: u64,
    pub allocated_bytes: Option<u64>,
    pub device: u64,
    pub inode: u64,
    pub link_count: u32,
    pub modified_secs: i64,
    pub is_hidden: bool,
    pub is_sparse: bool,
    pub is_symlink_target: Option<String>,
    pub permission_denied: bool,
    pub coverage: Coverage,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DirAggregate {
    pub logical_total: u64,
    pub allocated_total: u64,
    pub item_count: u64,
    pub coverage: Coverage,
}

#[derive(Debug, Default)]
pub struct Index {
    entries: Vec<Entry>,
    children: Vec<Vec<EntryId>>,
    aggregates: Vec<DirAggregate>,
}

impl Index {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_entry(&mut self, entry: Entry) -> EntryId {
        let id = EntryId(self.entries.len() as u32);
        let parent = entry.parent;
        self.entries.push(entry);
        self.children.push(Vec::new());
        self.aggregates.push(DirAggregate::default());
        if id != ROOT_ID && (parent.0 as usize) < self.children.len() {
            self.children[parent.0 as usize].push(id);
        }
        id
    }

    pub fn entry(&self, id: EntryId) -> Option<&Entry> {
        self.entries.get(id.0 as usize)
    }

    pub fn children(&self, id: EntryId) -> &[EntryId] {
        self.children
            .get(id.0 as usize)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    pub fn aggregate(&self, id: EntryId) -> Option<&DirAggregate> {
        self.aggregates.get(id.0 as usize)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Post-order aggregation pass — call after all entries are added.
    pub fn compute_aggregates(&mut self) {
        let n = self.entries.len();
        // Process in reverse insertion order (approximates post-order for tree built top-down)
        for i in (0..n).rev() {
            let entry = &self.entries[i];
            let logical = entry.logical_bytes;
            let allocated = entry.allocated_bytes.unwrap_or(logical);
            let is_dir = matches!(entry.kind, EntryKind::Directory);

            // Seed with own size
            self.aggregates[i].logical_total = if is_dir { 0 } else { logical };
            self.aggregates[i].allocated_total = if is_dir { 0 } else { allocated };
            self.aggregates[i].item_count = if is_dir { 0 } else { 1 };

            // Accumulate children
            let children: Vec<EntryId> = self.children[i].clone();
            for child_id in children {
                let ci = child_id.0 as usize;
                let child_logical = self.aggregates[ci].logical_total;
                let child_allocated = self.aggregates[ci].allocated_total;
                let child_count = self.aggregates[ci].item_count;
                self.aggregates[i].logical_total += child_logical;
                self.aggregates[i].allocated_total += child_allocated;
                self.aggregates[i].item_count += child_count;
            }
        }
    }

    /// Return children sorted by logical size descending.
    pub fn children_sorted_by_size(&self, id: EntryId) -> Vec<EntryId> {
        let mut children = self.children(id).to_vec();
        children.sort_by(|a, b| {
            let sa = self
                .aggregates
                .get(a.0 as usize)
                .map(|ag| ag.logical_total)
                .unwrap_or(0);
            let sb = self
                .aggregates
                .get(b.0 as usize)
                .map(|ag| ag.logical_total)
                .unwrap_or(0);
            sb.cmp(&sa)
        });
        children
    }
}

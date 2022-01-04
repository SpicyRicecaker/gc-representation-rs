// use std::collections::VecDeque;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug)]
/// we'll have "stack" pointing to nodes on the heap
/// it's really not necessary but it's useful in representing the layout of the stack
pub struct Stack {
    pub roots: Vec<Node>,
}

impl Stack {
    /// number of roots you want to have
    pub fn new(num_roots: usize) -> Self {
        let mut roots = Vec::new();
        (0..num_roots).for_each(|_| {
            let node = Node {
                value: Some(0),
                ..Default::default()
            };
            roots.push(node);
        });

        Self { roots }
    }
    /// Provides a breadth-first ordered print of all the reachable values on the stack
    /// keep in mind the stack pooints into the heap
    pub fn dump_all<T: MemoryManager>(&self, heap: &T) -> Result<String> {
        // for each root
        let mut root_list = Vec::new();
        for root in &self.roots {
            // aggregate the dump of all children in roots
            let mut list_strings = Vec::new();
            for child in &root.children {
                list_strings.push(heap.dump(*child)?);
            }
            // compose all the list strings and add them to root_list
            root_list.push(format!("[{}] {}", root.value.unwrap(), list_strings.join(" - ")));
        }
        Ok(root_list.join("\n"))
    }
}

pub trait MemoryManager {
    fn alloc(&mut self, node: Node, stack: &mut Stack) -> Result<NodePointer>;
    fn collect(&mut self, stack: &mut Stack) -> Result<()>;
    // lifetime is elided here:
    // by one of the lifetime ellision rules: given &self or &mut self, we apply the lifetime of &self to all output lifetimes
    fn get(&self, node_pointer: NodePointer) -> Option<&Node>;
    fn get_mut(&mut self, node_pointer: NodePointer) -> Option<&mut Node>;
    // 
    fn committed_memory(&self) -> &[Node];
    fn committed_memory_mut(&mut self) -> &mut [Node];
    // 
    fn dump(&self, node_pointer: NodePointer) -> Result<String>;
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct NodePointer {
    idx: usize,
}

impl From<usize> for NodePointer {
    fn from(idx: usize) -> Self {
        Self { idx }
    }
}

impl From<NodePointer> for usize {
    fn from(node_pointer: NodePointer) -> Self {
        node_pointer.idx
    }
}

/// A node represents some kind of object in memory
/// A node doesn't technically need a parent pointer, it's literally just there for eye candy
#[derive(Debug, Default)]
pub struct Node {
    pub forwarding_address: Option<NodePointer>,
    pub parent: Option<NodePointer>,
    pub children: Vec<NodePointer>,
    pub value: Option<u32>,
}

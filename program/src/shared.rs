// use std::collections::VecDeque;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Clone)]
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
            root_list.push(format!(
                "[{}] {}",
                root.value.unwrap(),
                list_strings.join(" - ")
            ));
        }
        Ok(root_list.join("\n"))
    }

    // TODO bugged, because with multiple roots pointing to something on the
    // heap we would traverse over it again. However, we only have 1 root on the
    // heap so it doesn't currently matter.
    pub fn sum<T: MemoryManager>(&self, heap: &T) -> Result<u64> {
        // for each root
        let mut sum = 0;
        for root in &self.roots {
            // aggregate the dump of all children in roots
            for child in &root.children {
                sum += heap.sum(*child)?;
            }
        }
        Ok(sum)
    }
}

pub trait MemoryManager {
    fn alloc(&mut self, node: Node, stack: &mut Stack) -> Result<NodePointer>;
    fn collect(&mut self, stack: &mut Stack) -> Result<()>;
    // lifetime is elided here: by one of the lifetime ellision rules: given
    // &self or &mut self, we apply the lifetime of &self to all output
    // lifetimes
    fn get(&self, node_pointer: NodePointer) -> Option<&Node>;
    fn get_mut(&mut self, node_pointer: NodePointer) -> Option<&mut Node>;
    fn free(&self) -> usize;
    fn dump(&self, node_pointer: NodePointer) -> Result<String> {
        let mut elements = Vec::new();

        let mut visited: HashSet<NodePointer> = HashSet::new();

        let mut worklist: VecDeque<NodePointer> = VecDeque::new();
        worklist.push_back(node_pointer);

        while let Some(node_pointer) = worklist.pop_front() {
            if !visited.contains(&node_pointer) {
                visited.insert(node_pointer);

                let node = self.get(node_pointer).unwrap();
                if let Some(value) = node.value {
                    elements.push(value.to_string());
                }
                for child in &node.children {
                    worklist.push_back(*child);
                }
            }
        }
        Ok(elements.join(", "))
    }
    fn sum(&self, node_pointer: NodePointer) -> Result<u64> {
        let mut sum = 0;

        let mut visited: HashSet<NodePointer> = HashSet::new();

        let mut worklist: VecDeque<NodePointer> = VecDeque::new();
        worklist.push_back(node_pointer);

        while let Some(node_pointer) = worklist.pop_front() {
            if !visited.contains(&node_pointer) {
                visited.insert(node_pointer);

                let node = self.get(node_pointer).unwrap();
                if let Some(value) = node.value {
                    sum += value as u64;
                }
                for child in &node.children {
                    worklist.push_back(*child);
                }
            }
        }
        Ok(sum)
    }
}

use std::collections::{HashSet, VecDeque};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
#[derive(Debug, Default, Clone)]
#[repr(align(8))]
pub struct Node {
    pub forwarding_address: Option<NodePointer>,
    pub parent: Option<NodePointer>,
    pub children: Vec<NodePointer>,
    pub value: Option<u32>,
}

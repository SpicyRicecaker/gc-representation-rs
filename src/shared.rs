use std::collections::VecDeque;

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
    pub fn dump_all<T: MemoryManager>(&self, heap: &T) -> Result<()> {
        // create a new queue. We use queues for breadth-first search, stack for depth-first search
        let mut queue = VecDeque::new();
        // add roots of stack first obviously
        for root in &self.roots {
            queue.push_back(root);
        }
        // we pop from front of queue
        while let Some(node) = queue.pop_front() {
            // print its value
            if let Some(value) = node.value {
                print!("{} ", value);
            }
            // then add the rest of its children to the back of the queue
            for child in &node.children {
                queue.push_back(heap.get(*child).unwrap());
            }
        }
        Ok(())
    }
}

pub trait MemoryManager {
    fn alloc(&mut self, node: Node, stack: &mut Stack) -> Result<NodePointer>;
    fn collect(&mut self, stack: &mut Stack) -> Result<()>;
    // lifetime is elided here:
    // by one of the lifetime ellision rules: given &self or &mut self, we apply the lifetime of &self to all output lifetimes
    fn get(&self, node_pointer: NodePointer) -> Option<&Node>;
    fn get_mut(&mut self, node_pointer: NodePointer) -> Option<&mut Node>;
    fn committed_memory(&self) -> &[Node];
    fn committed_memory_mut(&mut self) -> &mut [Node];
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

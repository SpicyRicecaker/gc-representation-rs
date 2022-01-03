use crate::shared::{MemoryManager, Node, NodePointer, Stack};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// This mark-compact algorithm uses the LISP-2 style sliding algorithm
/// Heap includes the graph data structure, and acts pretty much like an arena
pub struct StopAndCopyHeap {
    // should be at the start of the heap
    pub from_space: usize,
    // should be at the middle of the heap
    pub to_space: usize,
    // extent is always len() / 2
    pub extent: usize,
    // where we allocate from
    pub free: usize,
    //
    pub committed_memory: Vec<Node>,
}

impl StopAndCopyHeap {
    pub fn init(size: usize) -> Self {
        let mut committed_memory: Vec<Node> = Vec::new();
        for _ in 0..size {
            committed_memory.push(Node::default());
        }
        let extent = size / 2;
        Self {
            from_space: 0,
            to_space: extent,
            free: 0,
            committed_memory,
            extent,
        }
    }
}

impl StopAndCopyHeap {
    // breadth-first traversal of node, printing out
    pub fn dump(&self, node: NodePointer) {
        if let Some(n) = self.committed_memory.get(node.idx) {
            if let Some(value) = n.value {
                print!("{} ", value);
            }
            for child in &n.children {
                self.dump(*child);
            }
        }
    }

    /// copy function
    pub fn copy(&mut self, node_pointer: NodePointer) -> Result<NodePointer> {
        // if object has a forwarding address, it means that we've already moved it over to to space, so we can just give it its reference
        dbg!(node_pointer, stapi::value(node_pointer, self)?);
        if let Some(forwarding_address) = stapi::forwarding_address(node_pointer, self)? {
            Ok(forwarding_address)
        } else {
            let new_node_pointer = NodePointer::new(self.free);
            // otherwise, the new nodepointer value of this object will be whatever free there is
            // now use .swap() to move nodepointer current location to its new location free
            self.committed_memory
                .swap(node_pointer.idx, new_node_pointer.idx);

            // and remember to set the forwarding address of the moved nodepointer to none
            stapi::set_forwarding_address(new_node_pointer, None, self)?;

            // now update the old forwarding address to include itself
            // keep in mind that this object in to space is complete garbage except for the forwarding address part
            stapi::set_forwarding_address(node_pointer, Some(new_node_pointer), self)?;

            // also remember to bump free
            self.free += 1;

            // finally we can return the new_node_pointer
            Ok(new_node_pointer)
        }
    }
}

impl MemoryManager for StopAndCopyHeap {
    // allocates a new node
    // we can just add a new node and return its id
    fn alloc(&mut self, stack: &mut Stack) -> Result<NodePointer> {
        // check if free is going over fromspace + tospace
        if self.free >= self.from_space + self.to_space {
            // we need to run gc
            self.collect(stack)?;
        }
        if self.free >= self.committed_memory.len() {
            return Err("gg collection didn't result in any amount of garbage collected".into());
        }

        let node = Node::default();
        // set the node id to where the top of the heap is
        let node_pointer = NodePointer::new(self.free);
        // add it to the heap
        self.committed_memory[node_pointer.idx] = node;
        // bump the free pointer
        self.free += 1;

        Ok(node_pointer)
    }

    /// stop-and-copy algorithm
    fn collect(&mut self, stack: &mut Stack) -> Result<()> {
        // first we swap from space with tospace
        {
            // literally std::mem swap them. They're both locations, neither is size
            println!("{:?}", self.committed_memory);
            println!("to space before swap {} {}", self.to_space, self.from_space);
            std::mem::swap(&mut self.to_space, &mut self.from_space);
            println!("to space after swap {} {}", self.to_space, self.from_space);
            // set our free pointer to the new space
            self.free = self.to_space;
        }
        // the scan also starts from the beginning of the to_space
        let mut scan = self.free;

        // dbg!(&self.committed_memory);
        stack.dump_all(self)?;
        println!("SUCESSSCUESUCEUSCUESUCSEUCESUCSEUC");
        // next we populate the initial "working list" with roots
        {
            // copy the roots over
            // this technically adds them to the worklist
            for root in &mut stack.roots {
                for child in &mut root.children {
                    println!("12312312312");
                    dbg!(*child);
                    dbg!("value of child", stapi::value(*child, self)?);
                    // make sure to update the root refs to point in the right place
                    // *child = self.copy(*child)?;
                }
            }
        }

        // now we process all the references of the nodes in the worklist as well
        {
            // you might be wondering...
            // how do we do `for each node in worklist`?
            //
            // well, so long as the scan does not catch up to free
            // that is, so as long as we have not processed every single "copied" oject on the heap, keep on going
            while scan < self.free {
                let scan_node_pointer = NodePointer::new(scan);
                // get all references, or children of the object that was recently copied to tospace
                //
                //
                //  ... to copy the references over,
                for i in 0..stapi::get(scan_node_pointer, self)?.children.len() {
                    // set the reference to whatever the forwarding address stored inside the reference is, or copy it
                    //
                    // TL;DR the reference should now be pointing to copied objects in the tospace no matter what
                    //
                    // I don't know how I fked the api up this bad to make this function look like this jargon but yea it happens
                    stapi::get_mut(stapi::get(scan_node_pointer, self)?.children[i], self)?
                        .forwarding_address =
                        Some(stapi::get(scan_node_pointer, self)?.children[i]);
                    // the references get added to the worklist automatically
                }
                // don't forget to bump the scan pointer
                scan += 1;
            }
        }

        // now we know that our freed space is just committed_memory.len() / 2 - self.free
        Ok(())
    }

    // we provide a slice from from space to to space!
    fn committed_memory(&self) -> &[Node] {
        &self.committed_memory[self.from_space..self.from_space + self.extent]
    }

    fn committed_memory_mut(&mut self) -> &mut [Node] {
        &mut self.committed_memory[self.from_space..self.from_space + self.extent]
    }
}

/// don't have time, but would clean this up in two steps
/// first make this api a Memory Manager trait implementation
/// make a seperate mandatory internal API that uses the front API's code by default
pub mod stapi {
    type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
    use crate::shared::{Node, NodePointer};

    use super::StopAndCopyHeap;

    pub fn add_child(
        parent_node_pointer: NodePointer,
        child_node_pointer: NodePointer,
        heap: &mut StopAndCopyHeap,
    ) -> Result<()> {
        if let Some(child) = heap.committed_memory.get_mut(child_node_pointer.idx) {
            child.parent = Some(parent_node_pointer);
        } else {
            return Err("child not found while trying to add child to parent".into());
        }

        if let Some(parent) = heap.committed_memory.get_mut(parent_node_pointer.idx) {
            parent.children.push(child_node_pointer);
            Ok(())
        } else {
            Err("parent not found while adding child".into())
        }
    }

    pub fn children(
        parent_node_pointer: NodePointer,
        heap: &StopAndCopyHeap,
    ) -> Result<Vec<NodePointer>> {
        if let Some(parent) = heap.committed_memory.get(parent_node_pointer.idx) {
            Ok(parent.children.clone())
        } else {
            Err("parent not found while getting children".into())
        }
    }

    pub fn parent(
        child_node_pointer: NodePointer,
        heap: &StopAndCopyHeap,
    ) -> Result<Option<NodePointer>> {
        if let Some(child) = heap.committed_memory.get(child_node_pointer.idx) {
            Ok(child.parent)
        } else {
            Err("child not found while getting parent".into())
        }
    }

    pub fn value(node_pointer: NodePointer, heap: &StopAndCopyHeap) -> Result<Option<u32>> {
        if let Some(node) = heap.committed_memory.get(node_pointer.idx) {
            Ok(node.value)
        } else {
            Err("node not found when trying to get value".into())
        }
    }
    pub fn set_value(
        node_pointer: NodePointer,
        value: Option<u32>,
        heap: &mut StopAndCopyHeap,
    ) -> Result<()> {
        if let Some(node) = heap.committed_memory.get_mut(node_pointer.idx) {
            node.value = value;
            Ok(())
        } else {
            Err("node not found when trying to set value".into())
        }
    }

    pub fn forwarding_address(
        node_pointer: NodePointer,
        heap: &StopAndCopyHeap,
    ) -> Result<Option<NodePointer>> {
        if let Some(node) = heap.committed_memory.get(node_pointer.idx) {
            Ok(node.forwarding_address)
        } else {
            Err("node not found when trying to get forwarding address".into())
        }
    }
    pub fn set_forwarding_address(
        node_pointer: NodePointer,
        forwarding_address: Option<NodePointer>,
        heap: &mut StopAndCopyHeap,
    ) -> Result<()> {
        if let Some(node) = heap.committed_memory.get_mut(node_pointer.idx) {
            node.forwarding_address = forwarding_address;
            Ok(())
        } else {
            Err("node not found when trying to set value".into())
        }
    }

    pub fn get(node_pointer: NodePointer, heap: &StopAndCopyHeap) -> Result<&Node> {
        heap.committed_memory
            .get(node_pointer.idx)
            .ok_or_else(|| "node not found when trying to get it immutably from heap".into())
    }

    pub fn get_mut(node_pointer: NodePointer, heap: &mut StopAndCopyHeap) -> Result<&mut Node> {
        heap.committed_memory
            .get_mut(node_pointer.idx)
            .ok_or_else(|| "node not found when trying to get it mutably from heap".into())
    }
    /// deletes some children given a parent node pointer and a mutable reference to heap
    /// returns a result of nothing
    ///
    /// keep in mind that we cannot delete a node directly given a node pointer
    /// because we don't know exactly how many nodes are pointing to it
    /// we would have to do a complete traversal of the tree just to delete a node (which defeats the point of having this data structure)
    /// so instead we only allow deletions from parent
    ///
    /// this also means that a tree data structure doesn't quite perfectly
    /// represent the memory of a program, since trees only have one parent reference anyway
    pub fn delete_some_children(
        parent_node_pointer: NodePointer,
        number_to_remove: usize,
        heap: &mut StopAndCopyHeap,
    ) -> Result<()> {
        // go to parent
        if let Some(parent) = heap.committed_memory.get_mut(parent_node_pointer.idx) {
            // delete x number of children
            // we can just delete 5 children for now
            for _ in 0..number_to_remove {
                parent.children.pop();
            }
        } else {
            return Err("(delete) node to delete children from does not exist".into());
        };
        Ok(())
    }
}

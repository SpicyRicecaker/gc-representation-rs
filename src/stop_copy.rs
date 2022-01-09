use std::collections::VecDeque;

use log::trace;

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
    pub top: usize,
    pub committed_memory: Vec<Node>,
}

impl StopAndCopyHeap {
    /// make sure you allocate double the amount you actually need
    pub fn init(size: usize) -> Self {
        let mut committed_memory: Vec<Node> = Vec::new();
        for _ in 0..size {
            committed_memory.push(Node::default());
        }
        let extent = size / 2;
        let from_space = extent;
        // TODO *LEFT IS REQUIRED FOR TESTS RN, SHOULD EXPOSE A PUBLIC API*
        // we're starting on the left side of the heap
        let to_space = 0;
        // the extent is just half of the total size of the heap
        let free = to_space;
        // the top, or maximum value we can hold before having to reallocate
        // we precompute this to save calculations for the mutator
        let top = to_space + extent;
        // to_space also happens to be

        // we call the active heap "to_space" for now, because during collect we'll flip it so it will actually become from_space
        Self {
            from_space,
            to_space,
            top,
            free,
            extent,
            committed_memory,
        }
    }
}

impl MemoryManager for StopAndCopyHeap {
    // allocates a new node
    // we can just add a new node and return its id
    fn alloc(&mut self, node: Node, stack: &mut Stack) -> Result<NodePointer> {
        // check if free is going over fromspace + tospace
        if self.free >= self.top {
            log::trace!("exceeded from space, must run garbage collector");
            // we need to run gc
            self.collect(stack)?;
        }
        if self.free >= self.top {
            return Err("gg collection didn't result in any amount of garbage collected".into());
        }

        // set the node id to where the top of the heap is
        let node_pointer = NodePointer::from(self.free);
        // add it to the heap
        self.committed_memory[usize::from(node_pointer)] = node;
        // bump the free pointer
        self.free += 1;

        Ok(node_pointer)
    }

    /// stop-and-copy algorithm
    fn collect(&mut self, stack: &mut Stack) -> Result<()> {
        // first we swap from space with tospace
        {
            // literally std::mem swap them. They're both locations, neither is size
            // println!("{:?}", self.committed_memory);
            // println!("to space before swap {} {}", self.to_space, self.from_space);
            trace!("before swapping the heap, self.top (maxiumum of whatever is fromspace) is {}, self.from_space is {}, self.to_space is {}, and self.free is {}", self.top, self.from_space, self.to_space, self.free);
            std::mem::swap(&mut self.from_space, &mut self.to_space);
            // set free to be at the bot of the new to_space
            self.free = self.to_space;
            // set top to be to_space plus the extent
            self.top = self.to_space + self.extent;

            // println!("to space after swap {} {}", self.to_space, self.from_space);
            // set our free pointer to the new space
            // self.free = self.to_space;
            trace!("after swapping the heap, self.top (maxiumum of whatever is fromspace) is {}, self.from_space is {}, self.to_space is {}, and self.free is {}", self.top, self.from_space, self.to_space, self.free);
        }
        // the scan also starts from the beginning of the to_space
        let mut scan = self.free;

        // dbg!(&self.committed_memory);
        // stack.dump_all(self)?;
        // println!("SUCESSSCUESUCEUSCUESUCSEUCESUCSEUC");
        // next we populate the initial "working list" with roots
        {
            // copy the roots over
            // this technically adds them to the worklist
            for root in &mut stack.roots {
                for child in &mut root.children {
                    // println!("12312312312");
                    // dbg!(*child);
                    // dbg!("value of child", stapi::value(*child, self)?);
                    // make sure to update the root refs to point in the right place
                    *child = self.copy(*child)?;
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
                let scan_node_pointer = NodePointer::from(scan);
                // get all references, or children of the object that was recently copied to tospace
                //
                //
                //  ... to copy the references over,
                for i in 0..self.get(scan_node_pointer).unwrap().children.len() {
                    // set the reference to whatever the forwarding address stored inside the reference is, or copy it
                    //
                    // TL;DR the reference should now be pointing to copied objects in the tospace no matter what
                    self.get_mut(scan_node_pointer).unwrap().children[i] =
                        self.copy(self.get(scan_node_pointer).unwrap().children[i])?;
                    // the references get added to the worklist automatically
                }
                // don't forget to bump the scan pointer
                scan += 1;
            }
        }

        // now we know that our freed space is just committed_memory.len() / 2 - self.free
        Ok(())
    }

    #[inline(always)]
    fn get(&self, node_pointer: NodePointer) -> Option<&Node> {
        self.committed_memory.get(usize::from(node_pointer))
    }

    #[inline(always)]
    fn get_mut(&mut self, node_pointer: NodePointer) -> Option<&mut Node> {
        self.committed_memory.get_mut(usize::from(node_pointer))
    }

    // breadh-first traversal of node, printing out
    fn dump(&self, node_pointer: NodePointer) -> Result<String> {
        let mut elements = Vec::new();

        let mut worklist: VecDeque<NodePointer> = VecDeque::new();
        worklist.push_back(node_pointer);

        while let Some(node_pointer) = worklist.pop_front() {
            let node = self.get(node_pointer).unwrap();
            if let Some(value) = node.value {
                elements.push(value.to_string());
            }
            for child in &node.children {
                worklist.push_back(*child);
            }
        }
        Ok(elements.join(", "))
    }

    fn free(&self) -> usize {
        // `free` on stop-and-copy should be subtracted by to space
        self.free - self.to_space
    }
}

impl StopAndCopyHeap {
    /// copy function
    pub fn copy(&mut self, node_pointer: NodePointer) -> Result<NodePointer> {
        // if object has a forwarding address, it means that we've already moved it over to to space, so we can just give it its reference
        // dbg!(node_pointer, stapi::value(node_pointer, self)?);
        if let Some(forwarding_address) = self.get(node_pointer).unwrap().forwarding_address {
            Ok(forwarding_address)
        } else {
            let new_node_pointer = NodePointer::from(self.free);
            // otherwise, the new nodepointer value of this object will be whatever free there is
            // now use .swap() to move nodepointer current location to its new location free
            self.committed_memory
                .swap(usize::from(node_pointer), usize::from(new_node_pointer));

            // and remember to set the forwarding address of the moved nodepointer to none
            self.get_mut(new_node_pointer).unwrap().forwarding_address = None;

            // now update the old forwarding address to include itself
            // keep in mind that this object in to space is complete garbage except for the forwarding address part
            self.get_mut(node_pointer).unwrap().forwarding_address = Some(new_node_pointer);

            // also remember to bump free
            self.free += 1;

            // finally we can return the new_node_pointer
            Ok(new_node_pointer)
        }
    }
}

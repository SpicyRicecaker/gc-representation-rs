use std::collections::VecDeque;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug)]
/// we'll have "stack" pointing to nodes on the heap
/// it's really not necessary but it's useful in representing the layout of the stack
pub struct Stack {
    pub roots: Vec<Node>,
}

impl Stack {
    /// Provides a breadth-first ordered print of all the reachable values on the stack
    /// keep in mind the stack pooints into the heap
    pub fn dump_all(&self, heap: &Heap) -> Result<()> {
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
                queue.push_back(api::get(*child, heap)?);
            }
        }
        Ok(())
    }
}

/// Heap includes the graph data structure, and acts pretty much like an arena
pub struct Heap {
    // the `top` of the memory != strip.len()
    // because we don't want to have to zero them out if we don't need to, and don't want to push / pop the vec
    // especially when we're compacting
    pub committed_memory: Vec<Node>,
    // // when the length of vector len reaches the max
    // pub max_size: usize,
    // // the size of the top, where the last piece of recognizable memory is. 1 less than strip.len()
    // pub top: usize,
    // pub max: usize,
    pub free: usize,
}

impl Heap {
    // allocates a new node
    // we can just add a new node and return its id
    pub fn alloc(&mut self, stack: &mut Stack) -> Result<NodePointer> {
        // if our free pointer is over the committed memory length
        if self.free < self.committed_memory.len() {
            let node = Node::default();
            // set the node id to where the top of the heap is
            let node_pointer = NodePointer::new(self.free);
            // add it to the heap
            self.committed_memory[node_pointer.idx] = node;
            // bump the free pointer
            self.free += 1;

            Ok(node_pointer)
        } else {
            // we need to run gc
            let freed = self.collect(stack)?;
            if freed != 0 {
                self.alloc(stack)
            } else {
                Err("gg collection didn't result in any amount of garbage collected".into())
            }
        }
    }

    /// mark-compact algorithm
    pub fn collect(&mut self, stack: &mut Stack) -> Result<usize> {
        dbg!("exceeded heap size!");
        // # mark first
        // create marking bitmap
        // which isn't actually going to be a bitmap but rather a stack of node indices
        let mut marked_nodes: Vec<NodePointer> = Vec::new();

        // create worklist, which is going to be a queue, since we're doing breadth-first traversal
        // now do a breadth-first traversal of the tree,
        let mut worklist: VecDeque<NodePointer> = VecDeque::new();
        // populate worklist with children from stack first obviously
        for root in &stack.roots {
            for child in &root.children {
                worklist.push_back(*child);
            }
        }
        // we pop from front of queue
        while let Some(node) = worklist.pop_front() {
            // we mark it because it means it's accessible
            marked_nodes.push(node);
            // then add the rest of its children to the back of the queue
            for child_node_pointer in &api::get(node, self)?.children {
                worklist.push_back(*child_node_pointer);
            }
        }
        // now all our objects should be marked

        let mut free = 0;
        // # compact next
        {
            // first step is to calculate new locations of all objects

            // we iterate over all objects in the heap
            //
            // free starts at 0
            // if it is marked,
            for marked in &marked_nodes {
                // set its forwarding address equal to free
                api::set_forwarding_address(*marked, Some(NodePointer::new(free)), self)?;
                // then bump free
                free += 1;
            }
        }

        {
            // now we update object references
            //
            //
            // for every marked parent
            for marked in &marked_nodes {
                //   for every child of the marked node
                for i in 0..api::get(*marked, self)?.children.len() {
                    //      get the actual child_node's forwarding address
                    let forwarding_address =
                        api::forwarding_address(api::get(*marked, self)?.children[i], self)?
                            .unwrap();
                    //      then set the child_node to child node's forwarding address
                    api::get_mut(api::get(*marked, self)?.children[i], self)?.forwarding_address =
                        Some(forwarding_address);
                }
            }
        }

        {
            // actually move the objects
            //   for every marked node
            for marked in marked_nodes {
                let forwarding_address = api::forwarding_address(marked, self)?.unwrap();
                // swap node's current position with node's forwarding position
                self.committed_memory
                    .swap(marked.idx, forwarding_address.idx);
            }
        }
        self.free = free;
        Ok(self.committed_memory.len() - self.free)
    }

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
}

pub mod api {
    type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
    use super::{Heap, Node, NodePointer};

    pub fn add_child(
        parent_node_pointer: NodePointer,
        child_node_pointer: NodePointer,
        heap: &mut Heap,
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

    pub fn children(parent_node_pointer: NodePointer, heap: &Heap) -> Result<Vec<NodePointer>> {
        if let Some(parent) = heap.committed_memory.get(parent_node_pointer.idx) {
            Ok(parent.children.clone())
        } else {
            Err("parent not found while getting children".into())
        }
    }

    pub fn parent(child_node_pointer: NodePointer, heap: &Heap) -> Result<Option<NodePointer>> {
        if let Some(child) = heap.committed_memory.get(child_node_pointer.idx) {
            Ok(child.parent)
        } else {
            Err("child not found while getting parent".into())
        }
    }

    pub fn value(node_pointer: NodePointer, heap: &Heap) -> Result<Option<u32>> {
        if let Some(node) = heap.committed_memory.get(node_pointer.idx) {
            Ok(node.value)
        } else {
            Err("node not found when trying to get value".into())
        }
    }
    pub fn set_value(node_pointer: NodePointer, value: Option<u32>, heap: &mut Heap) -> Result<()> {
        if let Some(node) = heap.committed_memory.get_mut(node_pointer.idx) {
            node.value = value;
            Ok(())
        } else {
            Err("node not found when trying to set value".into())
        }
    }

    pub fn forwarding_address(
        node_pointer: NodePointer,
        heap: &Heap,
    ) -> Result<Option<NodePointer>> {
        if let Some(node) = heap.committed_memory.get(node_pointer.idx) {
            Ok(node.forwarding_address)
        } else {
            Err("node not found when trying to get value".into())
        }
    }
    pub fn set_forwarding_address(
        node_pointer: NodePointer,
        forwarding_address: Option<NodePointer>,
        heap: &mut Heap,
    ) -> Result<()> {
        if let Some(node) = heap.committed_memory.get_mut(node_pointer.idx) {
            node.forwarding_address = forwarding_address;
            Ok(())
        } else {
            Err("node not found when trying to set value".into())
        }
    }

    pub fn get(node_pointer: NodePointer, heap: &Heap) -> Result<&Node> {
        heap.committed_memory
            .get(node_pointer.idx)
            .ok_or_else(|| "node pointer not found".into())
    }

    pub fn get_mut(node_pointer: NodePointer, heap: &mut Heap) -> Result<&mut Node> {
        heap.committed_memory
            .get_mut(node_pointer.idx)
            .ok_or_else(|| "node pointer not found".into())
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
        heap: &mut Heap,
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

#[derive(Debug, Default, Clone, Copy)]
pub struct NodePointer {
    pub idx: usize,
}

impl NodePointer {
    pub fn new(idx: usize) -> Self {
        Self { idx }
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn sanity_garbage_collection_check() {
        let roots = {
            let mut roots = Vec::new();
            (0..1).for_each(|i| {
                let node = Node {
                    value: Some(i),
                    ..Default::default()
                };
                roots.push(node);
            });
            roots
        };
        const SIZE: usize = 4;
        let mut committed_memory = Vec::new();
        for _ in 0..SIZE {
            committed_memory.push(Node::default())
        }

        // initializing the stack
        let mut stack = Stack { roots };
        // this is memory allocation
        let mut heap = Heap {
            committed_memory,
            free: 0,
        };

        // add one child to root
        let temp = heap.alloc(&mut stack).unwrap();
        api::set_value(temp, Some(1), &mut heap).unwrap();
        stack.roots[0].children.push(temp);

        let mut second_node_pointer = None;
        // add one child to child of root
        for i in 0..stack.roots[0].children.len() {
            let temp = heap.alloc(&mut stack).unwrap();
            second_node_pointer = Some(temp);
            api::set_value(temp, Some(2), &mut heap).unwrap();
            api::add_child(stack.roots[0].children[i], temp, &mut heap).unwrap();
        }

        // add two children to child of child of root
        for i in 0..stack.roots[0].children.len() {
            let children = api::children(stack.roots[0].children[i], &heap).unwrap();
            for child in children {
                for i in 33..35 {
                    // iterations+=1;
                    let temp = heap.alloc(&mut stack).unwrap();
                    api::set_value(temp, Some(i), &mut heap).unwrap();
                    api::add_child(child, temp, &mut heap).unwrap();
                }
            }
        }
        stack.dump_all(&heap).unwrap();

        // should panic
        // heap.alloc(&mut stack).unwrap();

        // remove 1 child from second
        api::delete_some_children(second_node_pointer.unwrap(), 1, &mut heap).unwrap();

        println!("clean stuff");

        stack.dump_all(&heap).unwrap();
        // now add another child
        for i in 0..stack.roots[0].children.len() {
            let children = api::children(stack.roots[0].children[i], &heap).unwrap();
            for child in children {
                // iterations+=1;
                let temp = heap.alloc(&mut stack).unwrap();
                api::set_value(temp, Some(9999), &mut heap).unwrap();
                api::add_child(child, temp, &mut heap).unwrap();
            }
        }
        stack.dump_all(&heap).unwrap();
    }
}

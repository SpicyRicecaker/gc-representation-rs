// use bitvec::prelude::*;
use std::collections::VecDeque;

use crate::shared::*;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// This mark-compact algorithm uses the LISP-2 style sliding algorithm Heap
/// includes the graph data structure, and acts pretty much like an arena
#[derive(Clone)]
pub struct MarkCompactHeap {
    // the `top` of the memory != strip.len() because we don't want to have to
    // zero them out if we don't need to, and don't want to push / pop the vec
    // especially when we're compacting
    pub committed_memory: Vec<Node>,
    // pub marked_node_pointers: Vec<NodePointer>,
    // // when the length of vector len reaches the max pub max_size: usize, //
    // the size of the top, where the last piece of recognizable memory is. 1
    // less than strip.len() pub top: usize, pub max: usize,
    pub free: usize,
}

impl MarkCompactHeap {
    pub fn init(size: usize) -> Self {
        let mut committed_memory: Vec<Node> = Vec::new();
        for _ in 0..size {
            committed_memory.push(Node::default());
        }
        // let marked_node_pointers = Vec::with_capacity(committed_memory.len());
        Self {
            committed_memory,
            // marked_node_pointers,
            free: 0,
        }
    }
}

impl MemoryManager for MarkCompactHeap {
    // allocates a new node
    // we can just add a new node and return its id
    fn alloc(&mut self, node: Node, stack: &mut Stack) -> Result<NodePointer> {
        // if our free pointer is over the committed memory length
        if self.free >= self.committed_memory.len() {
            // we need to run gc
            self.collect(stack)?;
        }
        if self.free >= self.committed_memory.len() {
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

    // mark-compact algorithm
    fn collect(&mut self, stack: &mut Stack) -> Result<()> {
        // log::debug!("exceeded heap size! now calling collect function for mark_compact");

        // this block contains the code to mark all reachable objects
        {
            // first create a worklist, which is going to be a queue, since
            // we're doing breadth-first traversal
            let mut worklist: VecDeque<NodePointer> = VecDeque::new();

            // populate the worklist with children reachable from the roots
            for root in &stack.roots {
                for child in &root.children {
                    worklist.push_back(*child);
                }
            }

            // then we just keep on taking from the worklist until it's empty
            while let Some(node) = worklist.pop_front() {
                // if the node isn't marked (already)
                if !self.is_marked(node) {
                    // we mark it because it means it's accessible
                    self.mark(node);
                    // then add the rest of its children to the back of the queue
                    for child_node_pointer in &self.get(node).unwrap().children {
                        worklist.push_back(*child_node_pointer);
                    }
                }
            }
        }
        // now all our reachable objects should be marked, everything that isn't
        // is considered garbo we only care about the marked objects from now on

        // log::trace!(
        //     "marked_node_pointers is this long: {}",
        //     marked_node_pointers.len()
        // );
        // let iterator = marked_node_pointers.iter().enumerate().filter(|(_, v)| **v);
        // log::trace!(
        //     "we have this many marked nodes: {}",
        //     iterator.clone().count()
        // );

        // the next three blocks contain the compact code
        // free starts at 0, the beginning of the point which we wish to compact to
        let mut free = 0;

        // 1. the first step is to calculate new locations of all objects
        {
            // we iterate over all objects in the heap
            for idx in 0..self.free {
                // if it is marked,
                if self.is_marked(idx.into()) {
                    // set its forwarding address equal to free
                    self.set_forwarding_address(idx.into(), free.into());
                    // then bump free by the object's size
                    free += 1;
                }
            }
        }

        // 2. the next step is to update object references
        {
            // for every marked parent, set the parent's references to the
            // child's forwarding address
            for idx in 0..self.free {
                if self.is_marked(idx.into()) {
                    let node = NodePointer::from(idx);

                    //  for every child that the marked parent node holds
                    for i in 0..self.get_mut(node).unwrap().children.len() {
                        let child_node_pointer = self.get(node).unwrap().children[i];

                        // get the child node's forwarding address
                        let forwarding_address = self
                            .get(child_node_pointer)
                            .unwrap()
                            .forwarding_address
                            .unwrap();

                        // then update the parent's reference to the child's forwarding address
                        self.get_mut(node).unwrap().children[i] = forwarding_address;
                    }
                }
            }
        }

        // 3. actually move the objects
        {
            //  for every marked node
            for idx in 0..self.free {
                if self.is_marked(idx.into()) {
                    let node = NodePointer::from(idx);

                    // unset the forwarding address of the object that's about
                    // to be moved, consequently unmarking it for the next
                    // collection cycle
                    let forwarding_address = self.get(node).unwrap().forwarding_address.unwrap();
                    self.get_mut(node).unwrap().forwarding_address = None;

                    // swap node's current position with node's forwarding
                    // position...  but only if they're not already in the right
                    // place! This is an advantage of mark-compact over stop
                    // copy
                    if usize::from(forwarding_address) != usize::from(node) {
                        self.committed_memory
                            .swap(usize::from(node), usize::from(forwarding_address));
                    }
                }
            }
        }
        // set our new free pointer to the compacted point
        self.free = free;
        Ok(())
    }

    #[inline(always)]
    fn node_pointer_from_usize(&self, idx: usize) -> NodePointer {
        NodePointer::from(idx)
    }

    #[inline(always)]
    fn get(&self, node_pointer: NodePointer) -> Option<&Node> {
        self.committed_memory.get(usize::from(node_pointer))
    }

    #[inline(always)]
    fn get_mut(&mut self, node_pointer: NodePointer) -> Option<&mut Node> {
        self.committed_memory.get_mut(usize::from(node_pointer))
    }

    #[inline(always)]
    fn free(&self) -> usize {
        self.free
    }

    fn heap_size(&self) -> usize {
        self.committed_memory.len()
    }
}

impl MarkCompactHeap {
    #[inline]
    fn is_marked(&self, node_pointer: NodePointer) -> bool {
        self.get(node_pointer).unwrap().forwarding_address.is_some()
    }
    #[inline]
    fn mark(&mut self, node_pointer: NodePointer) {
        self.get_mut(node_pointer).unwrap().forwarding_address = Some(NodePointer::default());
    }
    #[inline]
    fn set_forwarding_address(
        &mut self,
        node_pointer: NodePointer,
        forwarding_address: NodePointer,
    ) {
        self.get_mut(node_pointer).unwrap().forwarding_address = Some(forwarding_address);
    }
}

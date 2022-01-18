use crate::mark_compact::*;
use crate::shared::*;
use crate::stop_copy::StopAndCopyHeap;
// use crate::stop_copy::*;
use std::collections::VecDeque;
// use std::time::{Duration, Instant};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

mod actual;
mod sanity;
mod collection;

pub fn init_log() {
    let _ = env_logger::builder().is_test(true).try_init();
}

pub fn recursively_add_children<T: MemoryManager>(
    parent_node_pointer: NodePointer,
    max_objects: usize,
    stack: &mut Stack,
    heap: &mut T,
) -> Result<()> {
    // we want to keep on adding objects to root until we get to `max_objects`.len
    let mut worklist = VecDeque::new();
    worklist.push_back(parent_node_pointer);

    // then keep on adding children until we get to the object limit
    let width = 2;

    // we start at 1 because we already allocated a node to the heap earlier
    let mut current_objects = 0;
    // for each node
    while let Some(parent_node_pointer) = worklist.pop_front() {
        // attempt to add children equal to `width`
        for _ in 0..width {
            // if we're below the object limit
            if current_objects < max_objects {
                // create a new node on the heap
                let node = Node {
                    value: Some(current_objects as u32),
                    ..Default::default()
                };
                let child_node_pointer = heap.alloc(node, stack).unwrap();
                // add the node as a child of node_pointer
                heap.get_mut(parent_node_pointer)
                    .unwrap()
                    .children
                    .push(child_node_pointer);
                // push the child into the worklist
                worklist.push_back(child_node_pointer);
                current_objects += 1;
            }
        }
    }
    Ok(())
}

pub fn seed_root<T: MemoryManager>(stack: &mut Stack, heap: &mut T) -> Result<NodePointer> {
    let temp = Node {
        value: Some(1),
        ..Default::default()
    };
    let node_pointer = heap.alloc(temp, stack).unwrap();
    stack.roots[0].children.push(node_pointer);
    Ok(node_pointer)
}
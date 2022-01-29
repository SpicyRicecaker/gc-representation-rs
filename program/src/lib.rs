use std::collections::VecDeque;

use rand::prelude::*;
use rand_pcg::Pcg64;

use shared::{MemoryManager, Node, NodePointer, Stack};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub mod shared;

pub mod mark_compact;
pub mod stop_copy;

// testing stuff below

#[cfg(test)]
pub mod tests;

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

// pub fn get_heap_boring<T: MemoryManager>(
//     stack: &mut Stack,
//     heap: &mut T,
//     heap_size: usize,
// ) -> Result<()> {
//     {
//         let child_node_pointer = seed_root(stack, heap).unwrap();
//         recursively_add_children(child_node_pointer, heap_size - 1, stack, heap).unwrap();
//     }

//     // create number of links equal to number of nodes, randomly from anywhere to anywhere
//     let mut rng = Pcg64::seed_from_u64(1234);
//     {
//         for _ in 0..heap_size {
//             // generate two random numbers
//             let (first, second) = (
//                 rng.gen_range(0..heap_size),
//                 rng.gen_range(heap_size / 2..heap_size),
//             );
//             // link child before point of removal to parent
//             heap.get_mut(NodePointer::from(first))
//                 .unwrap()
//                 .children
//                 .push(NodePointer::from(second));
//         }
//     }

//     // randomly remove links half the population of the latter half
//     {
//         for _ in 0..(heap_size / 10) {
//             // generate two random numbers
//             let num = rng.gen_range(heap_size / 200..heap_size / 100);
//             // link child before point of removal to parent
//             heap.get_mut(NodePointer::from(num)).unwrap().children.pop();
//         }
//     }

//     // run gc
//     // heap.collect(stack).unwrap();

//     // stack.dump_all(heap).unwrap();

//     Ok(())
// }

/// takes in a reference to stack and heap, and returns a clone deleted with
/// specification to the ratio included
pub fn make_garbage<T: MemoryManager + Clone>(
    stack: &mut Stack,
    heap: &mut T,
    heap_size: usize,
    garbage_ratio: f32,
) -> Result<()> {
    let mut rng = Pcg64::seed_from_u64(1234);

    let layers = (1. + heap_size as f32).log2().floor() as u32;

    let low = 12;
    let high = 14;

    let lowest_layer = 2_usize.pow(low);
    let highest_layer = 2_usize.pow(high);

    let num_nodes_between_layer = highest_layer - lowest_layer;
    // the average amount of nodes that are connected to a child in the range of
    // highest layer to lowest layer
    // let mean_children = (2_usize.pow(layers - low) + 2_usize.pow(layers - high)) / 2;

    // let normalized_deletions = num_nodes_between_layer - heap_size / mean_children;

    // println!("hello");
    // dbg!(
    //     // normalized_deletions,
    //     // mean_children,
    //     num_nodes_between_layer,
    //     highest_layer,
    //     lowest_layer,
    //     layers
    // );
    // randomly remove links, a ratio of the number of the population
    {
        for _ in 0..((num_nodes_between_layer * 4) as f32 * garbage_ratio) as usize {
            // generate two random numbers
            let num = rng.gen_range(lowest_layer..highest_layer);
            // link child before point of removal to parent
            heap.get_mut(NodePointer::from(num)).unwrap().children.pop();
        }
    }
    Ok(())
}

/// takes in a mutable reference to stack and heap, linking them to a ratio of
/// around 2 edges to : 1 node?
pub fn link_heap<T: MemoryManager>(
    stack: &mut Stack,
    heap: &mut T,
    heap_size: usize,
) -> Result<()> {
    // get number of powers of two (so we can know how many layers of binary
    // tree there are)

    {
        let child_node_pointer = seed_root(stack, heap).unwrap();
        recursively_add_children(child_node_pointer, heap_size - 1, stack, heap).unwrap();
    }

    // create number of links equal to number of nodes, randomly from anywhere to anywhere
    let mut rng = Pcg64::seed_from_u64(1234);
    {
        for _ in 0..heap_size {
            // generate two random numbers
            let (first, second) = (
                rng.gen_range(0..heap_size),
                rng.gen_range(heap_size / 2..heap_size),
            );
            // link child before point of removal to parent
            heap.get_mut(NodePointer::from(first))
                .unwrap()
                .children
                .push(NodePointer::from(second));
        }
    }
    // run gc
    // heap.collect(stack).unwrap();
    // stack.dump_all(heap).unwrap();
    Ok(())
}

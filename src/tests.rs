use crate::mark_compact::*;
use crate::shared::*;
// use crate::stop_copy::*;
use std::collections::VecDeque;
// use std::time::{Duration, Instant};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn init_log() {
    let _ = env_logger::builder().is_test(true).try_init();
}

fn recursively_add_children<T: MemoryManager>(
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

fn seed_root<T: MemoryManager>(stack: &mut Stack, heap: &mut T) -> Result<NodePointer> {
    let temp = Node {
        value: Some(1),
        ..Default::default()
    };
    let node_pointer = heap.alloc(temp, stack).unwrap();
    stack.roots[0].children.push(node_pointer);
    Ok(node_pointer)
}

#[test]
fn sanity_garbage_collection_check_mark_and_compact() {
    init_log();
    // initializing the stack
    const STACK_SIZE: usize = 1;
    let mut stack = Stack::new(STACK_SIZE);
    // initializing the heap
    const HEAP_SIZE: usize = 5;
    let mut heap = MarkCompactHeap::init(HEAP_SIZE);

    // first add one child (allocated on the heap) to our root on the stack
    let child_node_pointer = seed_root(&mut stack, &mut heap).unwrap();
    // then add 4 objects
    recursively_add_children(child_node_pointer, HEAP_SIZE - 1, &mut stack, &mut heap).unwrap();

    // explicit snapshot of heap here should show 1+4 objects in total
    log::trace!("{}", stack.dump_all(&heap).unwrap());

    assert_eq!(stack.dump_all(&heap).unwrap(), "[0] 1, 0, 1, 2, 3");

    log::debug!("successfully filled heap with {} objects", HEAP_SIZE);

    // ...but this action should now panic, since the heap is full
    assert!(heap.alloc(Node::default(), &mut stack).is_err());

    // now we remove 0 from 1's children
    // which should free up *3* slots after garbage collection
    log::trace!("now removing children of one node");

    heap.get_mut(child_node_pointer).unwrap().children.remove(0);

    log::trace!("{}", stack.dump_all(&heap).unwrap());

    assert_eq!(stack.dump_all(&heap).unwrap(), "[0] 1, 1");

    log::debug!("successfully removed *3* children from heap");

    // now this shouldn't panic, because we should automatically be able to clear heap
    recursively_add_children(child_node_pointer, 3, &mut stack, &mut heap).unwrap();
    // but this should
    assert!(heap.alloc(Node::default(), &mut stack).is_err());
    log::trace!("{}", stack.dump_all(&heap).unwrap());

    log::info!("this garbage collector works");
}

#[test]
fn actual_test_mark_compact() -> Result<()> {
    init_log();
    const STACK_SIZE: usize = 1;
    const HEAP_SIZE: usize = 1_000_000;
    // initializing the stack
    let mut stack = Stack::new(STACK_SIZE);
    // initializing the heap
    let mut heap = MarkCompactHeap::init(HEAP_SIZE);

    // first add one child (allocated on the heap) to our root on the stack
    {
        let child_node_pointer = seed_root(&mut stack, &mut heap).unwrap();

        // fill up the heap with an ungodly amount of nodes
        recursively_add_children(child_node_pointer, HEAP_SIZE - 1, &mut stack, &mut heap).unwrap();

        // heap freed
        log::debug!("this is the size of the filled heap: {}/{}", heap.free, HEAP_SIZE);
    }

    // the way that the tree was filled, nodes at the end of the heap are at the bottom of the tree
    // let's add some interesting refs from parent to children and children to parent
    // assume that we're removing an object at ~8000
    {
        // link parent to child after point of removal
        heap.get_mut(NodePointer::from(100))
            .unwrap()
            .children
            .push(NodePointer::from(16383));
        // link parent to child before point of removal
        heap.get_mut(NodePointer::from(100))
            .unwrap()
            .children
            .push(NodePointer::from(300));
        // link child after point of removal to parent
        heap.get_mut(NodePointer::from(8191))
            .unwrap()
            .children
            .push(NodePointer::from(500));
        // link child before point of removal to parent
        heap.get_mut(NodePointer::from(5000))
            .unwrap()
            .children
            .push(NodePointer::from(400));
        
        // cyclic data structure that should be removed
        heap.get_mut(NodePointer::from(9000))
            .unwrap()
            .children
            .push(NodePointer::from(10_000));
        heap.get_mut(NodePointer::from(10_000))
            .unwrap()
            .children
            .push(NodePointer::from(9000));
    }

    {
        // now delete a few nodes
        // parent to child
        log::trace!(
            "parent to delete child from from found: {:#?}",
            heap.get_mut(NodePointer::from(8000)).unwrap().children
        );
        heap.get_mut(NodePointer::from(8000))
            .unwrap()
            .children
            .pop();
        heap.alloc(Node::default(), &mut stack).unwrap();
        log::debug!(
            "this is the size of the cleaned up heap: {}/{}",
            heap.free,
            HEAP_SIZE
        );
    }

    // allocate a bunch of garbage just to be sure
    for _ in 0..100 {
        heap.alloc(Node::default(), &mut stack).unwrap();
    }

    // top-level roots, every thing else on stack
    //              a    1       // stack
    //           /    \    \
    //          b      c    2     // heap, and below
    //        / \     / \
    //       d   e   f   g
    //      / \ / \ / \ /
    //     h  i j k l m n
    Ok(())
}

// #[test]
// fn sanity_garbage_collection_check_stop_and_copy() {
//     const STACK_SIZE: usize = 1;
//     const HEAP_SIZE: usize = 8;
//     // initializing the stack
//     let mut stack = Stack::new(STACK_SIZE);
//     // initializing the heap
//     let mut heap = StopAndCopyHeap::init(HEAP_SIZE);

//     // add one child to root
//     let temp = heap.alloc(&mut stack).unwrap();
//     api::set_value(temp, Some(1), &mut heap).unwrap();
//     stack.roots[0].children.push(temp);

//     let mut second_node_pointer = None;
//     // add one child to child of root
//     for i in 0..stack.roots[0].children.len() {
//         let temp = heap.alloc(&mut stack).unwrap();
//         second_node_pointer = Some(temp);
//         api::set_value(temp, Some(2), &mut heap).unwrap();
//         api::add_child(stack.roots[0].children[i], temp, &mut heap).unwrap();
//     }

//     // add two children to child of child of root
//     for i in 0..stack.roots[0].children.len() {
//         let children = api::children(stack.roots[0].children[i], &heap).unwrap();
//         for child in children {
//             for i in 33..35 {
//                 // iterations+=1;
//                 let temp = heap.alloc(&mut stack).unwrap();
//                 api::set_value(temp, Some(i), &mut heap).unwrap();
//                 api::add_child(child, temp, &mut heap).unwrap();
//             }
//         }
//     }
//     // stack.dump_all(&heap).unwrap();

//     // should panic
//     // heap.alloc(&mut stack).unwrap();

//     // remove 1 child from second
//     api::delete_some_children(second_node_pointer.unwrap(), 1, &mut heap).unwrap();

//     // println!("clean stuff");

//     // stack.dump_all(&heap).unwrap();
//     // now add another child
//     for i in 0..stack.roots[0].children.len() {
//         let children = api::children(stack.roots[0].children[i], &heap).unwrap();
//         for child in children {
//             // iterations+=1;
//             let temp = heap.alloc(&mut stack).unwrap();
//             api::set_value(temp, Some(9999), &mut heap).unwrap();
//             api::add_child(child, temp, &mut heap).unwrap();
//         }
//     }
//     // stack.dump_all(&heap).unwrap();
// }

// #[test]
// fn insane_mark_compact() {
//     // we spawn a sht ton of garbage
//     const STACK_SIZE: usize = 1;
//     const HEAP_SIZE: usize = 40000;
//     // initializing the stack
//     let mut stack = Stack::new(STACK_SIZE);
//     // initializing the heap
//     let mut heap = MarkCompactHeap::init(HEAP_SIZE);

//     dbg!("start");

//     let start = Instant::now();

//     for _ in 0..100_000_000 {
//         heap.alloc(&mut stack).unwrap();
//     }

//     dbg!("done", start.elapsed());
// }

// #[test]
// fn insane_stop_copy() {
//     // we spawn a sht ton of garbage
//     const STACK_SIZE: usize = 1;
//     const HEAP_SIZE: usize = 80000;
//     // initializing the stack
//     let mut stack = Stack::new(STACK_SIZE);
//     // initializing the heap
//     let mut heap = StopAndCopyHeap::init(HEAP_SIZE);

//     dbg!("start");
//     let start = Instant::now();

//     for _ in 0..100_000_000 {
//         heap.alloc(&mut stack).unwrap();
//     }

//     dbg!("done", start.elapsed());
// }

// #[test]
// fn actual_test_stop_copy() {}

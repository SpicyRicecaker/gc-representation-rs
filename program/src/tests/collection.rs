use std::env;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

use rand::prelude::*;
use rand_pcg::Pcg64;

use crate::{init_log, recursively_add_children, seed_root};

use super::*;

use std::time::Instant;

fn random_garbage_collection<T: MemoryManager>(
    stack: &mut Stack,
    heap: &mut T,
    heap_size: usize,
) -> Result<()> {
    init_log();

    let mut instant = Instant::now();

    {
        let child_node_pointer = seed_root(stack, heap).unwrap();
        recursively_add_children(child_node_pointer, heap_size - 1, stack, heap).unwrap();
        log::debug!(
            "this is the size of the filled heap: {}/{}",
            heap.free(),
            heap_size
        );
        assert_eq!(heap.free(), heap_size);
    }

    log::info!("time it took to fill stuff up: {:#?}", instant.elapsed());

    instant = Instant::now();
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
    log::info!("time it took to link children: {:#?}", instant.elapsed());

    instant = Instant::now();
    // randomly remove links half the population of the latter half
    {
        for _ in heap_size / 2..heap_size {
            // generate two random numbers
            let num = rng.gen_range(100..10_000);
            // link child before point of removal to parent
            heap.get_mut(NodePointer::from(num)).unwrap().children.pop();
        }
    }
    log::info!("time it took to remove children: {:#?}", instant.elapsed());

    instant = Instant::now();
    // run gc
    heap.collect(stack).unwrap();
    log::info!(
        "this is the size of the cleaned up heap: {}/{}",
        heap.free(),
        heap_size
    );
    log::info!("CRITICAL time it took to collect: {:#?}", instant.elapsed());

    instant = Instant::now();

    stack.dump_all(heap).unwrap();

    log::info!(
        "CRITICAL time it took to traverse: {:#?}",
        instant.elapsed()
    );

    Ok(())
}

#[test]
fn mark_compact_random() {
    const STACK_SIZE: usize = 1;
    let heap_size: usize = env::var("HEAP_SIZE").unwrap().parse::<usize>().unwrap();
    // initializing the stack
    let mut stack = Stack::new(STACK_SIZE);
    // initializing the heap
    let mut heap = MarkCompactHeap::init(heap_size);

    random_garbage_collection(&mut stack, &mut heap, heap_size).unwrap();
}

#[test]
fn stop_and_copy_random() {
    const STACK_SIZE: usize = 1;
    let heap_size: usize = env::var("HEAP_SIZE").unwrap().parse::<usize>().unwrap() * 2;
    // initializing the stack
    let mut stack = Stack::new(STACK_SIZE);
    // initializing the heap
    let mut heap = StopAndCopyHeap::init(heap_size);

    random_garbage_collection(&mut stack, &mut heap, heap_size / 2).unwrap();
}

fn sum_garbage_collection<T: MemoryManager>(
    stack: &mut Stack,
    heap: &mut T,
    heap_size: usize,
) -> Result<()> {
    let node_pointer = heap.alloc(Node::default(), stack).unwrap();
    stack.roots[0].children.push(node_pointer);

    // dbg!(stack.dump_all(heap).unwrap());
    // 0, 1..., 8, 9
    recursively_add_children(node_pointer, heap_size - 1, stack, heap).unwrap();
    // the sum of all points should be 45
    assert_eq!(stack.sum(heap).unwrap(), 45);
    // also add some cyclic data structures
    heap.get_mut(NodePointer::from(heap_size - 1))
        .unwrap()
        .children
        .push(NodePointer::from(0));
    dbg!(stack.dump_all(heap).unwrap());
    // check again
    assert_eq!(stack.sum(heap).unwrap(), 45);

    Ok(())
}

#[test]
fn mark_compact_sum() {
    const STACK_SIZE: usize = 1;
    let heap_size: usize = 11;
    // initializing the stack
    let mut stack = Stack::new(STACK_SIZE);
    // initializing the heap
    let mut heap = MarkCompactHeap::init(heap_size);

    sum_garbage_collection(&mut stack, &mut heap, heap_size).unwrap();
}

#[test]
fn stop_and_copy_sum() {
    const STACK_SIZE: usize = 1;
    let heap_size: usize = 22;
    // initializing the stack
    let mut stack = Stack::new(STACK_SIZE);
    // initializing the heap
    let mut heap = MarkCompactHeap::init(heap_size);

    sum_garbage_collection(&mut stack, &mut heap, heap_size / 2).unwrap();
}

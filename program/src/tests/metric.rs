// use crate::{recursively_add_children, seed_root};


use crate::{get_heap, get_heap_boring};

use super::*;
/// tests each heap state, returns number of nodes before and number of nodes and connections after
#[test]
fn metric() -> Result<()> {
    // first test mark compact, heap
    {
        const STACK_SIZE: usize = 1;
        let heap_size: usize = 1_000_000;
        // initializing the stack
        let mut stack = Stack::new(STACK_SIZE);
        // initializing the heap
        let mut heap = MarkCompactHeap::init(heap_size);

        // now initialize the heap one way
        get_heap(&mut stack, &mut heap, heap_size).unwrap();
        println!(
            "mark compact heap with lots of children leftover: {:#?}",
            stack.count(&heap).unwrap()
        );
    }
    {
        const STACK_SIZE: usize = 1;
        let heap_size: usize = 1_000_000;
        // initializing the stack
        let mut stack = Stack::new(STACK_SIZE);
        // initializing the heap
        let mut heap = MarkCompactHeap::init(heap_size);

        // now initialize the heap one way
        get_heap_boring(&mut stack, &mut heap, heap_size).unwrap();
        println!(
            "mark compact heap with NOT ALOT of children leftover: {:#?}",
            stack.count(&heap).unwrap()
        );
    }
    {
        const STACK_SIZE: usize = 1;
        let heap_size: usize = 2_000_000;
        // initializing the stack
        let mut stack = Stack::new(STACK_SIZE);
        // initializing the heap
        let mut heap = MarkCompactHeap::init(heap_size);

        // now initialize the heap one way
        get_heap(&mut stack, &mut heap, heap_size / 2).unwrap();
        println!(
            "stop and copy heap with lots of children leftover: {:#?}",
            stack.count(&heap).unwrap()
        );
    }
    {
        const STACK_SIZE: usize = 1;
        let heap_size: usize = 2_000_000;
        // initializing the stack
        let mut stack = Stack::new(STACK_SIZE);
        // initializing the heap
        let mut heap = MarkCompactHeap::init(heap_size);

        // now initialize the heap one way
        get_heap_boring(&mut stack, &mut heap, heap_size / 2).unwrap();
        println!(
            "stop and copy heap with NOT A LOT of children leftover: {:#?}",
            stack.count(&heap).unwrap()
        );
    }
    Ok(())
}

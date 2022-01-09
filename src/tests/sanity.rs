use super::*;

fn sanity_garbage_collection<T: MemoryManager>(
    stack: &mut Stack,
    heap: &mut T,
    stack_size: usize,
    heap_size: usize,
) {
    init_log();

    // first add one child (allocated on the heap) to our root on the stack
    let child_node_pointer = seed_root(stack, heap).unwrap();
    // then add 4 objects
    recursively_add_children(child_node_pointer, heap_size - 1, stack, heap).unwrap();

    // explicit snapshot of heap here should show 1+4 objects in total
    log::trace!("{}", stack.dump_all(heap).unwrap());

    assert_eq!(stack.dump_all(heap).unwrap(), "[0] 1, 0, 1, 2, 3");

    log::debug!("successfully filled heap with {} objects", heap_size);

    // ...but this action should now panic, since the heap is full
    assert!(heap.alloc(Node::default(), stack).is_err());

    // now we remove 0 from 1's children
    // which should free up *3* slots after garbage collection
    log::trace!("now removing children of one node");

    heap.get_mut(child_node_pointer).unwrap().children.remove(0);

    log::trace!("{}", stack.dump_all(heap).unwrap());

    assert_eq!(stack.dump_all(heap).unwrap(), "[0] 1, 1");

    log::debug!("successfully removed *3* children from heap");

    // now this shouldn't panic, because we should automatically be able to clear heap
    recursively_add_children(child_node_pointer, 3, stack, heap).unwrap();
    // but this should
    assert!(heap.alloc(Node::default(), stack).is_err());
    log::trace!("{}", stack.dump_all(heap).unwrap());

    log::info!("this garbage collector works");
}

#[test]
fn mark_compact_sanity() {
    // initializing the stack
    const STACK_SIZE: usize = 1;
    let mut stack = Stack::new(STACK_SIZE);
    // initializing the heap
    const HEAP_SIZE: usize = 5;
    let mut heap = MarkCompactHeap::init(HEAP_SIZE);

    sanity_garbage_collection(&mut stack, &mut heap, STACK_SIZE, HEAP_SIZE);
}

#[test]
fn stop_copy_sanity() {
    // initializing the stack
    const STACK_SIZE: usize = 1;
    let mut stack = Stack::new(STACK_SIZE);
    // initializing the heap
    const HEAP_SIZE: usize = 10;
    let mut heap = StopAndCopyHeap::init(HEAP_SIZE);

    sanity_garbage_collection(&mut stack, &mut heap, STACK_SIZE, HEAP_SIZE);
}

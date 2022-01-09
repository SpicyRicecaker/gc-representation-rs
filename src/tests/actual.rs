use super::*;

fn actual_garbage_collection<T: MemoryManager>(
    stack: &mut Stack,
    heap: &mut T,
    stack_size: usize,
    heap_size: usize,
) -> Result<()> {
    init_log();

    // first add one child (allocated on the heap) to our root on the stack
    {
        let child_node_pointer = seed_root(stack, heap).unwrap();

        // fill up the heap with an ungodly amount of nodes
        recursively_add_children(child_node_pointer, heap_size - 1, stack, heap).unwrap();

        // heap freed
        log::debug!(
            "this is the size of the filled heap: {}/{}",
            heap.free(),
            heap_size
        );
        assert_eq!(heap.free(), 1_000_000);
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
        heap.alloc(Node::default(), stack).unwrap();
        log::debug!(
            "this is the size of the cleaned up heap: {}/{}",
            heap.free(),
            heap_size
        );
        assert_eq!(heap.free(), 999938);
    }

    // allocate a bunch of garbage just to be sure
    for _ in 0..100 {
        heap.alloc(Node::default(), stack).unwrap();
    }
    log::debug!(
        "this is the size of the cleaned up heap: {}/{}",
        heap.free(),
        heap_size
    );
    assert_eq!(heap.free(), 999975);

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

#[test]
fn mark_compact_actual() {
    const STACK_SIZE: usize = 1;
    const HEAP_SIZE: usize = 1_000_000;
    // initializing the stack
    let mut stack = Stack::new(STACK_SIZE);
    // initializing the heap
    let mut heap = MarkCompactHeap::init(HEAP_SIZE);

    actual_garbage_collection(&mut stack, &mut heap, STACK_SIZE, HEAP_SIZE).unwrap();
}

#[test]
fn stop_and_copy_actual() {
    const STACK_SIZE: usize = 1;
    const HEAP_SIZE: usize = 2_000_000;
    // initializing the stack
    let mut stack = Stack::new(STACK_SIZE);
    // initializing the heap
    let mut heap = StopAndCopyHeap::init(HEAP_SIZE);

    actual_garbage_collection(&mut stack, &mut heap, STACK_SIZE, HEAP_SIZE).unwrap();
}

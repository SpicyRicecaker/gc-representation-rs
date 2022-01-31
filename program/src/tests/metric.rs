// use crate::{recursively_add_children, seed_root};

use crate::{link_heap, make_garbage};

use super::*;
/// tests each heap state, returns number of nodes before and number of nodes and connections after
#[test]
fn bfs_dfs() -> Result<()> {
    // first test mark compact, heap
    let res;
    {
        const STACK_SIZE: usize = 1;
        let heap_size: usize = 1_000_000;
        // initializing the stack
        let mut stack = Stack::new(STACK_SIZE);
        // initializing the heap
        let mut heap = MarkCompactHeap::init(heap_size);

        // now initialize the heap one way
        link_heap(&mut stack, &mut heap).unwrap();
        make_garbage(&mut stack, &mut heap, 0.2).unwrap();
        // println!(
        //     "mark compact: {:#?} (nodes/connections)",
        //     stack.count(&heap).unwrap()
        // );
        res = Some(stack.sum_bfs(&heap).unwrap());
        // println!("out of a total stack size of {} nodes", heap_size);
        assert_eq!(res.unwrap(), stack.sum_dfs(&heap).unwrap());
        // println!("summation of bfs is equal to summation of dfs");
    }
    {
        const STACK_SIZE: usize = 1;
        let heap_size: usize = 2_000_000;
        // initializing the stack
        let mut stack = Stack::new(STACK_SIZE);
        // initializing the heap
        let mut heap = MarkCompactHeap::init(heap_size);

        // now initialize the heap one way
        link_heap(&mut stack, &mut heap).unwrap();
        make_garbage(&mut stack, &mut heap, 0.2).unwrap();
        // println!(
        //     "stop and copy heap with lots of children leftover: {:#?}",
        //     stack.count(&heap).unwrap()
        // );
        let res_2 = stack.sum_bfs(&heap).unwrap();
        assert_eq!(res_2, stack.sum_dfs(&heap).unwrap());
        // println!("summation of bfs is equal to summation of dfs");
        assert_eq!(res.unwrap(), res_2);
    }
    Ok(())
}

#[test]
fn metrics() -> Result<()> {
    let dead_objects_over_live_objects = [
        0., 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0, 1.1, 1.2, 1.3, 1.4, 1.5,
    ];

    const STACK_SIZE: usize = 1;
    let m_heap_size: usize = 1_000_000;
    // initializing the stack
    let m_stack = Stack::new(STACK_SIZE);
    // initializing the heap
    let m_heap = MarkCompactHeap::init(m_heap_size);

    let s_heap_size: usize = 2_000_000;
    // initializing the stack
    let s_stack = Stack::new(STACK_SIZE);
    // initializing the heap
    let s_heap = StopAndCopyHeap::init(s_heap_size);

    for ratio in dead_objects_over_live_objects {
        {
            let mut stack = m_stack.clone();
            let mut heap = m_heap.clone();
            // now initialize the heap one way
            link_heap(&mut stack, &mut heap).unwrap();
            make_garbage(&mut stack, &mut heap, ratio).unwrap();
            println!(
                "expected dead to live ratio: {}\nactual ratio: {:.3}",
                ratio,
                1. - stack.count(&heap).unwrap().0 as f32 / m_heap_size as f32,
            );
        }
        {
            let mut stack = s_stack.clone();
            let mut heap = s_heap.clone();
            // now initialize the heap one way
            link_heap(&mut stack, &mut heap).unwrap();
            make_garbage(&mut stack, &mut heap, ratio).unwrap();
            println!(
                "mark_compact: expected dead to live ratio: {}\nactual ratio: {:.3}",
                ratio,
                1. - stack.count(&heap).unwrap().0 as f32 / (s_heap_size / 2) as f32,
            );
        }
    }
    Ok(())
}

use crate::mark_compact::*;
use crate::shared::*;
use crate::stop_copy::*;
use std::time::{Duration, Instant};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn sanity_garbage_collection_check_mark_and_copy() {
    const STACK_SIZE: usize = 1;
    const HEAP_SIZE: usize = 4;
    // initializing the stack
    let mut stack = Stack::new(STACK_SIZE);
    // initializing the heap
    let mut heap = MarkCompactHeap::init(HEAP_SIZE);

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
    // stack.dump_all(&heap).unwrap();

    // should panic
    // heap.alloc(&mut stack).unwrap();

    // remove 1 child from second
    api::delete_some_children(second_node_pointer.unwrap(), 1, &mut heap).unwrap();

    // println!("clean stuff");

    // stack.dump_all(&heap).unwrap();
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
    // stack.dump_all(&heap).unwrap();
}

#[test]
fn sanity_garbage_collection_check_stop_and_copy() {
    const STACK_SIZE: usize = 1;
    const HEAP_SIZE: usize = 8;
    // initializing the stack
    let mut stack = Stack::new(STACK_SIZE);
    // initializing the heap
    let mut heap = StopAndCopyHeap::init(HEAP_SIZE);

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
    // stack.dump_all(&heap).unwrap();

    // should panic
    // heap.alloc(&mut stack).unwrap();

    // remove 1 child from second
    api::delete_some_children(second_node_pointer.unwrap(), 1, &mut heap).unwrap();

    // println!("clean stuff");

    // stack.dump_all(&heap).unwrap();
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
    // stack.dump_all(&heap).unwrap();
}

#[test]
fn insane_mark_compact() {
    // we spawn a sht ton of garbage
    const STACK_SIZE: usize = 1;
    const HEAP_SIZE: usize = 40000;
    // initializing the stack
    let mut stack = Stack::new(STACK_SIZE);
    // initializing the heap
    let mut heap = MarkCompactHeap::init(HEAP_SIZE);

    dbg!("start");

    let start = Instant::now();

    for _ in 0..100_000_000 {
        heap.alloc(&mut stack).unwrap();
    }

    dbg!("done", start.elapsed());
}

#[test]
fn insane_stop_copy() {
    // we spawn a sht ton of garbage
    const STACK_SIZE: usize = 1;
    const HEAP_SIZE: usize = 80000;
    // initializing the stack
    let mut stack = Stack::new(STACK_SIZE);
    // initializing the heap
    let mut heap = StopAndCopyHeap::init(HEAP_SIZE);

    dbg!("start");
    let start = Instant::now();

    for _ in 0..100_000_000 {
        heap.alloc(&mut stack).unwrap();
    }

    dbg!("done", start.elapsed());
}

#[test]
fn actual_test_mark_compact() -> Result<()>{
    // create a fixed memory heap that's "zerod" with empty nodes
    // also the max size of the heap, like when you pass a -XMX [size] flag into java vm
    // or the total amount of physical memory on a system
    const STACK_SIZE: usize = 100;
    const HEAP_SIZE: usize = 1_001_000;
    // initializing the stack
    let mut stack = Stack::new(STACK_SIZE);
    // initializing the heap
    let mut heap = MarkCompactHeap::init(HEAP_SIZE);

    // stack, 1st node
    // add 2 children
    // for each child, add 2 more children

    // add 20 children
    for _ in 0..100 {
        // allocate and set the value that the node holds to 1 (for first layer)
        let temp = heap.alloc(&mut stack)?;
        api::set_value(temp, Some(1), &mut heap)?;

        // add the node to the children
        stack.roots[0].children.push(temp);
    }
    // dbg!("added children to root");

    // for each child, add 20 more children
    for i in 0..stack.roots[0].children.len() {
        for _ in 0..100 {
            let temp = heap.alloc(&mut stack)?;
            api::set_value(temp, Some(2), &mut heap)?;
            api::add_child(stack.roots[0].children[i], temp, &mut heap)?;
        }
    }
    // dbg!(iterations_2);
    // for each child of child, add 20 more children
    for i in 0..stack.roots[0].children.len() {
        let children = api::children(stack.roots[0].children[i], &heap)?;
        for child in children {
            for _ in 0..50 {
                // iterations+=1;
                let temp = heap.alloc(&mut stack)?;
                api::set_value(temp, Some(3), &mut heap)?;
                api::add_child(child, temp, &mut heap)?;
                // println!("{}", heap.committed_memory.len());
            }
        }
    }
    // println!("{}", iterations);
    // now we have 20*20*12 + 20*20 + 20 total objects on heap, which is around 400*12 + 400 + 20 = 5220 objects

    // now remove some children at the second level
    for i in 0..stack.roots[0].children.len() {
        api::delete_some_children(stack.roots[0].children[i], 19, &mut heap)?;
    }

    // now the live objects are like 20*15*12 + 20*15 + 20
    // which is less than 5220 objects, but the heap is still technically full
    // println!("we're still running");
    for i in 0..stack.roots[0].children.len() {
        let children = api::children(stack.roots[0].children[i], &heap)?;
        for child in children {
            for _ in 0..120 {
                // iterations+=1;
                let temp = heap.alloc(&mut stack)?;
                api::set_value(temp, Some(3), &mut heap)?;
                api::add_child(child, temp, &mut heap)?;
                // println!("{}", heap.committed_memory.len());
            }
        }
    }
    // println!("yay garbage collection works");

    // stack.dump_all(&heap)?;
    // fs::write(
    //     "profiling/heap.txt",
    //     format!("{:#?}", heap.committed_memory),
    // )?;

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
fn actual_test_stop_copy() {}

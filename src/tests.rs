use crate::mark_compact::*;
use crate::shared::*;
use crate::stop_copy::*;

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
fn insane() {
    // we spawn a sht ton of garbage
    
}

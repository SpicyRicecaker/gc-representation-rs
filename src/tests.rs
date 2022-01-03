use crate::mark_compact::*;
use crate::shared::*;
use crate::stop_copy::*;

#[test]
fn sanity_garbage_collection_check() {
    let roots = {
        let mut roots = Vec::new();
        (0..1).for_each(|i| {
            let node = Node {
                value: Some(i),
                ..Default::default()
            };
            roots.push(node);
        });
        roots
    };
    const SIZE: usize = 4;
    let mut committed_memory = Vec::new();
    for _ in 0..SIZE {
        committed_memory.push(Node::default())
    }

    // initializing the stack
    let mut stack = Stack { roots };
    // this is memory allocation
    let mut heap = MarkCompactHeap {
        committed_memory,
        free: 0,
    };

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
    stack.dump_all(&heap).unwrap();

    // should panic
    // heap.alloc(&mut stack).unwrap();

    // remove 1 child from second
    api::delete_some_children(second_node_pointer.unwrap(), 1, &mut heap).unwrap();

    println!("clean stuff");

    stack.dump_all(&heap).unwrap();
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
    stack.dump_all(&heap).unwrap();
}
#[test]
fn sanity_garbage_collection_check_stopcopy() {
    let roots = {
        let mut roots = Vec::new();
        (0..1).for_each(|i| {
            let node = Node {
                value: Some(i),
                ..Default::default()
            };
            roots.push(node);
        });
        roots
    };
    const SIZE: usize = 4;
    let mut committed_memory = Vec::new();
    for _ in 0..SIZE {
        committed_memory.push(Node::default())
    }

    // initializing the stack
    let mut stack = Stack { roots };
    // this is memory allocation
    let mut heap = MarkCompactHeap {
        committed_memory,
        free: 0,
    };

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
    stack.dump_all(&heap).unwrap();

    // should panic
    // heap.alloc(&mut stack).unwrap();

    // remove 1 child from second
    api::delete_some_children(second_node_pointer.unwrap(), 1, &mut heap).unwrap();

    println!("clean stuff");

    stack.dump_all(&heap).unwrap();
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
    stack.dump_all(&heap).unwrap();
}

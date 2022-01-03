use gc_representation_rs::graph::*;

use std::fs;
use std::time;

// we're implementing mark-compact first
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let strip = Vec::new();
    let roots = {
        let mut roots = Vec::new();
        (0..5).for_each(|i| {
            let node = Node {
                value: Some(i),
                ..Default::default()
            };
            roots.push(node);
        });
        roots
    };
    // initializing the stack
    let mut stack = Stack { roots };
    // this is memory allocation
    let mut heap = Heap {
        committed_memory: strip,
        // the max size of the heap, like when you pass a -XMX [size] flag into java vm
        // or the total amount of physical memory on a system
        max_size: 5220,
        // top: 0,
        // max: UNITS - 1,
    };
    let start = time::Instant::now();
    // push some nodes onto the stack
    // stack.roots[0]
    // for root in stack.roots {
    //     root.children.push
    // }

    // stack, 1st node
    // add 2 children
    // for each child, add 2 more children

    // stack, 1st node
    let first = &mut stack.roots[0];

    let mut iterations = 0;
    // add 20 children
    for _ in 0..20 {
        iterations += 1;
        // allocate and set the value that the node holds to 1 (for first layer)
        let temp = heap.alloc()?;
        api::set_value(temp, Some(1), &mut heap)?;

        // add the node to the children
        first.children.push(temp);
    }

    // for each child, add 20 more children
    let mut iterations_2 = 0;
    for child in &first.children {
        iterations_2 += 1;
        for _ in 0..20 {
            iterations+=1;
            let temp = heap.alloc()?;
            api::set_value(temp, Some(2), &mut heap)?;
            api::add_child(*child, temp, &mut heap)?;
        }
    }
    dbg!(iterations_2);
    // for each child of child, add 20 more children
    for child in &first.children {
        let children = api::children(*child, &heap)?;
        for child in children {
            for _ in 0..12 {
                // iterations+=1;
                let temp = heap.alloc()?;
                api::set_value(temp, Some(3), &mut heap)?;
                api::add_child(child, temp, &mut heap)?;
                // println!("{}", heap.committed_memory.len());
            }
        }
    }
    println!("{}", iterations);
    // now we have 20*20*12 + 20*20 + 20 total objects on heap, which is around 400*12 + 400 + 20 = 5220 objects

    // now remove some children at the second level
    for child in &first.children {
        api::delete_some_children(*child, &mut heap)?;
    }

    // now the live objects are like 20*15*12 + 20*15 + 20
    // which is less than 5220 objects, but the heap is still technically full
    println!("we're still running");
    // let temp = heap.alloc()?;

    stack.dump_all(&heap)?;
    fs::write(
        "profiling/heap.txt",
        format!("{:#?}", heap.committed_memory),
    )?;

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

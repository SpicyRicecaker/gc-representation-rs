use gc_representation_rs::graph::*;

use std::time;
use std::fs;

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
        max_size: 500,
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
    // add 2 children
    for _ in 0..2 {
        // allocate
        let temp = heap.alloc()?;
        // 1 to represent first layer
        api::set_value(temp, Some(1), &mut heap)?;
        // add the node to the children
        first.children.push(temp);
    }
    // for each child, add 2 more children
    for child in &first.children {
        for _ in 0..2 {
            let temp = heap.alloc()?;
            api::set_value(temp, Some(2), &mut heap)?;
            api::add_child(*child, temp, &mut heap)?;
        }
    }
    // for each child of child, add 2 more children
    for child in &first.children {
        // DON'T KNOW IF THIS MATTERS BUT CLONE OUCH
        let children = api::children(*child, &heap)?;
        for child in children {
            for _ in 0..2 {
                let temp = heap.alloc()?;
                api::set_value(temp, Some(3), &mut heap)?;
                api::add_child(child, temp, &mut heap)?;
            }
        }
    }

    stack.dump_all(&heap)?;
    fs::write("profiling/heap.txt", format!("{:#?}", heap.committed_memory))?;
    

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

use criterion::{criterion_group, criterion_main, Criterion};

use gc_representation_rs::shared::{MemoryManager, Stack};

use gc_representation_rs::shared::*;
use gc_representation_rs::stop_copy::StopAndCopyHeap;
use gc_representation_rs::{mark_compact::*, recursively_add_children, seed_root};
// use crate::stop_copy::*;

// use std::time::{Duration, Instant};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

use rand::prelude::*;
use rand_pcg::Pcg64;
use std::env;

fn get_heap_boring<T: MemoryManager>(
    stack: &mut Stack,
    heap: &mut T,
    heap_size: usize,
) -> Result<()> {
    {
        let child_node_pointer = seed_root(stack, heap).unwrap();
        recursively_add_children(child_node_pointer, heap_size - 1, stack, heap).unwrap();
    }

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

    // randomly remove links half the population of the latter half
    {
        for _ in 0..(heap_size / 10) {
            // generate two random numbers
            let num = rng.gen_range(heap_size / 200..heap_size / 100);
            // link child before point of removal to parent
            heap.get_mut(NodePointer::from(num)).unwrap().children.pop();
        }
    }

    // run gc
    // heap.collect(stack).unwrap();

    // stack.dump_all(heap).unwrap();

    Ok(())
}

fn get_heap<T: MemoryManager>(stack: &mut Stack, heap: &mut T, heap_size: usize) -> Result<()> {
    {
        let child_node_pointer = seed_root(stack, heap).unwrap();
        recursively_add_children(child_node_pointer, heap_size - 1, stack, heap).unwrap();
    }

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

    // randomly remove links half the population of the latter half
    {
        for _ in 0..(((heap_size / 100) - heap_size / 200) / 2) {
            // generate two random numbers
            let num = rng.gen_range(heap_size / 200..heap_size / 100);
            // link child before point of removal to parent
            heap.get_mut(NodePointer::from(num)).unwrap().children.pop();
        }
    }

    // run gc
    // heap.collect(stack).unwrap();

    // stack.dump_all(heap).unwrap();

    Ok(())
}

fn collect<T: MemoryManager>(stack: &mut Stack, heap: &mut T) {
    heap.collect(stack).unwrap()
}

fn mark_compact_random_benchmark(c: &mut Criterion) {
    const STACK_SIZE: usize = 1;
    let heap_size: usize = env::var("HEAP_SIZE").unwrap().parse::<usize>().unwrap();
    // initializing the stack
    let mut stack = Stack::new(STACK_SIZE);
    // initializing the heap
    let mut heap = MarkCompactHeap::init(heap_size);

    get_heap(&mut stack, &mut heap, heap_size).unwrap();

    c.bench_function("collect mark compact", |b| {
        b.iter_batched(
            || (stack.clone(), heap.clone()),
            |(mut stack, mut heap)| collect(&mut stack, &mut heap),
            criterion::BatchSize::SmallInput,
        )
    });

    c.bench_function("traverse mark compact", |b| b.iter(|| stack.sum(&heap)));

    stack.roots[0].children.pop();
    collect(&mut stack, &mut heap);

    get_heap_boring(&mut stack, &mut heap, heap_size).unwrap();

    c.bench_function("[full clear] collect mark compact", |b| {
        b.iter_batched(
            || (stack.clone(), heap.clone()),
            |(mut stack, mut heap)| collect(&mut stack, &mut heap),
            criterion::BatchSize::SmallInput,
        )
    });

    c.bench_function("[full clear] traverse mark compact", |b| {
        b.iter(|| stack.sum(&heap))
    });
}

fn stop_and_copy_random_benchmark(c: &mut Criterion) {
    const STACK_SIZE: usize = 1;
    let heap_size: usize = env::var("HEAP_SIZE").unwrap().parse::<usize>().unwrap() * 2;
    // initializing the stack
    let mut stack = Stack::new(STACK_SIZE);
    // initializing the heap
    let mut heap = StopAndCopyHeap::init(heap_size);

    get_heap(&mut stack, &mut heap, heap_size / 2).unwrap();

    c.bench_function("collect stop and copy", |b| {
        b.iter_batched(
            || (stack.clone(), heap.clone()),
            |(mut stack, mut heap)| collect(&mut stack, &mut heap),
            criterion::BatchSize::SmallInput,
        )
    });

    c.bench_function("traverse stop and copy", |b| b.iter(|| stack.sum(&heap)));

    stack.roots[0].children.pop(); // collect(&mut stack, &mut heap);
    collect(&mut stack, &mut heap);

    get_heap_boring(&mut stack, &mut heap, heap_size / 2).unwrap();

    c.bench_function("[full clear] collect stop copy", |b| {
        b.iter_batched(
            || (stack.clone(), heap.clone()),
            |(mut stack, mut heap)| collect(&mut stack, &mut heap),
            criterion::BatchSize::SmallInput,
        )
    });

    c.bench_function("[full clear] traverse stop copy", |b| {
        b.iter(|| stack.sum(&heap))
    });
}

criterion_group!(
    benches,
    stop_and_copy_random_benchmark,
    mark_compact_random_benchmark,
);
criterion_main!(benches);

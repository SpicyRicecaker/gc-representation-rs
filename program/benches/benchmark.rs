use criterion::{criterion_group, criterion_main, Criterion};

use gc_representation_rs::shared::{MemoryManager, Stack};

use gc_representation_rs::stop_copy::StopAndCopyHeap;
use gc_representation_rs::{get_heap, get_heap_boring, mark_compact::*};

use std::env;

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

    c.bench_function("traverse mark compact", |b| b.iter(|| stack.sum_bfs(&heap)));

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
        b.iter(|| stack.sum_bfs(&heap))
    });

    c.bench_function("[full clear] traverse mark compact (depth first)", |b| {
        b.iter(|| stack.sum_dfs(&heap))
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

    c.bench_function("traverse stop and copy", |b| {
        b.iter(|| stack.sum_bfs(&heap))
    });

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
        b.iter(|| stack.sum_bfs(&heap))
    });

    c.bench_function("[full clear] traverse stop copy (depth first)", |b| {
        b.iter(|| stack.sum_dfs(&heap))
    });
}

criterion_group!(
    benches,
    stop_and_copy_random_benchmark,
    mark_compact_random_benchmark,
);
criterion_main!(benches);

use criterion::{criterion_group, criterion_main, Criterion};

use gc_representation_rs::shared::{MemoryManager, Stack};

use gc_representation_rs::stop_copy::StopAndCopyHeap;
use gc_representation_rs::{get_heap, get_heap_boring, mark_compact::*};

use std::env;

fn collect<T: MemoryManager>(stack: &mut Stack, heap: &mut T) {
    heap.collect(stack).unwrap()
}

fn random_benchmark_init(c: &mut Criterion) {
    // mark compact
    {
        const STACK_SIZE: usize = 1;
        let heap_size: usize = env::var("HEAP_SIZE").unwrap().parse::<usize>().unwrap();
        // initializing the stack
        let mut stack = Stack::new(STACK_SIZE);
        // initializing the heap
        let mut heap = MarkCompactHeap::init(heap_size);

        get_heap(&mut stack, &mut heap, heap_size).unwrap();
        random_benchmark(c, stack, heap, heap_size, "mark compact");
    }
    // stop copy
    {
        const STACK_SIZE: usize = 1;
        let heap_size: usize = env::var("HEAP_SIZE").unwrap().parse::<usize>().unwrap() * 2;
        // initializing the stack
        let mut stack = Stack::new(STACK_SIZE);
        // initializing the heap
        let mut heap = StopAndCopyHeap::init(heap_size);
        get_heap(&mut stack, &mut heap, heap_size / 2).unwrap();
        random_benchmark(c, stack, heap, heap_size / 2, "stop copy");
    }
}

fn random_benchmark<T: MemoryManager + Clone>(
    c: &mut Criterion,
    mut stack: Stack,
    mut heap: T,
    heap_size: usize,
    label: &str,
) {
    c.bench_function(&format!("{} collection", label), |b| {
        b.iter_batched(
            || (stack.clone(), heap.clone()),
            |(mut stack, mut heap)| collect(&mut stack, &mut heap),
            criterion::BatchSize::SmallInput,
        )
    });
    c.bench_function(&format!("{} traverse (breadth first)", label), |b| {
        b.iter(|| stack.sum_bfs(&heap))
    });
    c.bench_function(&format!("{} traverse (breadth first)", label), |b| {
        b.iter(|| stack.sum_dfs(&heap))
    });

    // reset the heap into a boring heap
    stack.roots[0].children.pop();
    collect(&mut stack, &mut heap);
    get_heap_boring(&mut stack, &mut heap, heap_size).unwrap();

    c.bench_function(&format!("{} full collection", label), |b| {
        b.iter_batched(
            || (stack.clone(), heap.clone()),
            |(mut stack, mut heap)| collect(&mut stack, &mut heap),
            criterion::BatchSize::SmallInput,
        )
    });

    c.bench_function(&format!("{} traverse (depth first)", label), |b| {
        b.iter(|| stack.sum_bfs(&heap))
    });

    c.bench_function(&format!("{} traverse (breadth first)", label), |b| {
        b.iter(|| stack.sum_dfs(&heap))
    });
}

criterion_group!(benches, random_benchmark_init,);
criterion_main!(benches);

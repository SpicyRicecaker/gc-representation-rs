use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

use gc_representation_rs::shared::{MemoryManager, Stack};

use gc_representation_rs::stop_copy::StopAndCopyHeap;
use gc_representation_rs::{link_heap, make_garbage, mark_compact::*};

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

        // get_heap(&mut stack, &mut heap, heap_size).unwrap();
        link_heap(&mut stack, &mut heap, heap_size).unwrap();
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
        // get_heap(&mut stack, &mut heap, heap_size / 2).unwrap();
        link_heap(&mut stack, &mut heap, heap_size / 2).unwrap();
        random_benchmark(c, stack, heap, heap_size / 2, "stop copy");
    }
}

fn random_benchmark<T: MemoryManager + Clone>(
    c: &mut Criterion,
    stack: Stack,
    heap: T,
    heap_size: usize,
    label: &str,
) {
    let input_data: Vec<(f32, f32)> = [
        0., 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0, 1.1, 1.2, 1.3, 1.4, 1.5,
    ]
    .iter()
    .map(|size| {
        let mut stack = stack.clone();
        let mut heap = heap.clone();

        // dead to live ratio
        make_garbage(&mut stack, &mut heap, heap_size, *size).unwrap();
        collect(&mut stack, &mut heap);

        (
            *size,
            ((1. - stack.count(&heap).unwrap().0 as f32 / heap_size as f32) * 1000.).round()
                / 1000.,
        )
    })
    .collect();

    let mut group = c.benchmark_group(&format!("{} collection", label));

    for (size, ratio) in input_data.iter() {
        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(ratio), size, |b, size| {
            b.iter_batched(
                || {
                    let mut stack = stack.clone();
                    let mut heap = heap.clone();

                    make_garbage(&mut stack, &mut heap, heap_size, *size).unwrap();

                    (stack.clone(), heap.clone())
                },
                |(mut stack, mut heap)| collect(&mut stack, &mut heap),
                criterion::BatchSize::SmallInput,
            )
        });
    }
    group.finish();

    let mut group = c.benchmark_group(&format!("{} traverse (breadth first)", label));

    for (size, ratio) in input_data.iter() {
        {
            let mut stack = stack.clone();
            let mut heap = heap.clone();
            make_garbage(&mut stack, &mut heap, heap_size, *size).unwrap();

            group.bench_function(BenchmarkId::from_parameter(ratio), |b| {
                b.iter(|| stack.sum_bfs(&heap))
            });
        }
    }
    group.finish();

    let mut group = c.benchmark_group(&format!("{} traverse (depth first)", label));

    for (size, ratio) in input_data.iter() {
        {
            let mut stack = stack.clone();
            let mut heap = heap.clone();
            make_garbage(&mut stack, &mut heap, heap_size, *size).unwrap();

            group.bench_function(BenchmarkId::from_parameter(ratio), |b| {
                b.iter(|| stack.sum_dfs(&heap))
            });
        }
    }
    group.finish();

    // c.bench_function(&format!("{} collection", label), |b| {
    //     b.iter_batched(
    //         || (stack.clone(), heap.clone()),
    //         |(mut stack, mut heap)| collect(&mut stack, &mut heap),
    //         criterion::BatchSize::SmallInput,
    //     )
    // });
    // c.bench_function(&format!("{} traverse (breadth first)", label), |b| {
    //     b.iter(|| stack.sum_bfs(&heap))
    // });
    // c.bench_function(&format!("{} traverse (breadth first)", label), |b| {
    //     b.iter(|| stack.sum_dfs(&heap))
    // });

    // // reset the heap into a boring heap
    // stack.roots[0].children.pop();
    // collect(&mut stack, &mut heap);
    // get_heap_boring(&mut stack, &mut heap, heap_size).unwrap();

    // c.bench_function(&format!("{} full collection", label), |b| {
    //     b.iter_batched(
    //         || (stack.clone(), heap.clone()),
    //         |(mut stack, mut heap)| collect(&mut stack, &mut heap),
    //         criterion::BatchSize::SmallInput,
    //     )
    // });

    // c.bench_function(&format!("{} traverse (depth first)", label), |b| {
    //     b.iter(|| stack.sum_bfs(&heap))
    // });

    // c.bench_function(&format!("{} traverse (breadth first)", label), |b| {
    //     b.iter(|| stack.sum_dfs(&heap))
    // });
}

criterion_group!(benches, random_benchmark_init);
criterion_main!(benches);

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

use gc_representation_rs::shared::{MemoryManager, Stack};

use gc_representation_rs::stop_copy::StopAndCopyHeap;
use gc_representation_rs::{link_heap, make_garbage, mark_compact::*};

use rand::prelude::*;
use rand_pcg::Pcg64;

use std::env;

fn collect<T: MemoryManager>(stack: &mut Stack, heap: &mut T) {
    heap.collect(stack).unwrap()
}

// enum MemoryType {
//     MarkCompact(Memory<MarkCompactHeap>),
//     StopAndCopy(Memory<StopAndCopyHeap>),
// }
// struct Memory<T: MemoryManager> {
//     label: Option<String>,
//     stack: Stack,
//     heap: T,
// }

fn random_benchmark_init(c: &mut Criterion) {
    // holds the two different types of heaps we have

    // initialize this rng which we will subsequently clone mutable references
    // of in the later benchmarks
    let mut rng = Pcg64::seed_from_u64(1234);
    // {
    // create marksweep thingy
    const STACK_SIZE: usize = 1;
    let heap_size: usize = env::var("HEAP_SIZE").unwrap().parse::<usize>().unwrap();
    // initializing the stack
    let mut m_stack = Stack::new(STACK_SIZE);
    // initializing the heap
    let mut m_heap = MarkCompactHeap::init(heap_size);

    // initialize the thing with a new seed

    // link the heap (actually changes rng)
    link_heap(&mut m_stack, &mut m_heap, &mut rng).unwrap();

    // make the memory type now own the heap and the stack of this thing
    // question: does this just update a pointer ref, or is memory actually
    // moved here?
    //
    // seems like structs exist on stack by default but if you have a vector
    // in a struct, then that field points to something allocated on the
    // heap
    //
    // in this case, moving the heap would move the vectors that we actually
    // care about, but the intergers are gonna be copied by value
    // memory_types.push(MemoryType::MarkCompact(Memory {
    //     label: Some(String::from("Mark Compact")),
    //     stack,
    //     heap,
    // }))
    // }
    // stop copy
    // {
    // const STACK_SIZE: usize = 1;
    // let heap_size: usize = env::var("HEAP_SIZE").unwrap().parse::<usize>().unwrap() * 2;
    // initializing the stack
    let mut s_stack = Stack::new(STACK_SIZE);
    // initializing the heap
    let mut s_heap = StopAndCopyHeap::init(heap_size * 2);
    // get_heap(&mut stack, &mut heap, heap_size / 2).unwrap();

    let mut rng = Pcg64::seed_from_u64(1234);
    link_heap(&mut s_stack, &mut s_heap, &mut rng).unwrap();
    // memory_types.push(MemoryType::StopAndCopy(Memory {
    //     label: Some(String::from("Stop and Copy")),
    //     stack,
    //     heap,
    // }))
    // }
    random_benchmark(c, m_stack, m_heap, s_stack, s_heap, rng);
}

fn random_benchmark(
    c: &mut Criterion,
    m_stack: Stack,
    m_heap: MarkCompactHeap,
    s_stack: Stack,
    s_heap: StopAndCopyHeap,
    rng: Pcg64,
) {
    let input_data: Vec<(f32, f32)> = [
        0., 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0, 1.1, 1.2, 1.3, 1.4, 1.5,
    ]
    .iter()
    .rev()
    .map(|size| {
        // pick any algorithm from mark compact
        let mut stack = m_stack.clone();
        let mut heap = m_heap.clone();

        // dead to live ratio
        make_garbage(&mut stack, &mut heap, *size, &mut rng.clone()).unwrap();
        collect(&mut stack, &mut heap);

        (
            *size,
            ((1. - stack.count(&heap).unwrap().0 as f32 / heap.heap_size() as f32) * 1000.).round()
                / 1000.,
        )
    })
    .collect();

    dbg!(&input_data);

    let mut group = c.benchmark_group("Collection Performance");

    for (size, ratio) in input_data.iter() {
        group.bench_with_input(BenchmarkId::new("Mark Sweep", ratio), ratio, |b, _ratio| {
            b.iter_batched(
                || {
                    let mut stack = m_stack.clone();
                    let mut heap = m_heap.clone();

                    make_garbage(&mut stack, &mut heap, *size, &mut rng.clone()).unwrap();

                    (stack, heap)
                },
                |(mut stack, mut heap)| collect(&mut stack, &mut heap),
                criterion::BatchSize::SmallInput,
            )
        });
        group.bench_with_input(
            BenchmarkId::new("Stop and Copy", ratio),
            ratio,
            |b, _ratio| {
                b.iter_batched(
                    || {
                        let mut stack = m_stack.clone();
                        let mut heap = m_heap.clone();

                        make_garbage(&mut stack, &mut heap, *size, &mut rng.clone()).unwrap();

                        (stack, heap)
                    },
                    |(mut stack, mut heap)| collect(&mut stack, &mut heap),
                    criterion::BatchSize::SmallInput,
                )
            },
        );
    }
    group.finish();

    let mut group = c.benchmark_group("Runtime Performance: Breadth-First Search");

    for (size, ratio) in input_data.iter() {
        {
            // clone the stack and heap (necessary)
            let mut stack = m_stack.clone();
            let mut heap = m_heap.clone();

            // create our own garbage from the heap that should already be linked
            make_garbage(&mut stack, &mut heap, *size, &mut rng.clone()).unwrap();
            collect(&mut stack, &mut heap);
            // println!("{}", heap.free);

            group.bench_with_input(
                BenchmarkId::new("Mark Compact", ratio),
                ratio,
                |b, _ratio| b.iter(|| stack.sum_bfs(&heap)),
            );
        }
        {
            let mut stack = s_stack.clone();
            let mut heap = s_heap.clone();

            make_garbage(&mut stack, &mut heap, *size, &mut rng.clone()).unwrap();
            collect(&mut stack, &mut heap);

            group.bench_with_input(
                BenchmarkId::new("Stop and Copy", ratio),
                ratio,
                |b, _ratio| b.iter(|| stack.sum_bfs(&heap)),
            );
        }
    }
    group.finish();

    let mut group = c.benchmark_group("Runtime Performance: Depth-First Search");

    for (size, ratio) in input_data.iter() {
        {
            // clone the stack and heap (necessary)
            let mut stack = s_stack.clone();
            let mut heap = s_heap.clone();

            // create our own garbage from the heap that should already be linked
            make_garbage(&mut stack, &mut heap, *size, &mut rng.clone()).unwrap();
            collect(&mut stack, &mut heap);

            group.bench_with_input(
                BenchmarkId::new("Mark Compact", ratio),
                ratio,
                |b, _ratio| b.iter(|| stack.sum_dfs(&heap)),
            );
        }
        {
            let mut stack = s_stack.clone();
            let mut heap = s_heap.clone();

            make_garbage(&mut stack, &mut heap, *size, &mut rng.clone()).unwrap();
            collect(&mut stack, &mut heap);

            group.bench_with_input(
                BenchmarkId::new("Stop and Copy", ratio),
                ratio,
                |b, _ratio| b.iter(|| stack.sum_dfs(&heap)),
            );
        }
    }
    group.finish();
}

criterion_group!(benches, random_benchmark_init);
criterion_main!(benches);

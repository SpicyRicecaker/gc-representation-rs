## Results

![Graph of Collection Perf](https://raw.githubusercontent.com/SpicyRicecaker/gc-representation-rs/master/program/res/collection.svg)

![Graph of BFS Perf](https://raw.githubusercontent.com/SpicyRicecaker/gc-representation-rs/master/program/res/bfs.svg)

![Graph of DFS Perf](https://raw.githubusercontent.com/SpicyRicecaker/gc-representation-rs/master/program/res/dfs.svg)

## Running

Install `cargo-criterion`.

```shell
cargo install cargo-criterion
```

(Optional) install `gnuplot` for graphs.

Clone and run benchmark

```shell
git clone https://github.com/SpicyRicecaker/gc-representation-rs
cd program
# Set heap_size to 1000000
HEAP_SIZE=1000000 cargo criterion
```

# Garbage Collection Representation in Rust

This program demonstrates two examples of garbage collection algorithms in Rust: sliding mark-compact and cheney style stop-and-copy (aka mark-copy).

The abstraction is that instead of messing with raw memory, we use pre-allocated arrays to represent memory, and we put structs on that struct that each have references to parent and children. 

We make a graph-like data structure and do hundreds / thousands of add remove operations, record cpu and memory usage, and of course, the time that it takes to complete the program.

## Running

Clone the git repo first

```shell
git clone https://github.com/SpicyRicecaker/gc-representation-rs.git --depth=1
```

### Linux

On linux, this project uses the [mold](https://github.com/rui314/mold) linker to compile its code. Did you know mold uses a **concurrent mark-sweep** garbage collection algorithm to remove files that aren't referenced?

Install the [dependencies](https://github.com/rui314/mold#install-dependencies) for your distro

Then clone and compile

```shell
git clone https://github.com/rui314/mold.git --depth=1
cd mold
git checkout v1.0.1
make -j$(nproc)
sudo make install
```

### Windows

Now just Just cargo run lol

```shell
cargo run
```
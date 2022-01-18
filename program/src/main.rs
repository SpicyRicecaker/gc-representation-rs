use rand::prelude::*;
use rand_pcg::Pcg64;

fn main() {
    // let mut rng = thread_rng();
    let mut rng = Pcg64::seed_from_u64(1234);
    println!("{}", rng.gen_range(0..100_000_000));
    println!("{}", rng.gen_range(0..100_000_000));
    println!("{}", rng.gen_range(0..100_000_000));
    println!("---------------------");
    let mut rng = Pcg64::seed_from_u64(1234);
    println!("{}", rng.gen_range(0..100_000_000));
    println!("{}", rng.gen_range(0..100_000_000));
    println!("{}", rng.gen_range(0..100_000_000));
}

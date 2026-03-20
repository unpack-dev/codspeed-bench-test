use std::env;

use bench_demo::repeat_string;

fn main() {
    let fragment = env::args().nth(1).unwrap_or_else(|| "a".to_string());
    let result = repeat_string(&fragment);
    println!("{result}");
}

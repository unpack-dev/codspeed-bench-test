use std::env;

const REPEAT_TIMES: usize = 10_000;

fn main() {
    let fragment = env::args().nth(1).unwrap_or_else(|| "a".to_string());
    let mut result = String::with_capacity(fragment.len() * REPEAT_TIMES);

    for _ in 0..REPEAT_TIMES {
        result.push_str(&fragment);
    }

    println!("{result}");
}

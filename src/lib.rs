const REPEAT_TIMES: usize = 10_000;

pub fn repeat_string(fragment: &str) -> String {
    let mut result = String::with_capacity(fragment.len() * REPEAT_TIMES);

    for _ in 0..REPEAT_TIMES {
        result.push_str(fragment);
    }

    result
}

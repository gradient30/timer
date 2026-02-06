use std::env;

fn main() {
    let count = env::args()
        .nth(1)
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(1);

    for _ in 0..count {
        println!("{}", timer_lib::activation::generate_activation_code());
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    rpn(&args[1]);
}

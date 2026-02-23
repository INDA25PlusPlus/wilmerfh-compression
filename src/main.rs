struct HuffmanTree {
    left: u8,
    right: Box<HuffmanTree>,
}

fn read_input(path: &str) -> Vec<u8> {
    std::fs::read(path).expect("failed to read file")
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = &args[1];
    let bytes = read_input(path);
    println!("{}", String::from_utf8_lossy(&bytes));
}

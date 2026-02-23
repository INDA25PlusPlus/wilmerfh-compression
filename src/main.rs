enum Node {
    Leaf(u8),
    Tree(Box<HuffmanTree>),
}

struct HuffmanTree {
    left: Node,
    right: Node,
}

impl HuffmanTree {
    fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        self.collect_leaves(&mut bytes);
        let mut result = vec![bytes.len() as u8];
        result.extend(bytes);
        result
    }

    fn collect_leaves(&self, out: &mut Vec<u8>) {
        match &self.left {
            Node::Leaf(b) => out.push(*b),
            Node::Tree(t) => t.collect_leaves(out),
        }
        match &self.right {
            Node::Leaf(b) => out.push(*b),
            Node::Tree(t) => t.collect_leaves(out),
        }
    }

    fn from_sorted(bytes: &[u8]) -> HuffmanTree {
        if bytes.len() == 2 {
            HuffmanTree {
                left: Node::Leaf(bytes[0]),
                right: Node::Leaf(bytes[1]),
            }
        } else {
            HuffmanTree {
                left: Node::Leaf(bytes[0]),
                right: Node::Tree(Box::new(HuffmanTree::from_sorted(&bytes[1..]))),
            }
        }
    }
}

fn read_input(path: &str) -> Vec<u8> {
    std::fs::read(path).expect("failed to read file")
}

fn count_frequencies(bytes: &[u8]) -> Vec<u8> {
    let mut freq = std::collections::HashMap::new();
    for &b in bytes {
        *freq.entry(b).or_insert(0) += 1;
    }
    let mut sorted: Vec<u8> = freq.keys().copied().collect();
    sorted.sort_by(|a, b| freq[b].cmp(&freq[a]));
    sorted
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = &args[1];
    let bytes = read_input(path);
    let sorted = count_frequencies(&bytes);
    println!("{:?}", sorted);
}

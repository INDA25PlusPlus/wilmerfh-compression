struct Encoded {
    tree: Vec<u8>,
    bytes: Vec<u8>,
    padding: u8,
}

impl Encoded {
    fn from_bits(bits: &[bool], tree: Vec<u8>) -> Encoded {
        let padding = if bits.len() % 8 == 0 {
            0
        } else {
            8 - (bits.len() % 8) as u8
        };
        let mut bytes = Vec::new();
        for chunk in bits.chunks(8) {
            let mut byte = 0u8;
            for (i, &bit) in chunk.iter().enumerate() {
                if bit {
                    byte |= 1 << (7 - i);
                }
            }
            bytes.push(byte);
        }
        Encoded {
            tree,
            bytes,
            padding,
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut out = vec![self.padding];
        out.extend(&self.tree);
        out.extend(&self.bytes);
        out
    }

    fn from_bytes(data: &[u8]) -> Encoded {
        let padding = data[0];
        let tree_len = data[1] as usize;
        let tree_end = 2 + tree_len;
        let tree = data[1..tree_end].to_vec();
        let bytes = data[tree_end..].to_vec();
        Encoded {
            tree,
            bytes,
            padding,
        }
    }
}

enum Node {
    Leaf(u8),
    Tree(Box<HuffmanTree>),
}

struct HuffmanTree {
    left: u8,
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
        out.push(self.left);
        match &self.right {
            Node::Leaf(b) => out.push(*b),
            Node::Tree(t) => t.collect_leaves(out),
        }
    }

    fn build_map(&self) -> std::collections::HashMap<u8, Vec<bool>> {
        let mut map = std::collections::HashMap::new();
        let mut code = Vec::new();
        let mut current = self;

        loop {
            code.push(false);
            map.insert(current.left, code.clone());

            code.pop();
            code.push(true);
            match &current.right {
                Node::Leaf(b) => {
                    map.insert(*b, code.clone());
                    break;
                }
                Node::Tree(t) => current = t,
            }
        }
        map
    }

    fn encode(&self, data: &[u8]) -> Encoded {
        let map = self.build_map();
        let mut bits: Vec<bool> = Vec::new();
        for &b in data {
            let code = map.get(&b).expect("byte not in tree");
            bits.extend(code);
        }
        Encoded::from_bits(&bits, self.serialize())
    }

    fn from_sorted(bytes: &[u8]) -> HuffmanTree {
        if bytes.len() == 2 {
            HuffmanTree {
                left: bytes[0],
                right: Node::Leaf(bytes[1]),
            }
        } else {
            HuffmanTree {
                left: bytes[0],
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_byte_tree_assigns_one_bit_each() {
        let tree = HuffmanTree::from_sorted(&[b'a', b'b']);
        let map = tree.build_map();
        assert_eq!(map[&b'a'], vec![false]);
        assert_eq!(map[&b'b'], vec![true]);
    }

    #[test]
    fn encode_produces_smaller_output_for_repetitive_data() {
        let data = b"aaaaaaaaaaaaaaaaaaaab";
        let sorted = count_frequencies(data);
        let tree = HuffmanTree::from_sorted(&sorted);
        let encoded = tree.encode(data);
        assert!(encoded.bytes.len() < data.len());
    }

    #[test]
    fn encoded_round_trips_through_bytes() {
        let tree = HuffmanTree::from_sorted(&[b'x', b'y', b'z']);
        let encoded = tree.encode(b"xxyzzy");
        let raw = encoded.to_bytes();
        let restored = Encoded::from_bytes(&raw);
        assert_eq!(encoded.tree, restored.tree);
        assert_eq!(encoded.bytes, restored.bytes);
        assert_eq!(encoded.padding, restored.padding);
    }

    #[test]
    fn serialize_starts_with_leaf_count() {
        let tree = HuffmanTree::from_sorted(&[1, 2, 3]);
        let serialized = tree.serialize();
        assert_eq!(serialized[0], 3);
        assert_eq!(&serialized[1..], &[1, 2, 3]);
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = &args[1];
    let bytes = read_input(path);
    let sorted = count_frequencies(&bytes);
    let tree = HuffmanTree::from_sorted(&sorted);
    let encoded = tree.encode(&bytes);
    let total = encoded.tree.len() + encoded.bytes.len();
    println!("original: {} bytes", bytes.len());
    println!(
        "encoded:  {} bytes (tree: {}, data: {}, {} padding bits)",
        total,
        encoded.tree.len(),
        encoded.bytes.len(),
        encoded.padding
    );
}

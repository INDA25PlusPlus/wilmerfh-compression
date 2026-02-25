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

    fn decode(&self) -> Vec<u8> {
        let tree = HuffmanTree::from_sorted(&self.tree[1..]);
        let total_bits = self.bytes.len() * 8 - self.padding as usize;
        let mut out = Vec::new();
        let mut current = &tree;
        for i in 0..total_bits {
            let byte_idx = i / 8;
            let bit_idx = 7 - (i % 8);
            let bit = (self.bytes[byte_idx] >> bit_idx) & 1 == 1;
            if !bit {
                out.push(current.left);
                current = &tree;
            } else {
                match &current.right {
                    Node::Leaf(b) => {
                        out.push(*b);
                        current = &tree;
                    }
                    Node::Tree(t) => current = t,
                }
            }
        }
        out
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

fn encode(data: &[u8]) -> Vec<u8> {
    let sorted = count_frequencies(data);
    let tree = HuffmanTree::from_sorted(&sorted);
    tree.encode(data).to_bytes()
}

fn decode(data: &[u8]) -> Vec<u8> {
    Encoded::from_bytes(data).decode()
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
    let data = read_input(path);
    let compressed = encode(&data);
    println!("original:   {} bytes", data.len());
    println!("compressed: {} bytes", compressed.len());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_repetitive_data() {
        let data = b"aaaaaaaaaaaaaaaaaaaab";
        assert_eq!(decode(&encode(data)), data);
    }

    #[test]
    fn round_trip_varied_data() {
        let data = b"abcdefghijabcdefghij";
        assert_eq!(decode(&encode(data)), data);
    }

    #[test]
    fn repetitive_data_is_smaller_overall() {
        let data = b"aaaaaaaaaaaaaaaaaaaab";
        assert!(encode(data).len() < data.len());
    }

    #[test]
    fn non_repetitive_data_has_smaller_data_part() {
        let data = b"abcdefghijabcdefghij";
        let encoded = Encoded::from_bytes(&encode(data));
        assert!(encoded.bytes.len() < data.len());
    }
}

use std::cmp::Ordering;

#[derive(Debug)]
struct BadOrd(i32);

impl PartialEq for BadOrd {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for BadOrd {}

impl PartialOrd for BadOrd {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for BadOrd {
    fn cmp(&self, other: &Self) -> Ordering {
        // Intentionally incorrect implementation that violates transitivity
        if self.0 <= other.0 {
            Ordering::Greater
        } else {
            Ordering::Less
        }
    }
}

fn main() {
    let mut vec = vec![BadOrd(3), BadOrd(2), BadOrd(4), BadOrd(1)];

    println!("Before sorting: {:?}", vec);

    // This should panic in Rust 1.81.0
    vec.sort();

    println!("After sorting: {:?}", vec);
}
// Output:
// rustup run 1.81.0 cargo run -v
// Before sorting: [BadOrd(3), BadOrd(2), BadOrd(4), BadOrd(1)]
// After sorting: [BadOrd(4), BadOrd(3), BadOrd(2), BadOrd(1)]

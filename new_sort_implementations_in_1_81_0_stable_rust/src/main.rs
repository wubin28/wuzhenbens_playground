use std::cmp::{Ord, Ordering, PartialOrd};

#[derive(Debug, Eq, PartialEq)]
struct BadlyOrdered(i32);

impl PartialOrd for BadlyOrdered {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for BadlyOrdered {
    fn cmp(&self, other: &Self) -> Ordering {
        // Violating transitivity:
        // We want 1 < 2, 2 < 3, but 3 < 1
        match (self.0, other.0) {
            (1, 2) | (2, 3) => Ordering::Less,
            (3, 1) => Ordering::Less,
            (a, b) if a == b => Ordering::Equal,
            _ => Ordering::Greater,
        }
    }
}

fn main() {
    let a = BadlyOrdered(1);
    let b = BadlyOrdered(2);
    let c = BadlyOrdered(3);

    println!("Comparing individual pairs:");
    println!("a < b: {:?}", a < b); // true
    println!("b < c: {:?}", b < c); // true
    println!("c < a: {:?}", c < a); // true

    println!("\nThis violates transitivity!");

    let mut vec = vec![c, b, a];
    println!("\nOriginal vector: {:?}", vec);

    println!("Attempting to sort...");
    vec.sort();
    println!("Sorted vector: {:?}", vec);

    println!("\nChecking if the vector is actually sorted:");
    println!("Is sorted: {:?}", vec.windows(2).all(|w| w[0] <= w[1]));
}
// Output:
// rustup run 1.81.0 cargo run -v
// Comparing individual pairs:
// a < b: true
// b < c: true
// c < a: true

// This violates transitivity!

// Original vector: [BadlyOrdered(3), BadlyOrdered(2), BadlyOrdered(1)]
// Attempting to sort...
// Sorted vector: [BadlyOrdered(2), BadlyOrdered(3), BadlyOrdered(1)]

// Checking if the vector is actually sorted:
// Is sorted: true
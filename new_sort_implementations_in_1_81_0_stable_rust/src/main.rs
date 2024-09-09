use std::cmp::Ordering;

#[derive(Debug)]
struct BadOrd(i32);

impl PartialEq for BadOrd {
    fn eq(&self, other: &Self) -> bool {
        // Intentionally inconsistent equality
        self.0 % 2 == other.0 % 2
    }
}

impl Eq for BadOrd {}

impl PartialOrd for BadOrd {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // Violates consistency, transitivity, and duality
        if self.0 % 2 == 0 && other.0 % 2 != 0 {
            Some(Ordering::Less)
        } else if self.0 % 2 != 0 && other.0 % 2 == 0 {
            Some(Ordering::Greater)
        } else if self.0 == other.0 {
            Some(Ordering::Equal)
        } else {
            None
        }
    }
}

impl Ord for BadOrd {
    fn cmp(&self, other: &Self) -> Ordering {
        // Inconsistent with PartialOrd and violates total ordering
        if self.0 < other.0 {
            Ordering::Greater
        } else if self.0 > other.0 {
            Ordering::Less
        } else {
            Ordering::Equal
        }
    }
}

fn main() {
    let mut vec = vec![BadOrd(3), BadOrd(2), BadOrd(4), BadOrd(1)];

    println!("Before sorting: {:?}", vec);

    vec.sort(); // This will likely panic due to inconsistent ordering

    println!("After sorting: {:?}", vec);

    // These assertions will fail, demonstrating incorrect ordering
    assert!(BadOrd(1) < BadOrd(2));
    assert!(BadOrd(2) > BadOrd(1));
    assert!(BadOrd(2) == BadOrd(2));

    println!("All assertions passed!");
}
// Output:
// rustup run 1.81.0 cargo run
// Before sorting: [BadOrd(3), BadOrd(2), BadOrd(4), BadOrd(1)]
// After sorting: [BadOrd(2), BadOrd(4), BadOrd(3), BadOrd(1)]
// thread 'main' panicked at src/main.rs:53:5:
// assertion failed: BadOrd(1) < BadOrd(2)
// note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
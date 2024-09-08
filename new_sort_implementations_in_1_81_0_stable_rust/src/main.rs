#[derive(Debug)]
struct BadOrd(i32);

impl PartialEq for BadOrd {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl PartialOrd for BadOrd {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.0.cmp(&other.0))
    }
}

impl Eq for BadOrd {}

impl Ord for BadOrd {
    // Incorrect implementation: Always returns Greater, violating all sorting rules
    fn cmp(&self, _: &Self) -> std::cmp::Ordering {
        std::cmp::Ordering::Greater
    }
}

fn main() {
    let mut vec = vec![BadOrd(3), BadOrd(2), BadOrd(4), BadOrd(1)];
    
    println!("Before sorting: {:?}", vec);
    
    // Sorting with a bad Ord implementation
    vec.sort(); // This should panic in Rust 1.81.0

    println!("After sorting: {:?}", vec);
}
// Output:
// rustup run 1.81.0 cargo run -v
// Before sorting: [BadOrd(3), BadOrd(2), BadOrd(4), BadOrd(1)]
// After sorting: [BadOrd(1), BadOrd(2), BadOrd(3), BadOrd(4)]
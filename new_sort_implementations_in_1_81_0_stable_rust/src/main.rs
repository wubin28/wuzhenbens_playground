use std::cmp::Ordering;

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
struct GoodOrd(i32);

fn main() {
    let mut vec = vec![GoodOrd(3), GoodOrd(2), GoodOrd(4), GoodOrd(1)];

    println!("Before sorting: {:?}", vec);

    vec.sort();

    println!("After sorting: {:?}", vec);

    // Demonstrating correct ordering
    assert!(GoodOrd(1) < GoodOrd(2));
    assert!(GoodOrd(2) > GoodOrd(1));
    assert!(GoodOrd(2) == GoodOrd(2));

    println!("All assertions passed!");
}
// Output:
// Before sorting: [GoodOrd(3), GoodOrd(2), GoodOrd(4), GoodOrd(1)]
// After sorting: [GoodOrd(1), GoodOrd(2), GoodOrd(3), GoodOrd(4)]
// All assertions passed!
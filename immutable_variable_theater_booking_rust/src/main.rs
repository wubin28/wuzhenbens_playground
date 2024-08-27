use std::sync::{Arc, Mutex};
use std::thread;

struct Theater {
    available_tickets: Mutex<i32>,
}

impl Theater {
    fn new(initial_tickets: i32) -> Self {
        Theater {
            available_tickets: Mutex::new(initial_tickets),
        }
    }

    fn book_ticket(&self) {
        let mut tickets = self.available_tickets.lock().unwrap();
        if *tickets > 0 {
            // 模拟一些处理时间
            thread::sleep(std::time::Duration::from_millis(10));
            *tickets -= 1;
            println!("Ticket booked. Remaining tickets: {}", *tickets);
        } else {
            println!("Sorry, no more tickets available.");
        }
    }

    fn get_available_tickets(&self) -> i32 {
        *self.available_tickets.lock().unwrap()
    }
}

fn main() {
    let theater = Arc::new(Theater::new(10)); // 初始有10张票

    let mut handles = vec![];
    for _ in 0..15 {
        let theater_clone = Arc::clone(&theater);
        let handle = thread::spawn(move || {
            theater_clone.book_ticket();
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Final ticket count: {}", theater.get_available_tickets());
}
// Output:
// Ticket booked. Remaining tickets: 9
// Ticket booked. Remaining tickets: 8
// Ticket booked. Remaining tickets: 7
// Ticket booked. Remaining tickets: 6
// Ticket booked. Remaining tickets: 5
// Ticket booked. Remaining tickets: 4
// Ticket booked. Remaining tickets: 3
// Ticket booked. Remaining tickets: 2
// Ticket booked. Remaining tickets: 1
// Ticket booked. Remaining tickets: 0
// Sorry, no more tickets available.
// Sorry, no more tickets available.
// Sorry, no more tickets available.
// Sorry, no more tickets available.
// Sorry, no more tickets available.
// Final ticket count: 0

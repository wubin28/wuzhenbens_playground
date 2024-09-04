use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone)]
struct Product {
    id: u32,
    name: String,
    price: f64,
}

impl Drop for Product {
    fn drop(&mut self) {
        println!("Dropping product: {} (ID: {})", self.name, self.id);
    }
}

#[derive(Debug)]
struct Inventory {
    // <key, value>: <product_id, (product, quantity)>
    products: HashMap<u32, (Product, u32)>,
}

#[derive(Debug)]
struct Order {
    id: u32,
    // <(product_id, quantity)>
    products: Vec<(u32, u32)>,
}

impl fmt::Display for Order {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Order ID: {}", self.id)?;
        writeln!(f, "Products:")?;
        for (product_id, quantity) in &self.products {
            writeln!(f, "  Product ID: {}, Quantity: {}", product_id, quantity)?;
        }
        Ok(())
    }
}

impl Inventory {
    fn new() -> Self {
        Inventory {
            products: HashMap::new(),
        }
    }

    fn add_product(&mut self, product: Product, quantity: u32) {
        println!(
            "Adding product to inventory: {} (ID: {}, Quantity: {})",
            product.name, product.id, quantity
        );
        self.products.insert(product.id, (product, quantity));
    }

    fn update_quantity(&mut self, product_id: u32, quantity: u32) -> Result<(), String> {
        if let Some((_, stock)) = self.products.get_mut(&product_id) {
            *stock = quantity;
            Ok(())
        } else {
            Err(format!("Product with id {} not found", product_id))
        }
    }

    fn get_product(&self, product_id: u32) -> Option<&Product> {
        self.products.get(&product_id).map(|(product, _)| product)
    }

    fn get_quantity(&self, product_id: u32) -> Option<u32> {
        self.products
            .get(&product_id)
            .map(|(_, quantity)| *quantity)
    }
}

struct OrderProcessor {
    inventory: Inventory,
    orders: Vec<Order>,
}

impl OrderProcessor {
    fn new(inventory: Inventory) -> Self {
        OrderProcessor {
            inventory,
            orders: Vec::new(),
        }
    }

    fn process_order(&mut self, order: Order) -> Result<(), String> {
        println!("\nProcessing order:\n{}", order);

        for (product_id, quantity) in &order.products {
            let current_quantity = self
                .inventory
                .get_quantity(*product_id)
                .ok_or_else(|| format!("Product with id {} not found", product_id))?;

            if current_quantity < *quantity {
                return Err(format!("Insufficient stock for product {}", product_id));
            }
        }

        for (product_id, quantity) in &order.products {
            let current_quantity = self.inventory.get_quantity(*product_id).unwrap();
            self.inventory
                .update_quantity(*product_id, current_quantity - quantity)?;
        }

        self.orders.push(order);
        Ok(())
    }
}

fn main() {
    let mut inventory = Inventory::new();

    // 添加产品到库存
    inventory.add_product(
        Product {
            id: 1,
            name: "Laptop".to_string(),
            price: 999.99,
        },
        10,
    );
    inventory.add_product(
        Product {
            id: 2,
            name: "Smartphone".to_string(),
            price: 499.99,
        },
        20,
    );
    inventory.add_product(
        Product {
            id: 3,
            name: "Tablet".to_string(),
            price: 299.99,
        },
        15,
    );

    println!("\nInitial inventory:");
    for (_id, (product, quantity)) in &inventory.products {
        println!("Product: {:?}, Quantity: {}", product, quantity);
    }

    // 演示复制（Clone）
    println!("\nDemonstrating Clone:");
    if let Some(product) = inventory.get_product(1) {
        let cloned_product = product.clone();
        println!("Original product: {:?}", product);
        println!("Cloned product: {:?}", cloned_product);
    }

    let mut order_processor = OrderProcessor::new(inventory);

    // 处理订单
    let order1 = Order {
        id: 1,
        products: vec![(1, 2), (2, 3)],
    };

    match order_processor.process_order(order1) {
        Ok(()) => println!("Order 1 processed successfully"),
        Err(e) => println!("Failed to process order 1: {}", e),
    }

    println!("\nInventory after processing order 1:");
    for (_id, (product, quantity)) in &order_processor.inventory.products {
        println!("Product: {:?}, Quantity: {}", product, quantity);
    }

    // 尝试处理一个库存不足的订单
    let order2 = Order {
        id: 2,
        products: vec![(3, 20)],
    };

    match order_processor.process_order(order2) {
        Ok(()) => println!("Order 2 processed successfully"),
        Err(e) => println!("Failed to process order 2: {}", e),
    }

    println!("\nFinal inventory:");
    for (_id, (product, quantity)) in &order_processor.inventory.products {
        println!("Product: {:?}, Quantity: {}", product, quantity);
    }

    // 演示丢弃（Drop）
    println!("\nDemonstrating Drop:");
    {
        let product_to_drop = Product {
            id: 4,
            name: "Headphones".to_string(),
            price: 99.99,
        };
        println!("Created product: {:?}", product_to_drop);
    } // product_to_drop 在这里离开作用域，将被丢弃

    println!("\nEnd of main function, all remaining products will be dropped.");
}
// Output
// Adding product to inventory: Laptop (ID: 1, Quantity: 10)
// Adding product to inventory: Smartphone (ID: 2, Quantity: 20)
// Adding product to inventory: Tablet (ID: 3, Quantity: 15)

// Initial inventory:
// Product: Product { id: 1, name: "Laptop", price: 999.99 }, Quantity: 10
// Product: Product { id: 3, name: "Tablet", price: 299.99 }, Quantity: 15
// Product: Product { id: 2, name: "Smartphone", price: 499.99 }, Quantity: 20

// Demonstrating Clone:
// Original product: Product { id: 1, name: "Laptop", price: 999.99 }
// Cloned product: Product { id: 1, name: "Laptop", price: 999.99 }
// Dropping product: Laptop (ID: 1)

// Processing order:
// Order ID: 1
// Products:
//   Product ID: 1, Quantity: 2
//   Product ID: 2, Quantity: 3

// Order 1 processed successfully

// Inventory after processing order 1:
// Product: Product { id: 1, name: "Laptop", price: 999.99 }, Quantity: 8
// Product: Product { id: 3, name: "Tablet", price: 299.99 }, Quantity: 15
// Product: Product { id: 2, name: "Smartphone", price: 499.99 }, Quantity: 17

// Processing order:
// Order ID: 2
// Products:
//   Product ID: 3, Quantity: 20

// Failed to process order 2: Insufficient stock for product 3

// Final inventory:
// Product: Product { id: 1, name: "Laptop", price: 999.99 }, Quantity: 8
// Product: Product { id: 3, name: "Tablet", price: 299.99 }, Quantity: 15
// Product: Product { id: 2, name: "Smartphone", price: 499.99 }, Quantity: 17

// Demonstrating Drop:
// Created product: Product { id: 4, name: "Headphones", price: 99.99 }
// Dropping product: Headphones (ID: 4)

// End of main function, all remaining products will be dropped.
// Dropping product: Laptop (ID: 1)
// Dropping product: Tablet (ID: 3)
// Dropping product: Smartphone (ID: 2)
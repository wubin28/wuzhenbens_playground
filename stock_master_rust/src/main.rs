use std::collections::HashMap;

#[derive(Debug, Clone)]
struct Product {
    id: u32,
    name: String,
    price: f64,
}

#[derive(Debug)]
struct Inventory {
    products: HashMap<u32, (Product, u32)>,
}

#[derive(Debug)]
struct Order {
    id: u32,
    products: Vec<(u32, u32)>, // (product_id, quantity)
}

impl Inventory {
    fn new() -> Self {
        Inventory {
            products: HashMap::new(),
        }
    }

    fn add_product(&mut self, product: Product, quantity: u32) {
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

    println!("Initial inventory:");
    for (_id, (product, quantity)) in &inventory.products {
        println!("Product: {:?}, Quantity: {}", product, quantity);
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
}
// Output:
// Initial inventory:
// Product: Product { id: 2, name: "Smartphone", price: 499.99 }, Quantity: 20
// Product: Product { id: 1, name: "Laptop", price: 999.99 }, Quantity: 10
// Product: Product { id: 3, name: "Tablet", price: 299.99 }, Quantity: 15
// Order 1 processed successfully

// Inventory after processing order 1:
// Product: Product { id: 2, name: "Smartphone", price: 499.99 }, Quantity: 17
// Product: Product { id: 1, name: "Laptop", price: 999.99 }, Quantity: 8
// Product: Product { id: 3, name: "Tablet", price: 299.99 }, Quantity: 15
// Failed to process order 2: Insufficient stock for product 3

// Final inventory:
// Product: Product { id: 2, name: "Smartphone", price: 499.99 }, Quantity: 17
// Product: Product { id: 1, name: "Laptop", price: 999.99 }, Quantity: 8
// Product: Product { id: 3, name: "Tablet", price: 299.99 }, Quantity: 15

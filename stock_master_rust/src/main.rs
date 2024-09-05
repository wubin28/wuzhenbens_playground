use std::collections::HashMap;

#[derive(Debug, Clone)]
struct Product {
    id: u32,
    name: String,
    price: f64,
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
    for (id, (product, quantity)) in &inventory.products {
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
    for (id, (product, quantity)) in &order_processor.inventory.products {
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
    for (id, (product, quantity)) in &order_processor.inventory.products {
        println!("Product: {:?}, Quantity: {}", product, quantity);
    }
}
// Output:
// Initial inventory:
// Product: Product { id: 2, name: "Smartphone", price: 499.99 }, Quantity: 20
// Product: Product { id: 3, name: "Tablet", price: 299.99 }, Quantity: 15
// Product: Product { id: 1, name: "Laptop", price: 999.99 }, Quantity: 10
// Order 1 processed successfully

// Inventory after processing order 1:
// Product: Product { id: 2, name: "Smartphone", price: 499.99 }, Quantity: 17
// Product: Product { id: 3, name: "Tablet", price: 299.99 }, Quantity: 15
// Product: Product { id: 1, name: "Laptop", price: 999.99 }, Quantity: 8
// Failed to process order 2: Insufficient stock for product 3

// Final inventory:
// Product: Product { id: 2, name: "Smartphone", price: 499.99 }, Quantity: 17
// Product: Product { id: 3, name: "Tablet", price: 299.99 }, Quantity: 15
// Product: Product { id: 1, name: "Laptop", price: 999.99 }, Quantity: 8

#[cfg(test)]
mod tests {
    use super::*;

    mod inventory_tests {
        use super::*;

        fn create_test_inventory() -> Inventory {
            let mut inventory = Inventory::new();
            inventory.add_product(
                Product {
                    id: 1,
                    name: "Test Product".to_string(),
                    price: 10.0,
                },
                5,
            );
            inventory
        }

        #[test]
        fn update_quantity_succeeds_for_existing_product() {
            // Given
            let mut inventory = create_test_inventory();
            let product_id = 1;
            let new_quantity = 10;

            // When
            let result = inventory.update_quantity(product_id, new_quantity);

            // Then
            assert!(result.is_ok());
            assert_eq!(inventory.get_quantity(product_id), Some(new_quantity));
        }

        #[test]
        fn update_quantity_fails_for_non_existent_product() {
            // Given
            let mut inventory = create_test_inventory();
            let non_existent_product_id = 999;
            let new_quantity = 10;

            // When
            let result = inventory.update_quantity(non_existent_product_id, new_quantity);

            // Then
            assert!(result.is_err());
            assert_eq!(
                result.unwrap_err(),
                format!("Product with id {} not found", non_existent_product_id)
            );
        }

        #[test]
        fn update_quantity_to_zero_is_allowed() {
            // Given
            let mut inventory = create_test_inventory();
            let product_id = 1;
            let new_quantity = 0;

            // When
            let result = inventory.update_quantity(product_id, new_quantity);

            // Then
            assert!(result.is_ok());
            assert_eq!(inventory.get_quantity(product_id), Some(new_quantity));
        }

        #[test]
        fn update_quantity_with_same_value_succeeds() {
            // Given
            let mut inventory = create_test_inventory();
            let product_id = 1;
            let original_quantity = inventory.get_quantity(product_id).unwrap();

            // When
            let result = inventory.update_quantity(product_id, original_quantity);

            // Then
            assert!(result.is_ok());
            assert_eq!(inventory.get_quantity(product_id), Some(original_quantity));
        }

        #[test]
        fn update_quantity_with_max_u32_value_succeeds() {
            // Given
            let mut inventory = create_test_inventory();
            let product_id = 1;
            let new_quantity = u32::MAX;

            // When
            let result = inventory.update_quantity(product_id, new_quantity);

            // Then
            assert!(result.is_ok());
            assert_eq!(inventory.get_quantity(product_id), Some(new_quantity));
        }
    }

    mod order_processor_tests {
        use super::*;

        fn create_test_inventory() -> Inventory {
            let mut inventory = Inventory::new();
            inventory.add_product(
                Product {
                    id: 1,
                    name: "Test Product 1".to_string(),
                    price: 10.0,
                },
                10,
            );
            inventory.add_product(
                Product {
                    id: 2,
                    name: "Test Product 2".to_string(),
                    price: 20.0,
                },
                5,
            );
            inventory
        }

        #[test]
        fn process_order_succeeds_with_sufficient_inventory() {
            // Given
            let inventory = create_test_inventory();
            let mut order_processor = OrderProcessor::new(inventory);
            let order = Order {
                id: 1,
                products: vec![(1, 5), (2, 2)],
            };

            // When
            let result = order_processor.process_order(order);

            // Then
            assert!(result.is_ok());
            assert_eq!(order_processor.inventory.get_quantity(1), Some(5));
            assert_eq!(order_processor.inventory.get_quantity(2), Some(3));
            assert_eq!(order_processor.orders.len(), 1);
        }

        #[test]
        fn process_order_fails_with_insufficient_inventory() {
            // Given
            let inventory = create_test_inventory();
            let mut order_processor = OrderProcessor::new(inventory);
            let order = Order {
                id: 1,
                products: vec![(1, 11)], // Requesting more than available
            };

            // When
            let result = order_processor.process_order(order);

            // Then
            assert!(result.is_err());
            assert_eq!(
                result.unwrap_err(),
                "Insufficient stock for product 1".to_string()
            );
            assert_eq!(order_processor.inventory.get_quantity(1), Some(10)); // Inventory unchanged
            assert_eq!(order_processor.orders.len(), 0);
        }

        #[test]
        fn process_order_fails_with_non_existent_product() {
            // Given
            let inventory = create_test_inventory();
            let mut order_processor = OrderProcessor::new(inventory);
            let order = Order {
                id: 1,
                products: vec![(3, 1)], // Product 3 doesn't exist
            };

            // When
            let result = order_processor.process_order(order);

            // Then
            assert!(result.is_err());
            assert_eq!(
                result.unwrap_err(),
                "Product with id 3 not found".to_string()
            );
            assert_eq!(order_processor.orders.len(), 0);
        }

        #[test]
        fn process_order_succeeds_with_zero_quantity() {
            // Given
            let inventory = create_test_inventory();
            let mut order_processor = OrderProcessor::new(inventory);
            let order = Order {
                id: 1,
                products: vec![(1, 0)],
            };

            // When
            let result = order_processor.process_order(order);

            // Then
            assert!(result.is_ok());
            assert_eq!(order_processor.inventory.get_quantity(1), Some(10)); // Inventory unchanged
            assert_eq!(order_processor.orders.len(), 1);
        }

        #[test]
        fn process_order_succeeds_with_multiple_products() {
            // Given
            let inventory = create_test_inventory();
            let mut order_processor = OrderProcessor::new(inventory);
            let order = Order {
                id: 1,
                products: vec![(1, 3), (2, 2)],
            };

            // When
            let result = order_processor.process_order(order);

            // Then
            assert!(result.is_ok());
            assert_eq!(order_processor.inventory.get_quantity(1), Some(7));
            assert_eq!(order_processor.inventory.get_quantity(2), Some(3));
            assert_eq!(order_processor.orders.len(), 1);
        }

        #[test]
        fn process_order_fails_if_any_product_has_insufficient_quantity() {
            // Given
            let inventory = create_test_inventory();
            let mut order_processor = OrderProcessor::new(inventory);
            let order = Order {
                id: 1,
                products: vec![(1, 5), (2, 6)], // Product 2 has insufficient quantity
            };

            // When
            let result = order_processor.process_order(order);

            // Then
            assert!(result.is_err());
            assert_eq!(
                result.unwrap_err(),
                "Insufficient stock for product 2".to_string()
            );
            assert_eq!(order_processor.inventory.get_quantity(1), Some(10)); // Inventory unchanged
            assert_eq!(order_processor.inventory.get_quantity(2), Some(5)); // Inventory unchanged
            assert_eq!(order_processor.orders.len(), 0);
        }
    }
}

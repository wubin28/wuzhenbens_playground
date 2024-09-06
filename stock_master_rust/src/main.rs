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

#[derive(Debug, Clone)]
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

    fn get_product_order_history(&self, product_id: u32) -> Vec<Order> {
        self.orders
            .iter()
            .filter(|order| order.products.iter().any(|(id, _)| *id == product_id))
            .cloned()
            .collect()
    }
}

fn print_order_history(history: &[Order]) {
    for order in history {
        println!("Order ID: {}", order.id);
        for (product_id, quantity) in &order.products {
            println!("  Product ID: {}, Quantity: {}", product_id, quantity);
        }
    }
}

fn main() {
    let laptop_history;
    {
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

        let mut order_processor = OrderProcessor::new(inventory);

        // 创建并处理订单
        let order1 = Order {
            id: 1,
            products: vec![(1, 2)],
        };
        order_processor.process_order(order1).unwrap();

        // 尝试获取产品订单历史并存储在外部变量中
        laptop_history = order_processor.get_product_order_history(1);
    } // order_processor 的生存期在这里结束

    // 尝试使用 laptop_history，这里会出现编译错误
    print_order_history(&laptop_history);
}
// Output:
// Order ID: 1
//   Product ID: 1, Quantity: 2

#[cfg(test)]
mod tests {
    use super::*;

    mod inventory_update_quantity_tests {
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

    mod inventory_add_product_tests {
        use super::*;

        #[test]
        fn add_new_product_increases_inventory_size() {
            // Given
            let mut inventory = Inventory::new();
            let initial_size = inventory.products.len();
            let new_product = Product {
                id: 1,
                name: "Test Product".to_string(),
                price: 10.0,
            };

            // When
            inventory.add_product(new_product.clone(), 5);

            // Then
            assert_eq!(inventory.products.len(), initial_size + 1);
            assert!(inventory.products.contains_key(&new_product.id));
        }

        #[test]
        fn add_existing_product_updates_quantity() {
            // Given
            let mut inventory = Inventory::new();
            let product = Product {
                id: 1,
                name: "Test Product".to_string(),
                price: 10.0,
            };
            inventory.add_product(product.clone(), 5);

            // When
            inventory.add_product(product.clone(), 3);

            // Then
            assert_eq!(inventory.products.len(), 1);
            assert_eq!(inventory.get_quantity(product.id), Some(3));
        }

        #[test]
        fn add_product_with_zero_quantity() {
            // Given
            let mut inventory = Inventory::new();
            let product = Product {
                id: 1,
                name: "Test Product".to_string(),
                price: 10.0,
            };

            // When
            inventory.add_product(product.clone(), 0);

            // Then
            assert!(inventory.products.contains_key(&product.id));
            assert_eq!(inventory.get_quantity(product.id), Some(0));
        }

        #[test]
        fn add_product_with_max_quantity() {
            // Given
            let mut inventory = Inventory::new();
            let product = Product {
                id: 1,
                name: "Test Product".to_string(),
                price: 10.0,
            };

            // When
            inventory.add_product(product.clone(), u32::MAX);

            // Then
            assert!(inventory.products.contains_key(&product.id));
            assert_eq!(inventory.get_quantity(product.id), Some(u32::MAX));
        }

        #[test]
        fn add_multiple_products_with_different_ids() {
            // Given
            let mut inventory = Inventory::new();
            let product1 = Product {
                id: 1,
                name: "Product 1".to_string(),
                price: 10.0,
            };
            let product2 = Product {
                id: 2,
                name: "Product 2".to_string(),
                price: 20.0,
            };

            // When
            inventory.add_product(product1.clone(), 5);
            inventory.add_product(product2.clone(), 3);

            // Then
            assert_eq!(inventory.products.len(), 2);
            assert_eq!(inventory.get_quantity(product1.id), Some(5));
            assert_eq!(inventory.get_quantity(product2.id), Some(3));
        }
    }

    mod inventory_get_product_tests {
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
        fn get_product_returns_some_for_existing_product() {
            // Given
            let inventory = create_test_inventory();
            let product_id = 1;

            // When
            let result = inventory.get_product(product_id);

            // Then
            assert!(result.is_some());
            assert_eq!(result.unwrap().id, product_id);
            assert_eq!(result.unwrap().name, "Test Product");
            assert_eq!(result.unwrap().price, 10.0);
        }

        #[test]
        fn get_product_returns_none_for_non_existent_product() {
            // Given
            let inventory = create_test_inventory();
            let non_existent_product_id = 999;

            // When
            let result = inventory.get_product(non_existent_product_id);

            // Then
            assert!(result.is_none());
        }

        #[test]
        fn get_product_returns_correct_product_for_multiple_products() {
            // Given
            let mut inventory = create_test_inventory();
            inventory.add_product(
                Product {
                    id: 2,
                    name: "Another Product".to_string(),
                    price: 20.0,
                },
                3,
            );

            // When
            let result = inventory.get_product(2);

            // Then
            assert!(result.is_some());
            assert_eq!(result.unwrap().id, 2);
            assert_eq!(result.unwrap().name, "Another Product");
            assert_eq!(result.unwrap().price, 20.0);
        }

        #[test]
        fn get_product_returns_product_without_affecting_quantity() {
            // Given
            let inventory = create_test_inventory();
            let product_id = 1;
            let initial_quantity = inventory.get_quantity(product_id).unwrap();

            // When
            let _ = inventory.get_product(product_id);

            // Then
            assert_eq!(
                inventory.get_quantity(product_id).unwrap(),
                initial_quantity
            );
        }

        #[test]
        fn get_product_returns_same_reference_for_multiple_calls() {
            // Given
            let inventory = create_test_inventory();
            let product_id = 1;

            // When
            let result1 = inventory.get_product(product_id);
            let result2 = inventory.get_product(product_id);

            // Then
            assert!(result1.is_some() && result2.is_some());
            assert_eq!(
                result1.unwrap() as *const Product,
                result2.unwrap() as *const Product
            );
        }
    }

    mod inventory_get_quantity_tests {
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
        fn get_quantity_returns_some_for_existing_product() {
            // Given
            let inventory = create_test_inventory();
            let product_id = 1;

            // When
            let result = inventory.get_quantity(product_id);

            // Then
            assert_eq!(result, Some(5));
        }

        #[test]
        fn get_quantity_returns_none_for_non_existent_product() {
            // Given
            let inventory = create_test_inventory();
            let non_existent_product_id = 999;

            // When
            let result = inventory.get_quantity(non_existent_product_id);

            // Then
            assert_eq!(result, None);
        }

        #[test]
        fn get_quantity_returns_correct_quantity_for_multiple_products() {
            // Given
            let mut inventory = create_test_inventory();
            inventory.add_product(
                Product {
                    id: 2,
                    name: "Another Product".to_string(),
                    price: 20.0,
                },
                3,
            );

            // When
            let result1 = inventory.get_quantity(1);
            let result2 = inventory.get_quantity(2);

            // Then
            assert_eq!(result1, Some(5));
            assert_eq!(result2, Some(3));
        }

        #[test]
        fn get_quantity_returns_zero_for_product_with_zero_quantity() {
            // Given
            let mut inventory = create_test_inventory();
            inventory.update_quantity(1, 0).unwrap();

            // When
            let result = inventory.get_quantity(1);

            // Then
            assert_eq!(result, Some(0));
        }

        #[test]
        fn get_quantity_returns_max_value_for_product_with_max_quantity() {
            // Given
            let mut inventory = create_test_inventory();
            inventory.update_quantity(1, u32::MAX).unwrap();

            // When
            let result = inventory.get_quantity(1);

            // Then
            assert_eq!(result, Some(u32::MAX));
        }

        #[test]
        fn get_quantity_is_consistent_after_multiple_calls() {
            // Given
            let inventory = create_test_inventory();
            let product_id = 1;

            // When
            let result1 = inventory.get_quantity(product_id);
            let result2 = inventory.get_quantity(product_id);

            // Then
            assert_eq!(result1, result2);
        }
    }
}

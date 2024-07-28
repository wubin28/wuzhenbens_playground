package org.example;

import java.util.concurrent.ArrayBlockingQueue;
import java.util.concurrent.BlockingQueue;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicInteger;

public class InventoryManager {
  final AtomicInteger inventory;
  private final BlockingQueue<Integer> changesQueue;
  private final BlockingQueue<Boolean> restockQueue;
  private final int threshold;
  final ExecutorService executorService;

  public InventoryManager(int initialInventory, int threshold) {
    this.inventory = new AtomicInteger(initialInventory);
    this.changesQueue = new ArrayBlockingQueue<>(60);
    this.restockQueue = new ArrayBlockingQueue<>(1);
    this.threshold = threshold;
    this.executorService = Executors.newCachedThreadPool();

    startInventoryMonitor();
  }

  private void startInventoryMonitor() {
    executorService.submit(() -> {
      while (!Thread.currentThread().isInterrupted()) {
        if (inventory.get() < threshold) {
          restockQueue.offer(true);
        }
        try {
          Thread.sleep(500);
        } catch (InterruptedException e) {
          Thread.currentThread().interrupt();
        }
      }
    });
  }

  public boolean addToCart() {
    int current = inventory.get();
    if (current > 0 && inventory.compareAndSet(current, current - 1)) {
      changesQueue.offer(-1);
      return true;
    }

    // Not thread-safe
//    if (inventory.get() > 0) {
//      inventory.decrementAndGet();
//      changesQueue.offer(-1);
//      return true;
//    }

    return false;
  }

  public void removeFromCart() {
    inventory.incrementAndGet();

    // Not thread-safe
//    inventory.set(inventory.get() + 1);

    changesQueue.offer(1);
  }

  public void confirmOrder(int quantity) {
    System.out.println("Order confirmed for " + quantity + " items");
  }

  public void startAnalytics() {
    executorService.submit(() -> {
      while (!Thread.currentThread().isInterrupted()) {
        try {
          int change = changesQueue.take();
          System.out.println("Inventory change: " + change);
        } catch (InterruptedException e) {
          Thread.currentThread().interrupt();
        }
      }
    });
  }

  public void startRestockMonitor() {
    executorService.submit(() -> {
      while (!Thread.currentThread().isInterrupted()) {
        try {
          restockQueue.take();
          System.out.println("Triggering restock process");
        } catch (InterruptedException e) {
          Thread.currentThread().interrupt();
        }
      }
    });
  }

  public void simulateUser(String userType) {
    switch (userType) {
      case "reader":
        System.out.println("Current inventory: " + inventory.get());
        break;
      case "buyer":
        if (addToCart()) {
          confirmOrder(1);
          System.out.println("Item added to cart and order confirmed");
        } else {
          System.out.println("Failed to add to cart: Out of stock");
        }
        break;
    }
  }

  public static void main(String[] args) throws InterruptedException {
    InventoryManager manager = new InventoryManager(10, 5);
    manager.startAnalytics();
    manager.startRestockMonitor();

    ExecutorService userSimulator = Executors.newFixedThreadPool(18);
    for (int i = 0; i < 9; i++) {
      userSimulator.submit(() -> manager.simulateUser("reader"));
      userSimulator.submit(() -> manager.simulateUser("buyer"));
    }

    userSimulator.shutdown();
    userSimulator.awaitTermination(5, TimeUnit.SECONDS);

    Thread.sleep(3000);

    System.out.println("Final inventory: " + manager.inventory.get());

    manager.executorService.shutdownNow();
    System.exit(0);
  }
}
// Output:
//Order confirmed for 1 items
//Item added to cart and order confirmed
//Current inventory: 5
//Current inventory: 10
//Current inventory: 7
//Order confirmed for 1 items
//Item added to cart and order confirmed
//Order confirmed for 1 items
//Item added to cart and order confirmed
//Inventory change: -1
//Inventory change: -1
//Inventory change: -1
//Order confirmed for 1 items
//Item added to cart and order confirmed
//Current inventory: 3
//Order confirmed for 1 items
//Item added to cart and order confirmed
//Current inventory: 9
//Current inventory: 2
//Order confirmed for 1 items
//Item added to cart and order confirmed
//Order confirmed for 1 items
//Item added to cart and order confirmed
//Order confirmed for 1 items
//Item added to cart and order confirmed
//Current inventory: 4
//Order confirmed for 1 items
//Item added to cart and order confirmed
//Current inventory: 8
//Current inventory: 6
//Inventory change: -1
//Inventory change: -1
//Inventory change: -1
//Inventory change: -1
//Inventory change: -1
//Inventory change: -1
//Triggering restock process
//Triggering restock process
//Triggering restock process
//Triggering restock process
//Triggering restock process
//Triggering restock process
//Final inventory: 1
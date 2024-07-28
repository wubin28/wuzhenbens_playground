package org.example;

import static org.junit.jupiter.api.Assertions.*;

import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicInteger;
import org.junit.jupiter.api.AfterEach;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

class InventoryManagerTest {

  private InventoryManager inventoryManager;
  private static final int INITIAL_INVENTORY = 1000;
  private static final int THRESHOLD = 100;
  private static final int THREAD_COUNT = 100;
  private static final int OPERATIONS_PER_THREAD = 100;

  @BeforeEach
  void setUp() {
    inventoryManager = new InventoryManager(INITIAL_INVENTORY, THRESHOLD);
  }

  @AfterEach
  void tearDown() throws InterruptedException {
    inventoryManager.executorService.shutdownNow();
    inventoryManager.executorService.awaitTermination(5, TimeUnit.SECONDS);
  }

  @Test
  void testConcurrentInventoryOperations() throws InterruptedException {
    ExecutorService executorService = Executors.newFixedThreadPool(THREAD_COUNT);
    AtomicInteger successfulAddToCart = new AtomicInteger(0);
    AtomicInteger failedAddToCart = new AtomicInteger(0);

    for (int i = 0; i < THREAD_COUNT; i++) {
      executorService.submit(
          () -> {
            for (int j = 0; j < OPERATIONS_PER_THREAD; j++) {
              if (inventoryManager.addToCart()) {
                successfulAddToCart.incrementAndGet();
              } else {
                failedAddToCart.incrementAndGet();
              }

              if (j % 2 == 0) {
                inventoryManager.removeFromCart();
              }
            }
          });
    }

    executorService.shutdown();
    executorService.awaitTermination(30, TimeUnit.SECONDS);

    int expectedInventoryChange =
        successfulAddToCart.get() - (THREAD_COUNT * OPERATIONS_PER_THREAD / 2);
    int actualInventoryChange = INITIAL_INVENTORY - inventoryManager.inventory.get();

    assertEquals(
        expectedInventoryChange,
        actualInventoryChange,
        "The inventory change should match the number of successful addToCart operations minus"
            + " removeFromCart operations");

    assertTrue(
        failedAddToCart.get() > 0,
        "There should be some failed addToCart operations due to concurrency");

    assertTrue(inventoryManager.inventory.get() >= 0, "The inventory should never become negative");
  }
}

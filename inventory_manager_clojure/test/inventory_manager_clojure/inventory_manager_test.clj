(ns inventory-manager-clojure.inventory-manager-test
  (:require [clojure.test :refer :all])
  (:require [clojure.core.async :as async])
  (:require [inventory-manager-clojure.inventory-manager :refer :all]))

(deftest test-create-inventory-manager
  (let [manager (create-inventory-manager 10 2)]
    (is (= 10 @(:inventory manager)))
    (is (instance? clojure.core.async.impl.channels.ManyToManyChannel (:changes manager)))
    (is (instance? clojure.core.async.impl.channels.ManyToManyChannel (:restock manager)))))

(deftest test-add-to-cart
  (let [manager (create-inventory-manager 5 2)]
    (is (true? (add-to-cart manager)))
    (is (= 4 @(:inventory manager)))
    (is (false? (add-to-cart (create-inventory-manager 0 2))))))  ; 使用 create-inventory-manager 创建空库存

(deftest test-remove-from-cart
  (let [manager (create-inventory-manager 5 2)]
    (remove-from-cart manager)
    (is (= 6 @(:inventory manager)))))

(deftest test-concurrent-operations
  (let [manager (create-inventory-manager 100 2)
        iterations 1000
        threads 10]
    (dotimes [_ threads]
      (async/go
        (dotimes [_ iterations]
          (add-to-cart manager)
          (remove-from-cart manager))))

    ; 等待所有操作完成
    (Thread/sleep 5000)

    (is (= 100 @(:inventory manager)) "Inventory should remain unchanged after equal adds and removes")))

(deftest test-restock-trigger
  (let [manager (create-inventory-manager 3 2)
        restock-triggered (atom false)]
    (async/go
      (async/<! (:restock manager))
      (reset! restock-triggered true))

    (dotimes [_ 2]
      (add-to-cart manager))

    ; 等待重新进货触发
    (Thread/sleep 1500)

    (is (true? @restock-triggered) "Restock should be triggered when inventory falls below threshold")))

(deftest test-analytics
  (let [manager (create-inventory-manager 10 2)
        changes (atom [])
        analytics-complete (promise)]
    (async/go
      (loop []
        (when-let [change (async/<! (:changes manager))]
          (swap! changes conj change)
          (recur)))
      (deliver analytics-complete true))

    (add-to-cart manager)
    (remove-from-cart manager)

    (async/close! (:changes manager))

    ; 等待分析完成
    (deref analytics-complete 1000 false)

    (is (= [-1 1] @changes) "Analytics should record all inventory changes")))
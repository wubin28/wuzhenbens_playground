(ns inventory-manager-clojure.inventory-manager
  (:require [clojure.core.async :as async])
  (:gen-class))

(defn create-inventory-manager [initial-inventory threshold]
  (let [inventory-atom (atom initial-inventory)
        changes-chan (async/chan (async/sliding-buffer 60))
        restock-chan (async/chan)]

    (async/go-loop []
      (when (< @inventory-atom threshold)
        (async/>! restock-chan :trigger-restock))
      (async/<! (async/timeout 1000))
      (recur))

    {:inventory inventory-atom
     :changes changes-chan
     :restock restock-chan}))

(defn add-to-cart [manager]
  (let [current @(:inventory manager)]
    (if (pos? current)
      (do
        (swap! (:inventory manager) dec)
        (async/>!! (:changes manager) -1)
        true)
      false)))  ; 明确返回 false 而不是 nil

(defn remove-from-cart [manager]
  (swap! (:inventory manager) inc)
  (async/>!! (:changes manager) 1))

(defn confirm-order [manager quantity]
  (println "Order confirmed for " quantity " items"))

(defn start-analytics [manager]
  (async/go-loop []
    (let [change (async/<! (:changes manager))]
      (println "Inventory change:" change))
    (recur)))

(defn start-restock-monitor [manager]
  (async/go-loop []
    (when (async/<! (:restock manager))
      (println "Triggering restock process"))
    (recur)))

(defn simulate-user [manager user-type]
  (case user-type
    :reader (println "Current inventory: " @(:inventory manager))
    :buyer (if (add-to-cart manager)
             (do
               (confirm-order manager 1)
               (println "Item added to cart and order confirmed"))
             (println "Failed to add to cart: Out of stock"))))

(defn -main []
  (let [manager (create-inventory-manager 10 2)]
    (start-analytics manager)
    (start-restock-monitor manager)

    (dotimes [_ 9]
      (async/go (simulate-user manager :reader))
      (async/go (simulate-user manager :buyer)))

    (Thread/sleep 5000)

    (println "Final inventory:" @(:inventory manager))

    (System/exit 0)))

; Run the main function
; (-main)
; Output:
;Current inventory: Current inventory:   Current inventory:  1010
;
;10
;Current inventory:  7
;Current inventory:  Current inventory: 6
;5
;Current inventory:  4
;Order confirmed for  1 Order confirmed for  1  items
;Item added to cart and order confirmed
;Order confirmed for  1 Current inventory:  3
;Order confirmed for  1Order confirmed for  1  items
;Item added to cart and order confirmed
;Order confirmed for  1 Order confirmed for  1  items items
;Order confirmed for  1  items
;Item added to cart and order confirmed
;items
;Item added to cart and order confirmed
;items
;Item added to cart and order confirmed
;Current inventory:  2
;Order confirmed for  1 items
;
;Item added to cart and order confirmed
;Inventory change: -1
;Item added to cart and order confirmed items
;
;Item added to cart and order confirmed
;Item added to cart and order confirmed
;Inventory change: -1
;Inventory change: -1
;Inventory change: -1
;Inventory change: -1
;Inventory change: -1
;Inventory change: -1
;Inventory change: -1
;Inventory change: -1
;Triggering restock process
;Triggering restock process
;Triggering restock process
;Triggering restock process
;Final inventory: 1
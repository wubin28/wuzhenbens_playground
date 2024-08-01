(ns movie-booking-clojure.movie-booking
  (:require [clojure.core.async :as async]))

(defn create-movie-theater [total-seats]
  (atom (vec (repeat total-seats false))))

(defn get-available-seats [theater]
  ;; 失误1：直接返回内部状态
  ;; Clojure 中难以重现，因为：
  ;; 1. @theater 返回的是不可变的 vector 的副本
  ;; 2. 函数返回的是新创建的 sequence，而不是直接暴露内部状态
  ;; 失误2：忘记加锁
  ;; Clojure 中难以重现，因为：
  ;; 1. 使用了不可变数据结构，不需要显式加锁
  ;; 2. 读取操作 (@theater) 是原子的，不会出现不一致状态
  (->> @theater
       (map-indexed (fn [idx seat] (when-not seat (inc idx))))
       (remove nil?)))

(defn book-seat [theater seat-number]
  ;; 失误3：在锁内部调用可能长时间阻塞的操作
  ;; Clojure 中难以重现，因为：
  ;; 1. 使用了 compare-and-set! 进行原子操作，不需要显式加锁
  ;; 2. 函数式编程鼓励将副作用分离，减少了在关键部分执行长时间操作的可能性
  (let [idx (dec seat-number)]
    (when (< -1 idx (count @theater))
      (loop []
        (let [current-state @theater]
          (if (get current-state idx)
            false  ; 座位已被预订
            (if (compare-and-set! theater current-state (assoc current-state idx true))
              true  ; 预订成功
              (recur))))))))  ; 重试

(defn cancel-booking [theater seat-number]
  (let [idx (dec seat-number)]
    (when (< -1 idx (count @theater))
      (reset! theater (assoc @theater idx false))
      true)))

(defrecord Booking [seat-number paid?])
;; 失误4：公开可变字段
;; Clojure 中难以重现，因为：
;; 1. Clojure 的 record 是不可变的数据结构
;; 2. 修改 Booking 实例需要创建新的实例，而不是直接修改字段

(defn create-booking-system [total-seats]
  (let [theater (create-movie-theater total-seats)]
    {:theater theater
     :bookings (atom [])}))

(defn make-booking [{:keys [theater bookings]} seat-number]
  ;; 失误5：忘记加锁且在锁外部修改共享资源
  ;; Clojure 中难以重现，因为：
  ;; 1. 使用 swap! 进行原子操作，确保了线程安全
  ;; 2. 不可变数据结构和函数式更新消除了显式锁的需求
  (when (book-seat theater seat-number)
    (swap! bookings conj (->Booking seat-number false))
    true))

(defn cancel-booking-system [{:keys [theater bookings]} seat-number]
  (when (cancel-booking theater seat-number)
    (swap! bookings (fn [bs] (remove #(and (= (:seat-number %) seat-number)
                                           (not (:paid? %))) bs)))
    true))

(defn pay-for-booking [{:keys [bookings]} seat-number]
  (let [payment-successful (atom false)]
    (swap! bookings (fn [bs]
                      (mapv (fn [b]
                             (if (and (= (:seat-number b) seat-number)
                                      (not (:paid? b)))
                               (do (reset! payment-successful true)
                                   (assoc b :paid? true))
                               b))
                           bs)))
    @payment-successful))

(def print-lock (Object.))

(defn safe-println [& args]
  (locking print-lock
    (apply println args)
    (flush)))

(defn simulate-user [booking-system user-name]
  (async/go
    (try
      (let [available-seats (get-available-seats (:theater booking-system))]
        (safe-println user-name "查看可用座位:" available-seats)

        (when (seq available-seats)
          (let [seat-to-book (rand-nth available-seats)
                booked (make-booking booking-system seat-to-book)]
            (safe-println user-name "尝试预订座位" seat-to-book ":" (if booked "成功" "失败"))

            (when booked
              (if (< (rand) 0.5)
                (let [paid (pay-for-booking booking-system seat-to-book)]
                  (safe-println user-name "尝试支付座位" seat-to-book ":" (if paid "成功" "失败")))
                (let [cancelled (cancel-booking-system booking-system seat-to-book)]
                  (safe-println user-name "尝试取消预订座位" seat-to-book ":" (if cancelled "成功" "失败")))))))

        (let [available-seats (get-available-seats (:theater booking-system))]
          (safe-println user-name "再次查看可用座位:" available-seats)))
      (catch Exception e
        (safe-println user-name "遇到错误:" (.getMessage e))))))

(defn -main []
  (let [booking-system (create-booking-system 10)]
    (doall (for [i (range 5)]
             (simulate-user booking-system (str "用户" (inc i)))))
    (Thread/sleep 5000)))
; Output:
;用户5 查看可用座位: (1 2 3 4 5 6 7 8 9 10)
;用户4 查看可用座位: (1 2 3 4 5 6 7 8 9 10)
;用户5 尝试预订座位 9 : 成功
;用户2 查看可用座位: (1 2 3 4 5 6 7 8 9 10)
;用户1 查看可用座位: (1 2 3 4 5 6 7 8 9 10)
;用户3 查看可用座位: (1 2 3 4 5 6 7 8 9 10)
;用户3 尝试预订座位 10 : 失败
;用户3 再次查看可用座位: (2 3 4 5 6 8)
;用户1 尝试预订座位 10 : 成功
;用户2 尝试预订座位 1 : 成功
;用户5 尝试支付座位 9 : 失败
;用户5 再次查看可用座位: (2 3 4 5 6 8)
;用户4 尝试预订座位 7 : 成功
;用户2 尝试取消预订座位 1 : 成功
;用户2 再次查看可用座位: (1 2 3 4 5 6 7 8 10)
;用户4 尝试取消预订座位 7 : 成功
;用户4 再次查看可用座位: (1 2 3 4 5 6 7 8 10)
;用户1 尝试取消预订座位 10 : 成功
;用户1 再次查看可用座位: (1 2 3 4 5 6 7 8 10)

(ns movie-booking-clojure.movie-booking-test
  (:require [clojure.test :refer :all]
            [movie-booking-clojure.movie-booking :refer :all]))

;; 测试创建电影院
(deftest create-movie-theater-test
  (testing "创建电影院with正确的座位数"
    (let [theater (create-movie-theater 5)]
      (is (= 5 (count @theater)))
      (is (every? false? @theater)))))

;; 测试获取可用座位
(deftest get-available-seats-test
  (testing "获取所有可用座位"
    (let [theater (atom [false false false])]
      (is (= [1 2 3] (get-available-seats theater)))))

  (testing "获取部分可用座位"
    (let [theater (atom [false true false])]
      (is (= [1 3] (get-available-seats theater)))))

  (testing "没有可用座位"
    (let [theater (atom [true true true])]
      (is (empty? (get-available-seats theater))))))

;; 测试预订座位
(deftest book-seat-test
  (testing "预订可用座位"
    (let [theater (create-movie-theater 3)]
      (is (book-seat theater 2))
      (is (= [false true false] @theater))))

  (testing "预订已被预订的座位"
    (let [theater (atom [false true false])]
      (is (not (book-seat theater 2)))))

  (testing "预订不存在的座位"
    (let [theater (create-movie-theater 3)]
      (is (nil? (book-seat theater 0)))
      (is (nil? (book-seat theater 4))))))

;; 测试取消预订
(deftest cancel-booking-test
  (testing "取消已预订的座位"
    (let [theater (atom [false true false])]
      (is (cancel-booking theater 2))
      (is (= [false false false] @theater))))

  (testing "取消未预订的座位"
    (let [theater (atom [false false false])]
      (is (cancel-booking theater 2))
      (is (= [false false false] @theater))))

  (testing "取消不存在的座位"
    (let [theater (create-movie-theater 3)]
      (is (nil? (cancel-booking theater 0)))
      (is (nil? (cancel-booking theater 4))))))

;; 测试创建预订系统
(deftest create-booking-system-test
  (testing "创建预订系统"
    (let [booking-system (create-booking-system 5)]
      (is (= 5 (count @(:theater booking-system))))
      (is (empty? @(:bookings booking-system))))))

;; 测试进行预订
(deftest make-booking-test
  (testing "预订可用座位"
    (let [booking-system (create-booking-system 3)]
      (is (make-booking booking-system 2))
      (is (= 1 (count @(:bookings booking-system))))
      (is (= 2 (:seat-number (first @(:bookings booking-system)))))
      (is (not (:paid? (first @(:bookings booking-system)))))))

  (testing "预订已被预订的座位"
    (let [booking-system (create-booking-system 3)]
      (make-booking booking-system 2)
      (is (not (make-booking booking-system 2))))))

;; 测试取消预订
(deftest cancel-booking-system-test
  (testing "取消未支付的预订"
    (let [booking-system (create-booking-system 3)]
      (make-booking booking-system 2)
      (is (cancel-booking-system booking-system 2))
      (is (empty? @(:bookings booking-system)))))

  (testing "取消已支付的预订"
    (let [booking-system (create-booking-system 3)]
      (make-booking booking-system 2)
      (pay-for-booking booking-system 2)
      (is (cancel-booking-system booking-system 2))
      (is (= 1 (count @(:bookings booking-system)))))))

;; 测试支付预订
(deftest pay-for-booking-test
  (testing "支付未支付的预订"
    (let [booking-system (create-booking-system 3)]
      (make-booking booking-system 2)
      (is (pay-for-booking booking-system 2))
      (is (:paid? (first @(:bookings booking-system))))))

  (testing "支付已支付的预订"
    (let [booking-system (create-booking-system 3)]
      (make-booking booking-system 2)
      (pay-for-booking booking-system 2)
      (is (not (pay-for-booking booking-system 2)))))

  (testing "支付不存在的预订"
    (let [booking-system (create-booking-system 3)]
      (is (not (pay-for-booking booking-system 2))))))
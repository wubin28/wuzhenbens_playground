package org.example;

import java.util.ArrayList;
import java.util.List;
import java.util.Random;
import java.util.concurrent.locks.Lock;
import java.util.concurrent.locks.ReentrantLock;

public class MovieTheater {
  private final int totalSeats;
  private List<Boolean> seats;
  private Lock lock = new ReentrantLock();

  public MovieTheater(int totalSeats) {
    this.totalSeats = totalSeats;
    this.seats = new ArrayList<>(totalSeats);
    for (int i = 0; i < totalSeats; i++) {
      seats.add(false);
    }
  }

  public List<Integer> getAvailableSeats() {
    List<Integer> availableSeats = new ArrayList<>();
    lock.lock();
    try {
      for (int i = 0; i < totalSeats; i++) {
        if (!seats.get(i)) {
          availableSeats.add(i + 1);
        }
      }
    } finally {
      lock.unlock();
    }
    return availableSeats;
  }

  public boolean bookSeat(int seatNumber) {
    if (seatNumber < 1 || seatNumber > totalSeats) {
      return false;
    }
    lock.lock();
    try {
      if (!seats.get(seatNumber - 1)) {
        seats.set(seatNumber - 1, true);
        return true;
      }
      return false;
    } finally {
      lock.unlock();
    }
  }

  public boolean cancelBooking(int seatNumber) {
    if (seatNumber < 1 || seatNumber > totalSeats) {
      return false;
    }
    lock.lock();
    try {
      if (seats.get(seatNumber - 1)) {
        seats.set(seatNumber - 1, false);
        return true;
      }
      return false;
    } finally {
      lock.unlock();
    }
  }

  public static void main(String[] args) {
    BookingSystem bookingSystem = new BookingSystem(10); // 创建一个有10个座位的影院
    Random random = new Random();

    Runnable userTask =
        () -> {
          String userName = Thread.currentThread().getName();
          try {
            // 查看可用座位
            List<Integer> availableSeats = bookingSystem.getAvailableSeats();
            System.out.println(userName + " 查看可用座位: " + availableSeats);

            if (!availableSeats.isEmpty()) {
              // 随机选择一个座位进行预订
              int seatToBook = availableSeats.get(random.nextInt(availableSeats.size()));
              boolean booked = bookingSystem.makeBooking(seatToBook);
              System.out.println(
                  userName + " 尝试预订座位 " + seatToBook + ": " + (booked ? "成功" : "失败"));

              if (booked) {
                // 50%的概率支付订单
                if (random.nextBoolean()) {
                  boolean paid = bookingSystem.payForBooking(seatToBook);
                  System.out.println(
                      userName + " 尝试支付座位 " + seatToBook + ": " + (paid ? "成功" : "失败"));
                } else {
                  // 50%的概率取消预订
                  boolean cancelled = bookingSystem.cancelBooking(seatToBook);
                  System.out.println(
                      userName + " 尝试取消预订座位 " + seatToBook + ": " + (cancelled ? "成功" : "失败"));
                }
              }
            }

            // 再次查看可用座位
            availableSeats = bookingSystem.getAvailableSeats();
            System.out.println(userName + " 再次查看可用座位: " + availableSeats);

          } catch (Exception e) {
            System.out.println(userName + " 遇到错误: " + e.getMessage());
          }
        };

    // 创建5个线程模拟5个并发用户
    for (int i = 0; i < 5; i++) {
      new Thread(userTask, "用户" + (i + 1)).start();
    }
  }
}

class Booking {
  private int seatNumber;
  private boolean isPaid = false;

  public Booking(int seatNumber) {
    this.seatNumber = seatNumber;
  }

  public void pay() {
    isPaid = true;
  }

  public boolean isPaid() {
    return isPaid;
  }

  public int getSeatNumber() {
    return seatNumber;
  }
}

class BookingSystem {
  private MovieTheater theater;
  private List<Booking> bookings = new ArrayList<>();
  private Lock bookingLock = new ReentrantLock();

  public BookingSystem(int totalSeats) {
    this.theater = new MovieTheater(totalSeats);
  }

  public boolean makeBooking(int seatNumber) {
    if (theater.bookSeat(seatNumber)) {
      bookingLock.lock();
      try {
        bookings.add(new Booking(seatNumber));
      } finally {
        bookingLock.unlock();
      }
      return true;
    }
    return false;
  }

  public boolean cancelBooking(int seatNumber) {
    if (theater.cancelBooking(seatNumber)) {
      bookingLock.lock();
      try {
        bookings.removeIf(booking -> booking.getSeatNumber() == seatNumber && !booking.isPaid());
      } finally {
        bookingLock.unlock();
      }
      return true;
    }
    return false;
  }

  public boolean payForBooking(int seatNumber) {
    bookingLock.lock();
    try {
      for (Booking booking : bookings) {
        if (booking.getSeatNumber() == seatNumber && !booking.isPaid()) {
          booking.pay();
          return true;
        }
      }
    } finally {
      bookingLock.unlock();
    }
    return false;
  }

  public List<Integer> getAvailableSeats() {
    return theater.getAvailableSeats();
  }
}
// Output:
//用户1 查看可用座位: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
//用户2 查看可用座位: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
//用户4 查看可用座位: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
//用户3 查看可用座位: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
//用户5 查看可用座位: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
//用户3 尝试预订座位 7: 失败
//用户1 尝试预订座位 2: 成功
//用户5 尝试预订座位 5: 成功
//用户1 尝试支付座位 2: 成功
//用户1 再次查看可用座位: [1, 4, 5, 6, 8, 9, 10]
//用户3 再次查看可用座位: [1, 4, 6, 8, 9, 10]
//用户4 尝试预订座位 3: 成功
//用户2 尝试预订座位 7: 成功
//用户2 尝试支付座位 7: 成功
//用户5 尝试取消预订座位 5: 成功
//用户2 再次查看可用座位: [1, 3, 4, 5, 6, 8, 9, 10]
//用户5 再次查看可用座位: [1, 3, 4, 5, 6, 8, 9, 10]
//用户4 尝试取消预订座位 3: 成功
//用户4 再次查看可用座位: [1, 3, 4, 5, 6, 8, 9, 10]
package org.example;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import java.util.List;
import static org.junit.jupiter.api.Assertions.*;

class MovieTheaterTest {

    private MovieTheater theater;

    @BeforeEach
    void setUp() {
        theater = new MovieTheater(10);
    }

    @Test
    void testGetAvailableSeats() {
        List<Integer> availableSeats = theater.getAvailableSeats();
        assertEquals(10, availableSeats.size());
        for (int i = 1; i <= 10; i++) {
            assertTrue(availableSeats.contains(i));
        }
    }

    @Test
    void testBookSeat() {
        assertTrue(theater.bookSeat(1));
        assertFalse(theater.bookSeat(1));
        assertFalse(theater.bookSeat(0));
        assertFalse(theater.bookSeat(11));
    }

    @Test
    void testCancelBooking() {
        theater.bookSeat(1);
        assertTrue(theater.cancelBooking(1));
        assertFalse(theater.cancelBooking(1));
        assertFalse(theater.cancelBooking(0));
        assertFalse(theater.cancelBooking(11));
    }

    @Test
    void testConcurrentBooking() throws InterruptedException {
        Thread t1 = new Thread(() -> theater.bookSeat(1));
        Thread t2 = new Thread(() -> theater.bookSeat(1));
        t1.start();
        t2.start();
        t1.join();
        t2.join();

        List<Integer> availableSeats = theater.getAvailableSeats();
        assertEquals(9, availableSeats.size());
        assertFalse(availableSeats.contains(1));
    }
}

class BookingSystemTest {

    private BookingSystem bookingSystem;

    @BeforeEach
    void setUp() {
        bookingSystem = new BookingSystem(10);
    }

    @Test
    void testMakeBooking() {
        assertTrue(bookingSystem.makeBooking(1));
        assertFalse(bookingSystem.makeBooking(1));
    }

    @Test
    void testCancelBooking() {
        bookingSystem.makeBooking(1);
        assertTrue(bookingSystem.cancelBooking(1));
        assertFalse(bookingSystem.cancelBooking(1));
    }

    @Test
    void testPayForBooking() {
        bookingSystem.makeBooking(1);
        assertTrue(bookingSystem.payForBooking(1));
        assertFalse(bookingSystem.payForBooking(1));
    }

    @Test
    void testGetAvailableSeats() {
        List<Integer> availableSeats = bookingSystem.getAvailableSeats();
        assertEquals(10, availableSeats.size());
        bookingSystem.makeBooking(1);
        availableSeats = bookingSystem.getAvailableSeats();
        assertEquals(9, availableSeats.size());
        assertFalse(availableSeats.contains(1));
    }

    @Test
    void testConcurrentOperations() throws InterruptedException {
        Thread t1 = new Thread(() -> bookingSystem.makeBooking(1));
        Thread t2 = new Thread(() -> bookingSystem.makeBooking(1));
        Thread t3 = new Thread(() -> bookingSystem.cancelBooking(2));

        t1.start();
        t2.start();
        t3.start();

        t1.join();
        t2.join();
        t3.join();

        List<Integer> availableSeats = bookingSystem.getAvailableSeats();
        assertEquals(9, availableSeats.size());
        assertFalse(availableSeats.contains(1));
    }
}
//! Typed, bounded event queues for gameplay code.
//!
//! These queues are ordinary owned data: keep them in host-owned systems or in
//! an individual script. They do not register global resources and are not a
//! cross-library message bus. Events are consumed explicitly, never by the clock.
//!
//! ```
//! use kvas_engine::events::EventQueue;
//! let mut events = EventQueue::new(2);
//! assert!(events.push("door opened").is_ok());
//! assert_eq!(events.drain().collect::<Vec<_>>(), vec!["door opened"]);
//! assert!(events.is_empty());
//! ```
use std::collections::VecDeque;

/// FIFO queue with an explicit capacity limit and no silent event loss.
#[derive(Clone, Debug)]
pub struct EventQueue<T> {
    events: VecDeque<T>,
    capacity: usize,
}

impl<T> Default for EventQueue<T> {
    /// Creates a queue allowing up to 256 events.
    fn default() -> Self {
        Self::new(256)
    }
}

impl<T> EventQueue<T> {
    /// Creates an empty bounded queue. A zero limit rejects every event.
    /// Memory is allocated on demand, not up front for the full limit.
    pub fn new(capacity: usize) -> Self {
        Self { events: VecDeque::new(), capacity }
    }
    /// Pushes to the back or returns the original event if the queue is full.
    pub fn push(&mut self, event: T) -> Result<(), T> {
        if self.events.len() >= self.capacity {
            return Err(event);
        }
        self.events.push_back(event);
        Ok(())
    }
    /// Pops the oldest pending event, if any.
    pub fn pop(&mut self) -> Option<T> {
        self.events.pop_front()
    }
    /// Borrows the oldest pending event without consuming it.
    pub fn front(&self) -> Option<&T> {
        self.events.front()
    }
    /// Borrows all pending events in insertion order.
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &T> + ExactSizeIterator {
        self.events.iter()
    }
    /// Consumes all queued events in insertion order.
    /// Dropping the drain early also removes the remaining events.
    pub fn drain(&mut self) -> std::collections::vec_deque::Drain<'_, T> {
        self.events.drain(..)
    }
    /// Discards every event, retaining allocated memory.
    pub fn clear(&mut self) {
        self.events.clear();
    }
    /// Current pending count.
    pub fn len(&self) -> usize {
        self.events.len()
    }
    /// Whether the queue contains no pending events.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
    /// Whether another push would fail.
    pub fn is_full(&self) -> bool {
        self.len() >= self.capacity
    }
    /// Maximum pending count, independent of allocated storage size.
    pub fn capacity(&self) -> usize {
        self.capacity
    }
    /// Free logical slots before reaching the limit.
    pub fn remaining_capacity(&self) -> usize {
        self.capacity - self.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fifo_overflow_returns_ownership_and_drain_clears() {
        let mut q = EventQueue::new(2);
        q.push(String::from("a")).unwrap();
        q.push(String::from("b")).unwrap();
        assert_eq!(q.push(String::from("c")), Err(String::from("c")));
        assert!(q.is_full());
        assert_eq!(q.pop().as_deref(), Some("a"));
        q.push(String::from("c")).unwrap();
        assert_eq!(q.iter().map(String::as_str).collect::<Vec<_>>(), vec!["b", "c"]);
        {
            let mut drain = q.drain();
            assert_eq!(drain.next().as_deref(), Some("b"));
        }
        assert!(q.is_empty());
        assert_eq!(q.remaining_capacity(), 2);
        assert_eq!(EventQueue::new(0).push(7), Err(7));
    }
}

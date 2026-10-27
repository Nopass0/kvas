//! Bounded next-frame messages shared by native gameplay scripts.
//!
//! Sending writes to a pending buffer. [`SignalBus::begin_frame`] publishes that
//! buffer before script callbacks, so delivery is independent of script ordering.
//! Every receiver may read the same event; reads do not consume messages. Process
//! one-shot reactions in `Script::update`, since fixed steps share the same inbox.
//! Signals are runtime-only and are omitted from scene JSON/snapshots.
//!
//! ```
//! use kvas_engine::signals::SignalBus;
//! let mut bus = SignalBus::with_capacity(16);
//! bus.send("door.open", serde_json::json!({"key": 3}), None, None).unwrap();
//! assert!(bus.incoming().is_empty());
//! bus.begin_frame();
//! assert_eq!(bus.channel("door.open").count(), 1);
//! bus.begin_frame();
//! assert!(bus.incoming().is_empty());
//! ```

use kvas_ecs::Entity;
use serde_json::Value;

/// One immutable delivered message, with a sequence unique within its bus.
#[derive(Clone, Debug, PartialEq)]
pub struct Signal {
    /// Monotonic sequence starting at one; not a persistent/network identifier.
    pub sequence: u64,
    /// Case-sensitive routing channel, such as `door.open` or `damage`.
    pub channel: String,
    /// Optional sender identity. It can be stale by delivery time.
    pub sender: Option<Entity>,
    /// Optional recipient; `None` broadcasts to every interested script.
    pub target: Option<Entity>,
    /// JSON data crossing the host/library boundary without user-defined vtables.
    pub payload: Value,
}

/// A message was rejected without changing either buffer or consuming a sequence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SignalError {
    /// Routing channels must contain at least one non-whitespace character.
    EmptyChannel,
    /// The pending-buffer limit was reached; retry or deliberately drop the message.
    Full,
    /// All u64 sequence values have been used within this bus.
    SequenceExhausted,
    /// A script attempted to address an entity that is no longer alive.
    TargetNotAlive,
}

impl std::fmt::Display for SignalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::EmptyChannel => "signal channel is empty",
            Self::Full => "signal pending buffer is full",
            Self::SequenceExhausted => "signal sequence exhausted",
            Self::TargetNotAlive => "signal target is not alive",
        })
    }
}
impl std::error::Error for SignalError {}

/// Double-buffered message service owned by a [`crate::scene::Scene`].
/// Each buffer is limited independently, so at most twice `capacity()` events
/// are retained. Payload sizes are caller-owned; no background/global state exists.
#[derive(Clone, Debug)]
pub struct SignalBus {
    incoming: Vec<Signal>,
    pending: Vec<Signal>,
    capacity: usize,
    sequence: u64,
}

impl Default for SignalBus {
    fn default() -> Self {
        Self::with_capacity(256)
    }
}

impl SignalBus {
    /// Creates an empty bus. A zero capacity deliberately rejects all sends.
    pub fn with_capacity(capacity: usize) -> Self {
        Self { incoming: Vec::new(), pending: Vec::new(), capacity, sequence: 0 }
    }
    /// Enqueues a broadcast or directed signal for the next simulation frame.
    /// Returns its sequence, or a typed rejection without modifying the bus.
    /// The host may address any handle; script helpers validate recipient liveness.
    pub fn send(&mut self, channel: &str, payload: Value, sender: Option<Entity>, target: Option<Entity>) -> Result<u64, SignalError> {
        if channel.trim().is_empty() {
            return Err(SignalError::EmptyChannel);
        }
        if self.pending.len() >= self.capacity {
            return Err(SignalError::Full);
        }
        let sequence = self.sequence.checked_add(1).ok_or(SignalError::SequenceExhausted)?;
        self.pending.push(Signal { sequence, channel: channel.to_owned(), sender, target, payload });
        self.sequence = sequence;
        Ok(sequence)
    }
    /// Expires the previous inbox and publishes all pending messages, preserving order.
    /// Runtime calls this once before script sync/update, including during single-step.
    pub fn begin_frame(&mut self) {
        self.incoming.clear();
        std::mem::swap(&mut self.incoming, &mut self.pending);
    }
    /// Current delivered snapshot, valid until the next mutable bus operation.
    pub fn incoming(&self) -> &[Signal] {
        &self.incoming
    }
    /// Messages on one exact channel, in send order, without consuming them.
    pub fn channel<'a>(&'a self, channel: &'a str) -> impl Iterator<Item = &'a Signal> {
        self.incoming.iter().filter(move |signal| signal.channel == channel)
    }
    /// Delivered messages visible to one recipient (broadcast plus exact directed target).
    pub fn for_entity(&self, entity: Entity) -> impl Iterator<Item = &Signal> {
        self.incoming.iter().filter(move |signal| signal.target.is_none_or(|target| target == entity))
    }
    /// Pending messages to be delivered at the next frame boundary.
    pub fn pending_len(&self) -> usize {
        self.pending.len()
    }
    /// Maximum event count in each buffer.
    pub fn capacity(&self) -> usize {
        self.capacity
    }
    /// Discards both inbox and pending messages, preserving the sequence and allocations.
    pub fn clear(&mut self) {
        self.incoming.clear();
        self.pending.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn buffering_limits_and_entity_generations_are_respected() {
        let mut world = kvas_ecs::World::new();
        let old = world.spawn();
        let mut bus = SignalBus::with_capacity(2);
        assert_eq!(bus.send("", Value::Null, None, None), Err(SignalError::EmptyChannel));
        assert_eq!(bus.send("a", Value::Null, None, Some(old)), Ok(1));
        assert_eq!(bus.send("a", Value::Null, None, None), Ok(2));
        assert_eq!(bus.send("a", Value::Null, None, None), Err(SignalError::Full));
        assert!(bus.incoming().is_empty());
        bus.begin_frame();
        assert_eq!(bus.for_entity(old).count(), 2);
        world.despawn(old);
        let new = world.spawn();
        assert_eq!(old.index(), new.index());
        assert_eq!(bus.for_entity(new).count(), 1);
        assert_eq!(bus.send("a", Value::Null, None, None), Ok(3));
        bus.begin_frame();
        assert_eq!(bus.incoming().len(), 1);
        assert_eq!(bus.incoming()[0].sequence, 3);
        bus.clear();
        assert_eq!(bus.pending_len(), 0);
        assert!(bus.incoming().is_empty());
    }
}

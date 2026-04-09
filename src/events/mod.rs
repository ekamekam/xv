//! Event system — decouples game logic from UI rendering.
//!
//! Components that produce interesting changes (the memory reader, the state
//! machine, feature toggles) emit [`GameEvent`]s.  Components that want to
//! react (UI overlays, loggers) subscribe to specific event types via
//! [`EventDispatcher`].

use crate::features::Feature;
use crate::state::AppState;

// ── GameEvent ─────────────────────────────────────────────────────────────────

/// Events emitted by the xv game-logic layer.
#[derive(Debug, Clone)]
pub enum GameEvent {
    /// A new enemy player entered detection range.
    PlayerDetected(u64),
    /// An enemy player left detection range or is no longer readable.
    PlayerLost(u64),
    /// A feature was enabled or disabled.
    FeatureToggled { feature: Feature, enabled: bool },
    /// The application state machine changed.
    StateChanged { from: AppState, to: AppState },
    /// A non-fatal error occurred (e.g. a single failed memory read).
    Error(String),
}

// ── EventDispatcher ───────────────────────────────────────────────────────────

/// A unique handle returned when subscribing, used to unsubscribe.
pub type SubscriberId = usize;

type Callback = Box<dyn FnMut(&GameEvent) + Send + 'static>;

/// Publish/subscribe event dispatcher.
///
/// Subscribers register a callback and receive it whenever a matching event is
/// emitted.  Each subscriber receives **all** events; filtering by variant is
/// the subscriber's responsibility.
pub struct EventDispatcher {
    next_id: SubscriberId,
    subscribers: Vec<(SubscriberId, Callback)>,
}

impl EventDispatcher {
    /// Creates a new, empty dispatcher.
    pub fn new() -> Self {
        Self { next_id: 0, subscribers: Vec::new() }
    }

    /// Registers `callback` to be called for every future event.
    ///
    /// Returns a [`SubscriberId`] that can be passed to [`unsubscribe`] to
    /// remove the listener.
    pub fn subscribe<F>(&mut self, callback: F) -> SubscriberId
    where
        F: FnMut(&GameEvent) + Send + 'static,
    {
        let id = self.next_id;
        self.next_id += 1;
        self.subscribers.push((id, Box::new(callback)));
        id
    }

    /// Removes the subscriber with the given `id`.
    ///
    /// Has no effect if the id is not registered.
    pub fn unsubscribe(&mut self, id: SubscriberId) {
        self.subscribers.retain(|(sid, _)| *sid != id);
    }

    /// Emits `event` to all currently registered subscribers.
    pub fn emit(&mut self, event: GameEvent) {
        for (_, callback) in &mut self.subscribers {
            callback(&event);
        }
    }

    /// Removes all subscribers.
    pub fn clear_subscribers(&mut self) {
        self.subscribers.clear();
    }

    /// Returns the number of currently registered subscribers.
    pub fn subscriber_count(&self) -> usize {
        self.subscribers.len()
    }
}

impl Default for EventDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn emit_reaches_subscriber() {
        let mut dispatcher = EventDispatcher::new();
        let received: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let received_clone = Arc::clone(&received);

        dispatcher.subscribe(move |event| {
            if let GameEvent::Error(msg) = event {
                received_clone.lock().unwrap().push(msg.clone());
            }
        });

        dispatcher.emit(GameEvent::Error("test error".to_owned()));
        assert_eq!(received.lock().unwrap().as_slice(), &["test error"]);
    }

    #[test]
    fn multiple_subscribers_all_receive_event() {
        let mut dispatcher = EventDispatcher::new();
        let count = Arc::new(Mutex::new(0u32));

        for _ in 0..3 {
            let count_clone = Arc::clone(&count);
            dispatcher.subscribe(move |_| {
                *count_clone.lock().unwrap() += 1;
            });
        }

        dispatcher.emit(GameEvent::PlayerDetected(42));
        assert_eq!(*count.lock().unwrap(), 3);
    }

    #[test]
    fn unsubscribe_stops_delivery() {
        let mut dispatcher = EventDispatcher::new();
        let count = Arc::new(Mutex::new(0u32));
        let count_clone = Arc::clone(&count);

        let id = dispatcher.subscribe(move |_| {
            *count_clone.lock().unwrap() += 1;
        });

        dispatcher.emit(GameEvent::PlayerDetected(1));
        assert_eq!(*count.lock().unwrap(), 1);

        dispatcher.unsubscribe(id);
        dispatcher.emit(GameEvent::PlayerDetected(2));
        assert_eq!(*count.lock().unwrap(), 1); // no new delivery
    }

    #[test]
    fn clear_subscribers_removes_all() {
        let mut dispatcher = EventDispatcher::new();
        dispatcher.subscribe(|_| {});
        dispatcher.subscribe(|_| {});
        assert_eq!(dispatcher.subscriber_count(), 2);
        dispatcher.clear_subscribers();
        assert_eq!(dispatcher.subscriber_count(), 0);
    }

    #[test]
    fn feature_toggled_event_carries_data() {
        let mut dispatcher = EventDispatcher::new();
        let saw: Arc<Mutex<Option<(Feature, bool)>>> = Arc::new(Mutex::new(None));
        let saw_clone = Arc::clone(&saw);

        dispatcher.subscribe(move |event| {
            if let GameEvent::FeatureToggled { feature, enabled } = event {
                *saw_clone.lock().unwrap() = Some((*feature, *enabled));
            }
        });

        dispatcher.emit(GameEvent::FeatureToggled { feature: Feature::Esp, enabled: true });
        let result = saw.lock().unwrap();
        assert_eq!(*result, Some((Feature::Esp, true)));
    }
}

//! Application/game state machine.
//!
//! [`AppState`] tracks the lifecycle of xv from startup through connection to
//! the game, active monitoring, and eventual disconnection or error.

use std::fmt;

/// Possible states of the xv application.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppState {
    /// Starting up — memory reader and config are being initialised.
    Initializing,
    /// The CS2 process was found and is accessible.
    Connected,
    /// The game loop is actively reading memory and dispatching events.
    Running,
    /// The game loop is temporarily suspended (e.g. user-requested pause).
    Paused,
    /// The CS2 process was lost (exited or crashed).
    Disconnected,
    /// A fatal error occurred.  The inner string describes the cause.
    Error(String),
}

impl fmt::Display for AppState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppState::Initializing   => write!(f, "Initializing"),
            AppState::Connected      => write!(f, "Connected"),
            AppState::Running        => write!(f, "Running"),
            AppState::Paused         => write!(f, "Paused"),
            AppState::Disconnected   => write!(f, "Disconnected"),
            AppState::Error(reason)  => write!(f, "Error({reason})"),
        }
    }
}

/// Manages [`AppState`] transitions with validation.
#[derive(Debug)]
pub struct StateMachine {
    current: AppState,
}

impl StateMachine {
    /// Creates a new state machine starting in [`AppState::Initializing`].
    pub fn new() -> Self {
        Self { current: AppState::Initializing }
    }

    /// Creates a state machine with a specific initial state.
    ///
    /// Intended for testing; prefer [`StateMachine::new`] in production code.
    pub fn with_state(state: AppState) -> Self {
        Self { current: state }
    }

    /// Returns the current state.
    pub fn state(&self) -> &AppState {
        &self.current
    }

    /// Returns `true` while the game loop should be running (i.e. the state is
    /// [`AppState::Running`]).
    pub fn is_running(&self) -> bool {
        self.current == AppState::Running
    }

    /// Returns `true` if the application is in a usable (non-terminal) state.
    pub fn is_active(&self) -> bool {
        !matches!(self.current, AppState::Disconnected | AppState::Error(_))
    }

    /// Attempts to transition to `new_state`.
    ///
    /// Returns `Ok(())` if the transition is valid, or `Err(reason)` if the
    /// transition is not allowed from the current state.
    pub fn transition(&mut self, new_state: AppState) -> Result<(), String> {
        let allowed = match (&self.current, &new_state) {
            // Forward flow
            (AppState::Initializing, AppState::Connected)    => true,
            (AppState::Initializing, AppState::Error(_))     => true,
            (AppState::Connected,    AppState::Running)      => true,
            (AppState::Connected,    AppState::Disconnected) => true,
            (AppState::Connected,    AppState::Error(_))     => true,
            (AppState::Running,      AppState::Paused)       => true,
            (AppState::Running,      AppState::Disconnected) => true,
            (AppState::Running,      AppState::Error(_))     => true,
            // Resume
            (AppState::Paused,       AppState::Running)      => true,
            (AppState::Paused,       AppState::Disconnected) => true,
            (AppState::Paused,       AppState::Error(_))     => true,
            // Re-connect after disconnect
            (AppState::Disconnected, AppState::Initializing) => true,
            // Any state → Error is always permitted
            (_,                      AppState::Error(_))     => true,
            _ => false,
        };

        if allowed {
            self.current = new_state;
            Ok(())
        } else {
            Err(format!("invalid transition: {} → {}", self.current, new_state))
        }
    }
}

impl Default for StateMachine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_initializing() {
        let sm = StateMachine::new();
        assert_eq!(sm.state(), &AppState::Initializing);
    }

    #[test]
    fn valid_forward_transitions() {
        let mut sm = StateMachine::new();
        sm.transition(AppState::Connected).unwrap();
        sm.transition(AppState::Running).unwrap();
        sm.transition(AppState::Paused).unwrap();
        sm.transition(AppState::Running).unwrap();
        sm.transition(AppState::Disconnected).unwrap();
    }

    #[test]
    fn invalid_transition_returns_err() {
        let mut sm = StateMachine::new();
        let result = sm.transition(AppState::Running);
        assert!(result.is_err());
    }

    #[test]
    fn is_running_only_in_running_state() {
        let mut sm = StateMachine::new();
        assert!(!sm.is_running());
        sm.transition(AppState::Connected).unwrap();
        sm.transition(AppState::Running).unwrap();
        assert!(sm.is_running());
        sm.transition(AppState::Paused).unwrap();
        assert!(!sm.is_running());
    }

    #[test]
    fn any_state_can_transition_to_error() {
        for start in [AppState::Initializing, AppState::Connected, AppState::Running, AppState::Paused] {
            let mut sm = StateMachine { current: start };
            sm.transition(AppState::Error("test".into())).unwrap();
        }
    }

    #[test]
    fn disconnected_can_reinitialise() {
        let mut sm = StateMachine { current: AppState::Disconnected };
        sm.transition(AppState::Initializing).unwrap();
    }
}

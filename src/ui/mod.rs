//! High-level UI/game-state orchestration.
//!
//! [`GameStateManager`] wraps the Phase 2 [`GameReader`] and adds the Phase 3
//! coordination layer: state tracking, feature flags, configuration, and event
//! dispatching.

pub mod traits;

use crate::config::ConfigManager;
use crate::data::Data;
use crate::events::{EventDispatcher, GameEvent};
use crate::features::{Feature, FeatureFlags};
use crate::process::{offsets::Offsets, Process, ProcessError};
use crate::reader::{GameReader, ReadError};
use crate::state::{AppState, StateMachine};

/// Error type for [`GameStateManager`] operations.
#[derive(Debug)]
pub enum ManagerError {
    /// The underlying process could not be opened or became inaccessible.
    Process(ProcessError),
    /// A memory read failed.
    Read(ReadError),
    /// An invalid state-machine transition was attempted.
    InvalidTransition(String),
}

impl std::fmt::Display for ManagerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ManagerError::Process(e)            => write!(f, "process error: {e}"),
            ManagerError::Read(e)               => write!(f, "read error: {e}"),
            ManagerError::InvalidTransition(s)  => write!(f, "invalid transition: {s}"),
        }
    }
}

impl std::error::Error for ManagerError {}

impl From<ProcessError> for ManagerError {
    fn from(e: ProcessError) -> Self { ManagerError::Process(e) }
}

impl From<ReadError> for ManagerError {
    fn from(e: ReadError) -> Self { ManagerError::Read(e) }
}

// ── GameStateManager ──────────────────────────────────────────────────────────

/// Orchestrates game-state reading, configuration, feature flags, and events.
///
/// This is the primary entry point for the Phase 3 integration layer.
///
/// # Typical usage
///
/// ```rust,ignore
/// let mut manager = GameStateManager::new(cs2_pid)?;
/// manager.transition(AppState::Connected)?;
/// manager.transition(AppState::Running)?;
///
/// loop {
///     manager.update()?;
///     let data = manager.get_data();
///     // render data …
/// }
/// ```
pub struct GameStateManager {
    reader:     GameReader,
    data:       Data,
    state:      StateMachine,
    config:     ConfigManager,
    features:   FeatureFlags,
    dispatcher: EventDispatcher,
}

impl GameStateManager {
    /// Opens the CS2 process with `pid` and initialises all subsystems.
    pub fn new(pid: u32) -> Result<Self, ManagerError> {
        let process = Process::open(pid)?;
        let offsets = Offsets::load();
        let reader = GameReader::new(process, offsets)?;

        Ok(Self {
            reader,
            data:       Data::default(),
            state:      StateMachine::new(),
            config:     ConfigManager::new(),
            features:   FeatureFlags::default(),
            dispatcher: EventDispatcher::new(),
        })
    }

    // ── State ─────────────────────────────────────────────────────────────────

    /// Returns the current [`AppState`].
    pub fn app_state(&self) -> &AppState {
        self.state.state()
    }

    /// Attempts to transition to `new_state`, emitting a
    /// [`GameEvent::StateChanged`] event on success.
    pub fn transition(&mut self, new_state: AppState) -> Result<(), ManagerError> {
        let from = self.state.state().clone();
        self.state.transition(new_state.clone())
            .map_err(ManagerError::InvalidTransition)?;
        self.dispatcher.emit(GameEvent::StateChanged { from, to: new_state });
        Ok(())
    }

    // ── Game data ─────────────────────────────────────────────────────────────

    /// Reads the latest game state into the internal snapshot.
    ///
    /// On process disconnection the state machine is automatically moved to
    /// [`AppState::Disconnected`] and the error is returned.
    pub fn update(&mut self) -> Result<(), ManagerError> {
        match self.reader.update_game_data(&mut self.data) {
            Ok(()) => Ok(()),
            Err(ReadError::NotInGame) => {
                self.data.in_game = false;
                Ok(())
            }
            Err(e @ ReadError::Memory(_)) => {
                // Process likely gone — transition to Disconnected if possible.
                if self.state.is_active() {
                    let _ = self.state.transition(AppState::Disconnected);
                    self.dispatcher.emit(GameEvent::Error(e.to_string()));
                }
                Err(ManagerError::Read(e))
            }
        }
    }

    /// Returns a reference to the most recently read game state snapshot.
    pub fn get_data(&self) -> &Data {
        &self.data
    }

    /// Returns `true` if the underlying process is still accessible.
    pub fn is_valid(&self) -> bool {
        self.state.is_active()
    }

    // ── Configuration ─────────────────────────────────────────────────────────

    /// Returns a reference to the configuration manager.
    pub fn config(&self) -> &ConfigManager {
        &self.config
    }

    /// Returns a mutable reference to the configuration manager.
    pub fn config_mut(&mut self) -> &mut ConfigManager {
        &mut self.config
    }

    // ── Feature flags ─────────────────────────────────────────────────────────

    /// Returns a reference to the current feature flags.
    pub fn features(&self) -> &FeatureFlags {
        &self.features
    }

    /// Enables or disables `feature`, emitting a
    /// [`GameEvent::FeatureToggled`] event.
    pub fn set_feature(&mut self, feature: Feature, enabled: bool) {
        self.features.set(feature, enabled);
        self.dispatcher.emit(GameEvent::FeatureToggled { feature, enabled });
    }

    // ── Events ────────────────────────────────────────────────────────────────

    /// Returns a mutable reference to the event dispatcher.
    ///
    /// Use this to subscribe to or unsubscribe from game events.
    pub fn dispatcher_mut(&mut self) -> &mut EventDispatcher {
        &mut self.dispatcher
    }
}

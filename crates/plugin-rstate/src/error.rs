use serde::{Serialize, ser::Serializer};

pub type Result<T> = std::result::Result<T, RstateError>;

/// Error type for the rstate plugin.
///
/// This enum represents all possible errors that can occur when using the rstate plugin.
#[derive(Debug, thiserror::Error)]
pub enum RstateError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[cfg(mobile)]
    #[error(transparent)]
    PluginInvoke(#[from] tauri::plugin::mobile::PluginInvokeError),

    /// Generic state-related error
    #[error("State error: {0}")]
    State(String),

    /// Error when emitting events
    #[error("Event emission error: {0}")]
    Emit(String),

    /// Error during serialization/deserialization
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Action handler not found for the given action kind
    #[error("Action not found: {0}")]
    ActionNotFound(String),

    /// Invalid or malformed payload
    #[error("Invalid payload: {0}")]
    InvalidPayload(String),

    /// Missing required payload for an action
    #[error("Missing payload for action: {0}")]
    MissingPayload(String),

    /// State manager was not registered
    #[error("State manager not registered")]
    NotRegistered,

    /// Mutex lock was poisoned
    #[error("Lock poisoned: {0}")]
    LockPoisoned(String),
}

impl RstateError {
    /// Create a state error with a message
    pub fn state(msg: impl Into<String>) -> Self {
        Self::State(msg.into())
    }

    /// Create an invalid payload error with a message
    pub fn invalid_payload(msg: impl Into<String>) -> Self {
        Self::InvalidPayload(msg.into())
    }

    /// Create a missing payload error for an action
    pub fn missing_payload(action: impl Into<String>) -> Self {
        Self::MissingPayload(action.into())
    }

    /// Create an action not found error
    pub fn action_not_found(action: impl Into<String>) -> Self {
        Self::ActionNotFound(action.into())
    }

    /// Create a serialization error
    pub fn serialization(msg: impl Into<String>) -> Self {
        Self::Serialization(msg.into())
    }
}

impl Serialize for RstateError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

// Backward compatibility alias (deprecated)
#[deprecated(since = "0.2.0", note = "Use RstateError instead")]
pub type Error = RstateError;

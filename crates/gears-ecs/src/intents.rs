use crossbeam::channel::{Receiver, Sender, unbounded};
use std::sync::Arc;

/// Represents different types of intents that can be sent.
#[derive(Debug, Clone)]
pub enum Intent {
    /// Player wants to shoot.
    Shoot { entity: u32 },
    /// Entity wants to interact with another entity.
    Interact { source: u32, target: u32 },
    /// Custom intent with string data.
    Custom { name: String, data: Vec<u8> },
}

/// Intent receiver that processes intents from the channel.
/// Wrapped in Arc to allow sharing across systems.
#[derive(Clone)]
pub struct IntentReceiver {
    /// Receiver channel for intents.
    receiver: Arc<Receiver<Intent>>,
}

impl IntentReceiver {
    /// Creates a new intent receiver with the given channel receiver.
    ///
    /// # Arguments
    ///
    /// * `receiver` - The channel receiver to wrap.
    ///
    /// # Returns
    ///
    /// A new [`IntentReceiver`] instance.
    pub fn new(receiver: Receiver<Intent>) -> Self {
        Self {
            receiver: Arc::new(receiver),
        }
    }

    /// Tries to receive an intent without blocking.
    ///
    /// # Returns
    ///
    /// The first available intent if any.
    pub fn try_recv(&self) -> Option<Intent> {
        self.receiver.try_recv().ok()
    }

    /// Receives all pending intents without blocking.
    ///
    /// # Returns
    ///
    /// A vector of all received intents.
    pub fn try_recv_all(&self) -> Vec<Intent> {
        let mut intents = Vec::new();
        while let Ok(intent) = self.receiver.try_recv() {
            intents.push(intent);
        }
        intents
    }

    /// Blocks until an intent is received.
    ///
    /// # Returns
    ///
    /// The received intent.
    pub fn recv(&self) -> Option<Intent> {
        self.receiver.recv().ok()
    }

    /// Returns an iterator over all pending intents.
    ///
    /// # Returns
    ///
    /// An iterator yielding intents.
    pub fn iter(&self) -> impl Iterator<Item = Intent> + '_ {
        self.receiver.try_iter()
    }
}

/// Intent sender that can send intents to the channel.
#[derive(Clone)]
pub struct IntentSender {
    /// Sender channel for intents.
    sender: Sender<Intent>,
}

impl IntentSender {
    /// Creates a new intent sender with the given channel sender.
    ///
    /// # Arguments
    ///
    /// * `sender` - The channel sender to wrap.
    ///
    /// # Returns
    ///
    /// A new [`IntentSender`] instance.
    pub fn new(sender: Sender<Intent>) -> Self {
        Self { sender }
    }

    /// Sends an intent through the channel.
    ///
    /// # Arguments
    ///
    /// * `intent` - The intent to send.
    ///
    /// # Returns
    ///
    /// `true` if the intent was sent successfully.
    pub fn send(&self, intent: Intent) -> bool {
        self.sender.send(intent).is_ok()
    }

    /// Sends a shoot intent for the given entity.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity that wants to shoot.
    ///
    /// # Returns
    ///
    /// `true` if the intent was sent successfully.
    pub fn send_shoot(&self, entity: u32) -> bool {
        self.send(Intent::Shoot { entity })
    }

    /// Sends an interact intent between two entities.
    ///
    /// # Arguments
    ///
    /// * `source` - The entity initiating the interaction.
    /// * `target` - The entity being interacted with.
    ///
    /// # Returns
    ///
    /// `true` if the intent was sent successfully.
    pub fn send_interact(&self, source: u32, target: u32) -> bool {
        self.send(Intent::Interact { source, target })
    }

    /// Sends a custom intent with string name and binary data.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the custom intent.
    ///
    /// * `data` - The binary data associated with the intent.
    ///
    /// # Returns
    ///
    /// `true` if the intent was sent successfully.
    pub fn send_custom(&self, name: String, data: Vec<u8>) -> bool {
        self.send(Intent::Custom { name, data })
    }
}

/// Creates a new intent channel pair (sender, receiver).
///
/// # Returns
///
/// A tuple containing the [`IntentSender`] and [`IntentReceiver`].
pub fn create_intent_channel() -> (IntentSender, IntentReceiver) {
    let (sender, receiver) = unbounded();
    (IntentSender::new(sender), IntentReceiver::new(receiver))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_channel() {
        let (sender, receiver) = create_intent_channel();
        assert!(sender.send_shoot(1));
        assert!(matches!(
            receiver.try_recv(),
            Some(Intent::Shoot { entity: 1 })
        ));
    }

    #[test]
    fn test_send_multiple_intents() {
        let (sender, receiver) = create_intent_channel();
        sender.send_shoot(1);
        sender.send_interact(3, 4);

        let intents = receiver.try_recv_all();
        assert_eq!(intents.len(), 3);
    }

    #[test]
    fn test_try_recv_empty() {
        let (_sender, receiver) = create_intent_channel();
        assert!(receiver.try_recv().is_none());
    }

    #[test]
    fn test_iter() {
        let (sender, receiver) = create_intent_channel();
        sender.send_shoot(1);
        sender.send_shoot(2);
        sender.send_shoot(3);

        let count = receiver.iter().count();
        assert_eq!(count, 3);
    }
}

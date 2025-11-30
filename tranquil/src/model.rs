use serde::{Deserialize, Deserializer};

enum InteractionResponse {
    Pong,
}

impl<'de> Deserialize<'de> for InteractionResponse {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(InteractionResponseVisitor)
    }
}

enum InteractionCallbackType {
    /// ACK a Ping
    Pong = 1,
    /// Respond to an interaction with a message
    ChannelMessageWithSource = 4,
    /// ACK an interaction and edit a response later, the user sees a loading state
    DeferredChannelMessageWithSource = 5,
    /// For components, ACK an interaction and edit the original message later; the user does not
    /// see a loading state.
    ///
    /// Only valid for component-based interactions
    DeferredUpdateMessage = 6,
    /// For components, edit the message the component was attached to
    ///
    /// Only valid for component-based interactions
    UpdateMessage = 7,
    /// Respond to an autocomplete interaction with suggested choices
    ApplicationCommandAutocompleteResult = 8,
    /// Respond to an interaction with a popup modal
    ///
    /// Not available for MODAL_SUBMIT and PING interactions.
    Modal = 9,
    /// Deprecated; respond to an interaction with an upgrade button, only available for apps with
    /// monetization enabled
    PremiumRequired = 10,
    /// Launch the Activity associated with the app. Only available for apps with Activities enabled
    LaunchActivity = 12,
}

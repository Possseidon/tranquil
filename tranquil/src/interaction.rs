use uuid::{NonNilUuid, Uuid};

pub mod command;
// pub mod component;
// pub mod modal;

struct CustomId(NonNilUuid);

impl CustomId {
    fn new() -> Self {
        Self(NonNilUuid::new(Uuid::new_v4()).expect("uuid should not be nil"))
    }
}

enum CustomIdAssignment {
    /// Generates a fresh `custom_id` when the bot is started.
    ///
    /// This makes any pre-existing interactions unusable. This is generally good, since the
    /// underlying logic might have changed breaking things in subtle (or not-so-subtle) ways.
    OnStart,
    /// Uses a fixed `custom_id` that persists across restarts of the bot.
    ///
    /// This allows pre-existing interactions to continue to work when the bot is restarted, but the
    /// `custom_id` should still be updated manually whenever it makes sense to prevent breaking old
    /// interactions from behaving in odd ways.
    Fixed(CustomId),
}

impl CustomIdAssignment {
    fn get(self) -> CustomId {
        match self {
            CustomIdAssignment::OnStart => CustomId::new(),
            CustomIdAssignment::Fixed(custom_id) => custom_id,
        }
    }
}

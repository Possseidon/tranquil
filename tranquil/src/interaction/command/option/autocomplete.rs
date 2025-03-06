use std::mem::take;

use serenity::all::CommandDataOptionValue;

use super::{CommandOption, Resolve, ResolveRequired, Unvalidated};
use crate::interaction::error::{DiscordError, Result};

pub enum NotAutocompletable {}

impl ResolveRequired for NotAutocompletable {
    fn resolve(_data: &mut CommandDataOptionValue) -> Result<Self> {
        Err(DiscordError::NotAutocompletable.into())
    }
}

/// Used for autocompelted options during autocompletion.
///
/// Types that are not autocompleted can just use `Option<T::Unvalidated>`, since they can never be
/// focused.
pub enum Autocompleted<T: Unvalidated> {
    /// Unfocused options are reported as unvalidated strings, integers or numbers.
    Unfocused(Option<T::Unvalidated>),
    /// The focused option is always reported as a string.
    Focused(String),
}

impl<T: Unvalidated<Unvalidated: CommandOption>> Resolve for Autocompleted<T> {
    fn resolve(data: Option<&mut CommandDataOptionValue>) -> Result<Self> {
        let Some(data) = data else {
            return Ok(Self::Unfocused(None));
        };

        let CommandDataOptionValue::Autocomplete { kind, value } = data else {
            return Ok(Self::Unfocused(Some(
                <T::Unvalidated as ResolveRequired>::resolve(data)?,
            )));
        };

        if *kind == T::Unvalidated::KIND {
            Ok(Self::Focused(take(value)))
        } else {
            Err(T::Unvalidated::unexpected_type(data).into())
        }
    }
}

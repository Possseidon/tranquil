use serenity::{
    builder::CreateAutocompleteResponse, client::Context,
    model::application::interaction::autocomplete::AutocompleteInteraction,
};

pub struct AutocompleteCtx {
    pub bot: Context,
    pub interaction: AutocompleteInteraction,
}

impl AutocompleteCtx {
    pub async fn autocomplete<F>(self, f: F) -> serenity::Result<()>
    where
        F: FnOnce(&mut CreateAutocompleteResponse) -> &mut CreateAutocompleteResponse,
    {
        self.interaction
            .create_autocomplete_response(&self.bot, f)
            .await
    }
}

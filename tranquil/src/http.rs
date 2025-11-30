use std::{fmt::Display, iter::once};

use reqwest::{
    Client, Method, RequestBuilder,
    header::{AUTHORIZATION, HeaderMap, HeaderValue, InvalidHeaderValue},
};

use crate::display::display;

pub struct Http {
    client: Client,
}

impl Http {
    pub const DISCORD_API_VERSION: usize = 10;

    pub const USER_AGENT: &str = concat!(
        "DiscordBot (https://github.com/Possseidon/tranquil, ",
        env!("CARGO_PKG_VERSION"),
        ")"
    );

    pub fn new(authorization: Authorization) -> reqwest::Result<Self> {
        Ok(Self {
            client: Client::builder()
                .https_only(true)
                .user_agent(Self::USER_AGENT)
                .default_headers(once((AUTHORIZATION, authorization.header_value)).collect())
                .build()?,
        })
    }

    pub fn request(&self, method: Method, path: impl Display) -> RequestBuilder {
        self.client.request(
            method,
            format!(
                "https://discord.com/api/v{}/{path}",
                Self::DISCORD_API_VERSION
            ),
        )
    }
}

/// https://discord.com/developers/docs/interactions/receiving-and-responding#endpoints
impl Http {
    pub fn create_interaction_response(
        &self,
        interaction: Id<Interaction>,
        token: InteractionToken,
        response: InteractionResponse,
    ) -> impl Future<Output = reqwest::Result<()>> + use<> {
        self.request(
            Method::POST,
            display!("interactions/{interaction}/{token}/callback"),
        );
        // TODO: should return 204 without body
        todo!()
    }

    pub fn create_interaction_response_with_response(
        &self,
        interaction: Id<Interaction>,
        token: InteractionToken,
        response: InteractionResponse,
    ) -> impl Future<Output = reqwest::Result<InteractionCallbackObject>> + use<> {
        self.request(
            Method::POST,
            display!("interactions/{interaction}/{token}/callback"),
        )
        .query(&[("with_response", '1')]);
        // TODO: should return 200 with InteractionCallbackObject
        todo!()
    }

    pub async fn get_original_interaction_response() {
        // GET/webhooks/{application.id}/{interaction.token}/messages/@original
    }

    pub async fn edit_original_interaction_response() {
        // PATCH/webhooks/{application.id}/{interaction.token}/messages/@original
    }

    pub async fn delete_original_interaction_response() {
        // DELETE/webhooks/{application.id}/{interaction.token}/messages/@original
    }

    pub async fn create_followup_message() {
        // POST/webhooks/{application.id}/{interaction.token}
    }

    pub async fn get_followup_message() {
        // GET/webhooks/{application.id}/{interaction.token}/messages/{message.id}
    }

    pub async fn edit_followup_message() {
        // PATCH/webhooks/{application.id}/{interaction.token}/messages/{message.id}
    }

    pub async fn delete_followup_message() {
        // DELETE/webhooks/{application.id}/{interaction.token}/messages/{message.id}
    }
}

/// https://discord.com/developers/docs/interactions/application-commands#endpoints
impl Http {}

pub struct Authorization {
    header_value: HeaderValue,
}

impl Authorization {
    pub fn bearer(token: impl Display) -> Result<Self, InvalidHeaderValue> {
        let mut header_value = HeaderValue::from_str(&format!("Bearer: {token}"))?;
        header_value.set_sensitive(true);
        Ok(Self { header_value })
    }

    pub fn bot(token: impl Display) -> Result<Self, InvalidHeaderValue> {
        let mut header_value = HeaderValue::from_str(&format!("Bot: {token}"))?;
        header_value.set_sensitive(true);
        Ok(Self { header_value })
    }
}

/// Converts the given [`bool`] to a value that can be used in a query string.
fn query_bool(value: bool) -> char {
    if value { '1' } else { '0' }
}

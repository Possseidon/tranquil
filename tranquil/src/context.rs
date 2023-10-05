pub mod autocomplete;
pub mod command;
pub mod component;
pub mod modal;

use serenity::http::{CacheHttp, Http};

macro_rules! impl_http {
    ( $( $T:ty, )* ) => { $(
        impl AsRef<Http> for $T {
            fn as_ref(&self) -> &Http {
                self.bot.http()
            }
        }

        impl CacheHttp for $T {
            fn http(&self) -> &Http {
                self.bot.http()
            }
        }
    )* };
}

impl_http![
    autocomplete::AutocompleteCtx,
    command::CommandCtx,
    command::CommandCtxWithResponse,
    command::CommandCtxWithDeletedResponse,
    component::ComponentCtx,
    component::ComponentCtxWithResponse,
    component::ComponentCtxWithDeletedResponse,
    modal::ModalCtx,
    modal::ModalCtxWithResponse,
    modal::ModalCtxWithDeletedResponse,
];

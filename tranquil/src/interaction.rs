use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use uuid::Uuid;

use crate::{context::ComponentCtx, module::Module};

#[async_trait]
pub trait Interact: Serialize + DeserializeOwned {
    const UUID: Uuid;

    type Module: Module;

    async fn interact(self, module: &Self::Module, ctx: ComponentCtx) -> anyhow::Result<()>;
}

#[macro_export]
macro_rules! handle_interactions {
    [ $( $Interact:ty ),* $( , )? ] => {
        fn interaction_uuids(&self) -> &'static [$crate::uuid::Uuid] {
            &[ $( <$Interact>::UUID, )* ]
        }

        #[allow(unused_variables)]
        fn interact<'life0, 'life1, 'async_trait>(
            &'life0 self,
            uuid: $crate::uuid::Uuid,
            state: &'life1 str,
            ctx: $crate::context::ComponentCtx,
        ) -> ::std::pin::Pin<
            ::std::boxed::Box<
                dyn ::std::future::Future<Output = $crate::anyhow::Result<()>>
                    + ::std::marker::Send
                    + 'async_trait
            >
        >
        where
            'life0: 'async_trait,
            'life1: 'async_trait,
            Self: 'async_trait,
        {
            Box::pin(async move {
                match uuid {
                    $( <$Interact>::UUID => {
                        $crate::interaction::Interact::interact(
                            $crate::custom_id::custom_id_decode::<$Interact>(state)?,
                            self,
                            ctx,
                        ).await
                    } )*
                    _ => ::std::panic!("module does not handle interactions with uuid {uuid}"),
                }
            })
        }
    };
}

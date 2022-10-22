use serenity::{
    async_trait,
    builder::CreateApplicationCommandOption,
    model::{
        application::command::CommandOptionType,
        channel::{
            Channel, ChannelCategory, ChannelType, GuildChannel, PartialChannel, PrivateChannel,
        },
        id::ChannelId,
    },
};

use crate::l10n::L10n;

use super::{Resolve, ResolveContext, ResolveError, ResolveResult};

macro_rules! make_partial_channels {
    { $( $channel:ident => $channel_type:ident ),* $(,)? } => { $(
        #[derive(Clone, Debug)]
        pub struct $channel(pub PartialChannel);

        #[async_trait]
        impl Resolve for $channel {
            const KIND: CommandOptionType = <PartialChannel as Resolve>::KIND;

            fn describe(option: &mut CreateApplicationCommandOption, _l10n: &L10n) {
                option.channel_types(&[ChannelType::$channel_type]);
            }

            async fn resolve(ctx: ResolveContext) -> ResolveResult<Self> {
                Ok(Self(<PartialChannel as Resolve>::resolve(ctx).await?))
            }
        }
    )* };
}

make_partial_channels! {
    PartialTextChannel => Text,
    PartialPrivateChannel => Private,
    PartialVoiceChannel => Voice,
    PartialChannelCategory => Category,
    PartialNewsChannel => News,
    PartialNewsThreadChannel => NewsThread,
    PartialPublicThreadChannel => PublicThread,
    PartialPrivateThreadChannel => PrivateThread,
    PartialStageChannel => Stage,
    PartialDirectoryChannel => Directory,
    // PartialForumChannel => Forum,
}

#[async_trait]
impl Resolve for ChannelId {
    const KIND: CommandOptionType = <PartialChannel as Resolve>::KIND;

    fn describe(option: &mut CreateApplicationCommandOption, l10n: &L10n) {
        <PartialChannel as Resolve>::describe(option, l10n);
    }

    async fn resolve(ctx: ResolveContext) -> ResolveResult<Self> {
        Ok(<PartialChannel as Resolve>::resolve(ctx).await?.id)
    }
}

#[async_trait]
impl Resolve for Channel {
    const KIND: CommandOptionType = <ChannelId as Resolve>::KIND;

    fn describe(option: &mut CreateApplicationCommandOption, l10n: &L10n) {
        <ChannelId as Resolve>::describe(option, l10n);
    }

    async fn resolve(ctx: ResolveContext) -> ResolveResult<Self> {
        let http = ctx.http.clone();
        let id = <ChannelId as Resolve>::resolve(ctx).await?;
        Ok(http.get_channel(id.0).await?)
    }
}

#[async_trait]
impl Resolve for GuildChannel {
    const KIND: CommandOptionType = <Channel as Resolve>::KIND;

    fn describe(option: &mut CreateApplicationCommandOption, l10n: &L10n) {
        <Channel as Resolve>::describe(option, l10n);
        option.channel_types(&[
            ChannelType::Text,
            ChannelType::Voice,
            ChannelType::News,
            ChannelType::NewsThread,
            ChannelType::PublicThread,
            ChannelType::PrivateThread,
            ChannelType::Stage,
            ChannelType::Directory,
            // ChannelType::Forum,
        ]);
    }

    async fn resolve(ctx: ResolveContext) -> ResolveResult<Self> {
        match <Channel as Resolve>::resolve(ctx).await? {
            Channel::Guild(channel) => Ok(channel),
            _ => Err(ResolveError::InvalidChannelType),
        }
    }
}

#[async_trait]
impl Resolve for PrivateChannel {
    const KIND: CommandOptionType = <Channel as Resolve>::KIND;

    fn describe(option: &mut CreateApplicationCommandOption, l10n: &L10n) {
        <Channel as Resolve>::describe(option, l10n);
        option.channel_types(&[ChannelType::Private]);
    }

    async fn resolve(ctx: ResolveContext) -> ResolveResult<Self> {
        match <Channel as Resolve>::resolve(ctx).await? {
            Channel::Private(channel) => Ok(channel),
            _ => Err(ResolveError::InvalidChannelType),
        }
    }
}

#[async_trait]
impl Resolve for ChannelCategory {
    const KIND: CommandOptionType = <Channel as Resolve>::KIND;

    fn describe(option: &mut CreateApplicationCommandOption, l10n: &L10n) {
        <Channel as Resolve>::describe(option, l10n);
        option.channel_types(&[ChannelType::Category]);
    }

    async fn resolve(ctx: ResolveContext) -> ResolveResult<Self> {
        match <Channel as Resolve>::resolve(ctx).await? {
            Channel::Category(channel) => Ok(channel),
            _ => Err(ResolveError::InvalidChannelType),
        }
    }
}

macro_rules! make_channels {
    { $( $channel:ident => $channel_type:ident ),* $(,)? } => { $(
        #[derive(Clone, Debug)]
        pub struct $channel(pub GuildChannel);

        #[async_trait]
        impl Resolve for $channel {
            const KIND: CommandOptionType = <GuildChannel as Resolve>::KIND;

            fn describe(option: &mut CreateApplicationCommandOption, _l10n: &L10n) {
                option.channel_types(&[ChannelType::$channel_type]);
            }

            async fn resolve(ctx: ResolveContext) -> ResolveResult<Self> {
                Ok(Self(<GuildChannel as Resolve>::resolve(ctx).await?))
            }
        }
    )* };
}

make_channels! {
    TextChannel => Text,
    VoiceChannel => Voice,
    NewsChannel => News,
    NewsThreadChannel => NewsThread,
    PublicThreadChannel => PublicThread,
    PrivateThreadChannel => PrivateThread,
    StageChannel => Stage,
    DirectoryChannel => Directory,
    // ForumChannel => Forum,
}

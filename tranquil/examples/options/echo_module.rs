use bounded_integer::{
    BoundedI128, BoundedI16, BoundedI32, BoundedI64, BoundedI8, BoundedIsize, BoundedU128,
    BoundedU16, BoundedU32, BoundedU64, BoundedU8, BoundedUsize,
};
use serenity::model::{
    channel::{Attachment, Channel, ChannelCategory, GuildChannel, PartialChannel, PrivateChannel},
    guild::{PartialMember, Role},
    id::ChannelId,
    mention::Mention,
    user::User,
};
use tranquil::{
    bounded_number, bounded_string,
    command::CommandContext,
    l10n::CommandL10nProvider,
    macros::{command_provider, slash},
    module::Module,
    resolve::{
        Choices, DirectoryChannel, Mentionable, NewsChannel, NewsThreadChannel,
        PartialChannelCategory, PartialDirectoryChannel, PartialNewsChannel,
        PartialNewsThreadChannel, PartialPrivateChannel, PartialPrivateThreadChannel,
        PartialPublicThreadChannel, PartialStageChannel, PartialTextChannel, PartialVoiceChannel,
        PrivateThreadChannel, PublicThreadChannel, StageChannel, TextChannel, VoiceChannel,
    },
};

#[derive(Module, CommandL10nProvider)]
pub(crate) struct EchoModule;

#[command_provider]
impl EchoModule {
    #[slash]
    async fn echo_string(&self, ctx: CommandContext, value: String) -> anyhow::Result<()> {
        echo(ctx, value).await
    }

    #[slash]
    async fn echo_i64(&self, ctx: CommandContext, value: i64) -> anyhow::Result<()> {
        echo(ctx, value).await
    }

    #[slash]
    async fn echo_bool(&self, ctx: CommandContext, value: bool) -> anyhow::Result<()> {
        echo(ctx, value).await
    }

    #[slash(rename = "echo partial-channel any")]
    async fn echo_partial_channel_any(
        &self,
        ctx: CommandContext,
        value: PartialChannel,
    ) -> anyhow::Result<()> {
        echo(ctx, value).await
    }

    #[slash]
    async fn echo_role(&self, ctx: CommandContext, value: Role) -> anyhow::Result<()> {
        echo(ctx, value).await
    }

    #[slash]
    async fn echo_f64(&self, ctx: CommandContext, value: f64) -> anyhow::Result<()> {
        echo(ctx, value).await
    }

    #[slash]
    async fn echo_attachment(&self, ctx: CommandContext, value: Attachment) -> anyhow::Result<()> {
        echo(ctx, value).await
    }

    // --- channel

    #[slash(rename = "echo partial-channel text")]
    async fn echo_partial_channel_text(
        &self,
        ctx: CommandContext,
        value: PartialTextChannel,
    ) -> anyhow::Result<()> {
        echo(ctx, value).await
    }

    #[slash(rename = "echo partial-channel private")]
    async fn echo_partial_channel_private(
        &self,
        ctx: CommandContext,
        value: PartialPrivateChannel,
    ) -> anyhow::Result<()> {
        echo(ctx, value).await
    }

    #[slash(rename = "echo partial-channel voice")]
    async fn echo_partial_channel_voice(
        &self,
        ctx: CommandContext,
        value: PartialVoiceChannel,
    ) -> anyhow::Result<()> {
        echo(ctx, value).await
    }

    #[slash(rename = "echo partial-channel category")]
    async fn echo_partial_channel_category(
        &self,
        ctx: CommandContext,
        value: PartialChannelCategory,
    ) -> anyhow::Result<()> {
        echo(ctx, value).await
    }

    #[slash(rename = "echo partial-channel news")]
    async fn echo_partial_channel_news(
        &self,
        ctx: CommandContext,
        value: PartialNewsChannel,
    ) -> anyhow::Result<()> {
        echo(ctx, value).await
    }

    #[slash(rename = "echo partial-channel news-thread")]
    async fn echo_partial_channel_news_thread(
        &self,
        ctx: CommandContext,
        value: PartialNewsThreadChannel,
    ) -> anyhow::Result<()> {
        echo(ctx, value).await
    }

    #[slash(rename = "echo partial-channel public-thread")]
    async fn echo_partial_channel_public_thread(
        &self,
        ctx: CommandContext,
        value: PartialPublicThreadChannel,
    ) -> anyhow::Result<()> {
        echo(ctx, value).await
    }

    #[slash(rename = "echo partial-channel private-thread")]
    async fn echo_partial_channel_private_thread(
        &self,
        ctx: CommandContext,
        value: PartialPrivateThreadChannel,
    ) -> anyhow::Result<()> {
        echo(ctx, value).await
    }

    #[slash(rename = "echo partial-channel stage")]
    async fn echo_partial_channel_stage(
        &self,
        ctx: CommandContext,
        value: PartialStageChannel,
    ) -> anyhow::Result<()> {
        echo(ctx, value).await
    }

    #[slash(rename = "echo partial-channel directory")]
    async fn echo_partial_channel_directory(
        &self,
        ctx: CommandContext,
        value: PartialDirectoryChannel,
    ) -> anyhow::Result<()> {
        echo(ctx, value).await
    }

    // #[slash(rename = "echo partial-channel forum")]
    // async fn echo_partial_channel_forum(
    //     &self,
    //     ctx: CommandContext,
    //     value: PartialForumChannel,
    // ) -> anyhow::Result<()> {
    //     echo(ctx, value).await
    // }

    #[slash(rename = "echo channel-id")]
    async fn echo_channel_id(&self, ctx: CommandContext, value: ChannelId) -> anyhow::Result<()> {
        echo(ctx, value).await
    }

    #[slash]
    async fn echo_channel_any(&self, ctx: CommandContext, value: Channel) -> anyhow::Result<()> {
        echo(ctx, value).await
    }

    #[slash]
    async fn echo_channel_guild(
        &self,
        ctx: CommandContext,
        value: GuildChannel,
    ) -> anyhow::Result<()> {
        echo(ctx, value).await
    }

    #[slash]
    async fn echo_channel_private(
        &self,
        ctx: CommandContext,
        value: PrivateChannel,
    ) -> anyhow::Result<()> {
        echo(ctx, value).await
    }

    #[slash]
    async fn echo_channel_category(
        &self,
        ctx: CommandContext,
        value: ChannelCategory,
    ) -> anyhow::Result<()> {
        echo(ctx, value).await
    }

    #[slash]
    async fn echo_channel_text(
        &self,
        ctx: CommandContext,
        value: TextChannel,
    ) -> anyhow::Result<()> {
        echo(ctx, value).await
    }

    #[slash]
    async fn echo_channel_voice(
        &self,
        ctx: CommandContext,
        value: VoiceChannel,
    ) -> anyhow::Result<()> {
        echo(ctx, value).await
    }

    #[slash]
    async fn echo_channel_news(
        &self,
        ctx: CommandContext,
        value: NewsChannel,
    ) -> anyhow::Result<()> {
        echo(ctx, value).await
    }

    #[slash(rename = "echo channel news-thread")]
    async fn echo_channel_news_thread(
        &self,
        ctx: CommandContext,
        value: NewsThreadChannel,
    ) -> anyhow::Result<()> {
        echo(ctx, value).await
    }

    #[slash(rename = "echo channel public-thread")]
    async fn echo_channel_public_thread(
        &self,
        ctx: CommandContext,
        value: PublicThreadChannel,
    ) -> anyhow::Result<()> {
        echo(ctx, value).await
    }

    #[slash(rename = "echo channel private-thread")]
    async fn echo_channel_private_thread(
        &self,
        ctx: CommandContext,
        value: PrivateThreadChannel,
    ) -> anyhow::Result<()> {
        echo(ctx, value).await
    }

    #[slash]
    async fn echo_channel_stage(
        &self,
        ctx: CommandContext,
        value: StageChannel,
    ) -> anyhow::Result<()> {
        echo(ctx, value).await
    }

    #[slash]
    async fn echo_channel_directory(
        &self,
        ctx: CommandContext,
        value: DirectoryChannel,
    ) -> anyhow::Result<()> {
        echo(ctx, value).await
    }

    // #[slash]
    // async fn echo_channel_forum(
    //     &self,
    //     ctx: CommandContext,
    //     value: ForumChannel,
    // ) -> anyhow::Result<()> {
    //     echo(ctx, value).await
    // }

    // --- integer

    #[slash]
    async fn echo_integer_i8(&self, ctx: CommandContext, value: i8) -> anyhow::Result<()> {
        echo(ctx, value).await
    }

    #[slash]
    async fn echo_integer_i16(&self, ctx: CommandContext, value: i16) -> anyhow::Result<()> {
        echo(ctx, value).await
    }

    #[slash]
    async fn echo_integer_i32(&self, ctx: CommandContext, value: i32) -> anyhow::Result<()> {
        echo(ctx, value).await
    }

    #[slash]
    async fn echo_integer_i128(&self, ctx: CommandContext, value: i128) -> anyhow::Result<()> {
        echo(ctx, value).await
    }

    #[slash]
    async fn echo_integer_isize(&self, ctx: CommandContext, value: isize) -> anyhow::Result<()> {
        echo(ctx, value).await
    }

    #[slash]
    async fn echo_integer_u8(&self, ctx: CommandContext, value: u8) -> anyhow::Result<()> {
        echo(ctx, value).await
    }

    #[slash]
    async fn echo_integer_u16(&self, ctx: CommandContext, value: u16) -> anyhow::Result<()> {
        echo(ctx, value).await
    }

    #[slash]
    async fn echo_integer_u32(&self, ctx: CommandContext, value: u32) -> anyhow::Result<()> {
        echo(ctx, value).await
    }

    #[slash]
    async fn echo_integer_u64(&self, ctx: CommandContext, value: u64) -> anyhow::Result<()> {
        echo(ctx, value).await
    }

    #[slash]
    async fn echo_integer_u128(&self, ctx: CommandContext, value: u128) -> anyhow::Result<()> {
        echo(ctx, value).await
    }

    #[slash]
    async fn echo_integer_usize(&self, ctx: CommandContext, value: usize) -> anyhow::Result<()> {
        echo(ctx, value).await
    }

    #[slash(rename = "echo bounded-integer i8")]
    async fn echo_bounded_integer_i8(
        &self,
        ctx: CommandContext,
        value: BoundedI8<42, 69>,
    ) -> anyhow::Result<()> {
        echo(ctx, value).await
    }

    #[slash(rename = "echo bounded-integer i16")]
    async fn echo_bounded_integer_i16(
        &self,
        ctx: CommandContext,
        value: BoundedI16<42, 69>,
    ) -> anyhow::Result<()> {
        echo(ctx, value).await
    }

    #[slash(rename = "echo bounded-integer i32")]
    async fn echo_bounded_integer_i32(
        &self,
        ctx: CommandContext,
        value: BoundedI32<42, 69>,
    ) -> anyhow::Result<()> {
        echo(ctx, value).await
    }

    #[slash(rename = "echo bounded-integer i64")]
    async fn echo_bounded_integer_i64(
        &self,
        ctx: CommandContext,
        value: BoundedI64<42, 69>,
    ) -> anyhow::Result<()> {
        echo(ctx, value).await
    }

    #[slash(rename = "echo bounded-integer i128")]
    async fn echo_bounded_integer_i128(
        &self,
        ctx: CommandContext,
        value: BoundedI128<42, 69>,
    ) -> anyhow::Result<()> {
        echo(ctx, value).await
    }

    #[slash(rename = "echo bounded-integer isize")]
    async fn echo_bounded_integer_isize(
        &self,
        ctx: CommandContext,
        value: BoundedIsize<42, 69>,
    ) -> anyhow::Result<()> {
        echo(ctx, value).await
    }

    #[slash(rename = "echo bounded-integer u8")]
    async fn echo_bounded_integer_u8(
        &self,
        ctx: CommandContext,
        value: BoundedU8<42, 69>,
    ) -> anyhow::Result<()> {
        echo(ctx, value).await
    }

    #[slash(rename = "echo bounded-integer u16")]
    async fn echo_bounded_integer_u16(
        &self,
        ctx: CommandContext,
        value: BoundedU16<42, 69>,
    ) -> anyhow::Result<()> {
        echo(ctx, value).await
    }

    #[slash(rename = "echo bounded-integer u32")]
    async fn echo_bounded_integer_u32(
        &self,
        ctx: CommandContext,
        value: BoundedU32<42, 69>,
    ) -> anyhow::Result<()> {
        echo(ctx, value).await
    }

    #[slash(rename = "echo bounded-integer u64")]
    async fn echo_bounded_integer_u64(
        &self,
        ctx: CommandContext,
        value: BoundedU64<42, 69>,
    ) -> anyhow::Result<()> {
        echo(ctx, value).await
    }

    #[slash(rename = "echo bounded-integer u128")]
    async fn echo_bounded_integer_u128(
        &self,
        ctx: CommandContext,
        value: BoundedU128<42, 69>,
    ) -> anyhow::Result<()> {
        echo(ctx, value).await
    }

    #[slash(rename = "echo bounded-integer usize")]
    async fn echo_bounded_integer_usize(
        &self,
        ctx: CommandContext,
        value: BoundedUsize<42, 69>,
    ) -> anyhow::Result<()> {
        echo(ctx, value).await
    }

    // --- mentionable

    #[slash]
    async fn echo_mentionable(
        &self,
        ctx: CommandContext,
        value: Mentionable,
    ) -> anyhow::Result<()> {
        echo(ctx, value).await
    }

    #[slash]
    async fn echo_mention(&self, ctx: CommandContext, value: Mention) -> anyhow::Result<()> {
        echo(ctx, value).await
    }

    // --- number

    #[slash]
    async fn echo_f32(&self, ctx: CommandContext, value: f32) -> anyhow::Result<()> {
        echo(ctx, value).await
    }

    #[slash(rename = "echo bounded-number")]
    async fn echo_bounded_number(
        &self,
        ctx: CommandContext,
        value: NiceNumber,
    ) -> anyhow::Result<()> {
        echo(ctx, value).await
    }

    // --- option

    #[slash]
    async fn echo_option(&self, ctx: CommandContext, value: Option<String>) -> anyhow::Result<()> {
        echo(ctx, value).await
    }

    // --- string

    #[slash]
    async fn echo_choice(&self, ctx: CommandContext, value: Color) -> anyhow::Result<()> {
        echo(ctx, value).await
    }

    #[slash(rename = "echo bounded-string")]
    async fn echo_bounded_string(
        &self,
        ctx: CommandContext,
        value: NiceString,
    ) -> anyhow::Result<()> {
        echo(ctx, value).await
    }

    // --- user

    #[slash]
    async fn echo_user(&self, ctx: CommandContext, value: User) -> anyhow::Result<()> {
        echo(ctx, value).await
    }

    #[slash(rename = "echo partial-member")]
    async fn echo_partial_member(
        &self,
        ctx: CommandContext,
        value: PartialMember,
    ) -> anyhow::Result<()> {
        echo(ctx, value).await
    }
}

async fn echo(ctx: CommandContext, value: impl std::fmt::Debug) -> anyhow::Result<()> {
    ctx.create_interaction_response(|response| {
        response.interaction_response_data(|data| data.content(format!("```rust\n{value:#?}\n```")))
    })
    .await?;
    Ok(())
}

bounded_number!(NiceNumber: 42.069..=420.69);
bounded_string!(NiceString: 42..=69);

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Choices)]
enum Color {
    Red,
    Green,
    Blue,
}

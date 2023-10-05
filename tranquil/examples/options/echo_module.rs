use anyhow::Result;
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
    context::command::CommandCtx,
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

#[derive(Module)]
pub(crate) struct EchoModule;

#[command_provider]
impl EchoModule {
    #[slash]
    async fn echo_string(&self, ctx: CommandCtx, value: String) -> Result<()> {
        echo(ctx, value).await
    }

    #[slash]
    async fn echo_i64(&self, ctx: CommandCtx, value: i64) -> Result<()> {
        echo(ctx, value).await
    }

    #[slash]
    async fn echo_bool(&self, ctx: CommandCtx, value: bool) -> Result<()> {
        echo(ctx, value).await
    }

    #[slash(rename = "echo partial-channel any")]
    async fn echo_partial_channel_any(&self, ctx: CommandCtx, value: PartialChannel) -> Result<()> {
        echo(ctx, value).await
    }

    #[slash]
    async fn echo_role(&self, ctx: CommandCtx, value: Role) -> Result<()> {
        echo(ctx, value).await
    }

    #[slash]
    async fn echo_f64(&self, ctx: CommandCtx, value: f64) -> Result<()> {
        echo(ctx, value).await
    }

    #[slash]
    async fn echo_attachment(&self, ctx: CommandCtx, value: Attachment) -> Result<()> {
        echo(ctx, value).await
    }

    // --- channel

    #[slash(rename = "echo partial-channel text")]
    async fn echo_partial_channel_text(
        &self,
        ctx: CommandCtx,
        value: PartialTextChannel,
    ) -> Result<()> {
        echo(ctx, value).await
    }

    #[slash(rename = "echo partial-channel private")]
    async fn echo_partial_channel_private(
        &self,
        ctx: CommandCtx,
        value: PartialPrivateChannel,
    ) -> Result<()> {
        echo(ctx, value).await
    }

    #[slash(rename = "echo partial-channel voice")]
    async fn echo_partial_channel_voice(
        &self,
        ctx: CommandCtx,
        value: PartialVoiceChannel,
    ) -> Result<()> {
        echo(ctx, value).await
    }

    #[slash(rename = "echo partial-channel category")]
    async fn echo_partial_channel_category(
        &self,
        ctx: CommandCtx,
        value: PartialChannelCategory,
    ) -> Result<()> {
        echo(ctx, value).await
    }

    #[slash(rename = "echo partial-channel news")]
    async fn echo_partial_channel_news(
        &self,
        ctx: CommandCtx,
        value: PartialNewsChannel,
    ) -> Result<()> {
        echo(ctx, value).await
    }

    #[slash(rename = "echo partial-channel news-thread")]
    async fn echo_partial_channel_news_thread(
        &self,
        ctx: CommandCtx,
        value: PartialNewsThreadChannel,
    ) -> Result<()> {
        echo(ctx, value).await
    }

    #[slash(rename = "echo partial-channel public-thread")]
    async fn echo_partial_channel_public_thread(
        &self,
        ctx: CommandCtx,
        value: PartialPublicThreadChannel,
    ) -> Result<()> {
        echo(ctx, value).await
    }

    #[slash(rename = "echo partial-channel private-thread")]
    async fn echo_partial_channel_private_thread(
        &self,
        ctx: CommandCtx,
        value: PartialPrivateThreadChannel,
    ) -> Result<()> {
        echo(ctx, value).await
    }

    #[slash(rename = "echo partial-channel stage")]
    async fn echo_partial_channel_stage(
        &self,
        ctx: CommandCtx,
        value: PartialStageChannel,
    ) -> Result<()> {
        echo(ctx, value).await
    }

    #[slash(rename = "echo partial-channel directory")]
    async fn echo_partial_channel_directory(
        &self,
        ctx: CommandCtx,
        value: PartialDirectoryChannel,
    ) -> Result<()> {
        echo(ctx, value).await
    }

    // #[slash(rename = "echo partial-channel forum")]
    // async fn echo_partial_channel_forum(
    //     &self,
    //     ctx: CommandCtx,
    //     value: PartialForumChannel,
    // ) -> Result<()> {
    //     echo(ctx, value).await
    // }

    #[slash(rename = "echo channel-id")]
    async fn echo_channel_id(&self, ctx: CommandCtx, value: ChannelId) -> Result<()> {
        echo(ctx, value).await
    }

    #[slash]
    async fn echo_channel_any(&self, ctx: CommandCtx, value: Channel) -> Result<()> {
        echo(ctx, value).await
    }

    #[slash]
    async fn echo_channel_guild(&self, ctx: CommandCtx, value: GuildChannel) -> Result<()> {
        echo(ctx, value).await
    }

    #[slash]
    async fn echo_channel_private(&self, ctx: CommandCtx, value: PrivateChannel) -> Result<()> {
        echo(ctx, value).await
    }

    #[slash]
    async fn echo_channel_category(&self, ctx: CommandCtx, value: ChannelCategory) -> Result<()> {
        echo(ctx, value).await
    }

    #[slash]
    async fn echo_channel_text(&self, ctx: CommandCtx, value: TextChannel) -> Result<()> {
        echo(ctx, value).await
    }

    #[slash]
    async fn echo_channel_voice(&self, ctx: CommandCtx, value: VoiceChannel) -> Result<()> {
        echo(ctx, value).await
    }

    #[slash]
    async fn echo_channel_news(&self, ctx: CommandCtx, value: NewsChannel) -> Result<()> {
        echo(ctx, value).await
    }

    #[slash(rename = "echo channel news-thread")]
    async fn echo_channel_news_thread(
        &self,
        ctx: CommandCtx,
        value: NewsThreadChannel,
    ) -> Result<()> {
        echo(ctx, value).await
    }

    #[slash(rename = "echo channel public-thread")]
    async fn echo_channel_public_thread(
        &self,
        ctx: CommandCtx,
        value: PublicThreadChannel,
    ) -> Result<()> {
        echo(ctx, value).await
    }

    #[slash(rename = "echo channel private-thread")]
    async fn echo_channel_private_thread(
        &self,
        ctx: CommandCtx,
        value: PrivateThreadChannel,
    ) -> Result<()> {
        echo(ctx, value).await
    }

    #[slash]
    async fn echo_channel_stage(&self, ctx: CommandCtx, value: StageChannel) -> Result<()> {
        echo(ctx, value).await
    }

    #[slash]
    async fn echo_channel_directory(&self, ctx: CommandCtx, value: DirectoryChannel) -> Result<()> {
        echo(ctx, value).await
    }

    // #[slash]
    // async fn echo_channel_forum(
    //     &self,
    //     ctx: CommandCtx,
    //     value: ForumChannel,
    // ) -> Result<()> {
    //     echo(ctx, value).await
    // }

    // --- integer

    #[slash]
    async fn echo_integer_i8(&self, ctx: CommandCtx, value: i8) -> Result<()> {
        echo(ctx, value).await
    }

    #[slash]
    async fn echo_integer_i16(&self, ctx: CommandCtx, value: i16) -> Result<()> {
        echo(ctx, value).await
    }

    #[slash]
    async fn echo_integer_i32(&self, ctx: CommandCtx, value: i32) -> Result<()> {
        echo(ctx, value).await
    }

    #[slash]
    async fn echo_integer_i128(&self, ctx: CommandCtx, value: i128) -> Result<()> {
        echo(ctx, value).await
    }

    #[slash]
    async fn echo_integer_isize(&self, ctx: CommandCtx, value: isize) -> Result<()> {
        echo(ctx, value).await
    }

    #[slash]
    async fn echo_integer_u8(&self, ctx: CommandCtx, value: u8) -> Result<()> {
        echo(ctx, value).await
    }

    #[slash]
    async fn echo_integer_u16(&self, ctx: CommandCtx, value: u16) -> Result<()> {
        echo(ctx, value).await
    }

    #[slash]
    async fn echo_integer_u32(&self, ctx: CommandCtx, value: u32) -> Result<()> {
        echo(ctx, value).await
    }

    #[slash]
    async fn echo_integer_u64(&self, ctx: CommandCtx, value: u64) -> Result<()> {
        echo(ctx, value).await
    }

    #[slash]
    async fn echo_integer_u128(&self, ctx: CommandCtx, value: u128) -> Result<()> {
        echo(ctx, value).await
    }

    #[slash]
    async fn echo_integer_usize(&self, ctx: CommandCtx, value: usize) -> Result<()> {
        echo(ctx, value).await
    }

    #[slash(rename = "echo bounded-integer i8")]
    async fn echo_bounded_integer_i8(
        &self,
        ctx: CommandCtx,
        value: BoundedI8<42, 69>,
    ) -> Result<()> {
        echo(ctx, value).await
    }

    #[slash(rename = "echo bounded-integer i16")]
    async fn echo_bounded_integer_i16(
        &self,
        ctx: CommandCtx,
        value: BoundedI16<42, 69>,
    ) -> Result<()> {
        echo(ctx, value).await
    }

    #[slash(rename = "echo bounded-integer i32")]
    async fn echo_bounded_integer_i32(
        &self,
        ctx: CommandCtx,
        value: BoundedI32<42, 69>,
    ) -> Result<()> {
        echo(ctx, value).await
    }

    #[slash(rename = "echo bounded-integer i64")]
    async fn echo_bounded_integer_i64(
        &self,
        ctx: CommandCtx,
        value: BoundedI64<42, 69>,
    ) -> Result<()> {
        echo(ctx, value).await
    }

    #[slash(rename = "echo bounded-integer i128")]
    async fn echo_bounded_integer_i128(
        &self,
        ctx: CommandCtx,
        value: BoundedI128<42, 69>,
    ) -> Result<()> {
        echo(ctx, value).await
    }

    #[slash(rename = "echo bounded-integer isize")]
    async fn echo_bounded_integer_isize(
        &self,
        ctx: CommandCtx,
        value: BoundedIsize<42, 69>,
    ) -> Result<()> {
        echo(ctx, value).await
    }

    #[slash(rename = "echo bounded-integer u8")]
    async fn echo_bounded_integer_u8(
        &self,
        ctx: CommandCtx,
        value: BoundedU8<42, 69>,
    ) -> Result<()> {
        echo(ctx, value).await
    }

    #[slash(rename = "echo bounded-integer u16")]
    async fn echo_bounded_integer_u16(
        &self,
        ctx: CommandCtx,
        value: BoundedU16<42, 69>,
    ) -> Result<()> {
        echo(ctx, value).await
    }

    #[slash(rename = "echo bounded-integer u32")]
    async fn echo_bounded_integer_u32(
        &self,
        ctx: CommandCtx,
        value: BoundedU32<42, 69>,
    ) -> Result<()> {
        echo(ctx, value).await
    }

    #[slash(rename = "echo bounded-integer u64")]
    async fn echo_bounded_integer_u64(
        &self,
        ctx: CommandCtx,
        value: BoundedU64<42, 69>,
    ) -> Result<()> {
        echo(ctx, value).await
    }

    #[slash(rename = "echo bounded-integer u128")]
    async fn echo_bounded_integer_u128(
        &self,
        ctx: CommandCtx,
        value: BoundedU128<42, 69>,
    ) -> Result<()> {
        echo(ctx, value).await
    }

    #[slash(rename = "echo bounded-integer usize")]
    async fn echo_bounded_integer_usize(
        &self,
        ctx: CommandCtx,
        value: BoundedUsize<42, 69>,
    ) -> Result<()> {
        echo(ctx, value).await
    }

    // --- mentionable

    #[slash]
    async fn echo_mentionable(&self, ctx: CommandCtx, value: Mentionable) -> Result<()> {
        echo(ctx, value).await
    }

    #[slash]
    async fn echo_mention(&self, ctx: CommandCtx, value: Mention) -> Result<()> {
        echo(ctx, value).await
    }

    // --- number

    #[slash]
    async fn echo_f32(&self, ctx: CommandCtx, value: f32) -> Result<()> {
        echo(ctx, value).await
    }

    #[slash(rename = "echo bounded-number")]
    async fn echo_bounded_number(&self, ctx: CommandCtx, value: NiceNumber) -> Result<()> {
        echo(ctx, value).await
    }

    // --- option

    #[slash]
    async fn echo_option(&self, ctx: CommandCtx, value: Option<String>) -> Result<()> {
        echo(ctx, value).await
    }

    // --- string

    #[slash]
    async fn echo_choice(&self, ctx: CommandCtx, value: Color) -> Result<()> {
        echo(ctx, value).await
    }

    #[slash(rename = "echo bounded-string")]
    async fn echo_bounded_string(&self, ctx: CommandCtx, value: NiceString) -> Result<()> {
        echo(ctx, value).await
    }

    // --- user

    #[slash]
    async fn echo_user(&self, ctx: CommandCtx, value: User) -> Result<()> {
        echo(ctx, value).await
    }

    #[slash(rename = "echo partial-member")]
    async fn echo_partial_member(&self, ctx: CommandCtx, value: PartialMember) -> Result<()> {
        echo(ctx, value).await
    }
}

async fn echo(ctx: CommandCtx, value: impl std::fmt::Debug) -> Result<()> {
    ctx.respond(|response| {
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

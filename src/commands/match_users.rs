use crate::{Context, Error, database, types::Phase, utilities::ensure_host_role};
use poise::CreateReply;
use rusqlite::Result;
use serenity::all::User;

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn match_users(
    ctx: Context<'_>,
    #[description = "User 1"] santa: User,
    #[description = "User 2"] giftee: User,
) -> Result<(), Error> {
    if !ensure_host_role(&ctx, ctx.author()).await? {
        return Ok(());
    }
    if !crate::utilities::ensure_correct_phase(&ctx, vec![Phase::Swap, Phase::Watch]).await? {
        return Ok(());
    }

    let check1 = database::check_if_matched(santa.id.get()).await?;
    let check2 = database::check_if_matched(giftee.id.get()).await?;
    if check1 || check2 {
        ctx.send(CreateReply::default()
        .content("Either the santa already has claimed a letter, or the gifee already has had their letter claimed")
        .ephemeral(true)).await?;
        return Ok(());
    }

    match database::set_match(santa.id.get(), giftee.id.get()).await {
        Ok(_) => {
            let message = format!("Successfully set {} as {}'s Santa", santa.name, giftee.name);
            ctx.send(CreateReply::default().content(message).ephemeral(true))
                .await?;
        }
        Err(e) => {
            ctx.send(
                CreateReply::default()
                    .content("Error setting Santa")
                    .ephemeral(true),
            )
            .await?;
        }
    }
    Ok(())
}

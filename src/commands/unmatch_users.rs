use crate::{Context, Error, database, types::Phase, utilities::ensure_host_role};
use poise::CreateReply;
use rusqlite::Result;
use serenity::all::User;

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn unmatch_users(
    ctx: Context<'_>,
    #[description = "User"] santa: User,
) -> Result<(), Error> {
    if !ensure_host_role(&ctx, ctx.author()).await? {
        return Ok(());
    }
    if !crate::utilities::ensure_correct_phase(&ctx, vec![Phase::Join, Phase::Watch, Phase::Swap])
        .await?
    {
        return Ok(());
    }

    let check1 = database::check_if_matched(santa.id.get()).await?;
    if !check1 {
        ctx.send(
            CreateReply::default()
                .content("This user was not matched")
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    }

    match database::remove_match(santa.id.get()).await {
        Ok(_) => {
            let message = format!("Successfully removed the match of {}", santa.name);
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
            eprintln!("Unmatching error: {}", e);
        }
    }
    Ok(())
}

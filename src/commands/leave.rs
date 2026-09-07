use crate::{Context, Error, database, types::Phase, utilities::{ensure_correct_phase, ensure_joined}};
use poise::CreateReply;
use rusqlite::Result;

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn leave(
    ctx: Context<'_>,
) -> Result<(), Error> {
    if !ensure_joined(&ctx).await? {return Ok(())}
    if !ensure_correct_phase(&ctx, vec![Phase::Join]).await? {return Ok(())}

    let user_id = ctx.author().id.get();
    match database::leave(user_id).await {
        Ok(_) => {
            ctx.send(CreateReply::default().content("Succesfully left the event").ephemeral(true)).await?;
        },
        Err(e) => {
            ctx.send(CreateReply::default().content("Error leaving the event").ephemeral(true)).await?;
            eprintln!("Error deleting user: {}", e);
        }
    }
    Ok(())
}
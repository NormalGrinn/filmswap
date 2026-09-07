use crate::{Context, Error, database, types::Phase, utilities::ensure_host_role};
use poise::CreateReply;
use rusqlite::Result;
use serenity::all::User;

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn unban_user(
    ctx: Context<'_>,
    #[description = "User"]
    user: User,
) -> Result<(), Error> {
    if !ensure_host_role(&ctx, ctx.author()).await? {return Ok(())}
    
    let res = database::unban_user(user.id.get());
    match res {
        Ok(_) => {
            let message = format!("Unbanned user: {}", user.name);
            ctx.send(CreateReply::default()
                .content(message)
                .ephemeral(true)).await?;
        },
        Err(e) => {
            let message = format!("Error unbanning user: {}", user.name);
            ctx.send(CreateReply::default()
                .content(message)
                .ephemeral(true)).await?;
                eprintln!("Error unbanning user: {} {}", user.name, e);
        },
    }

    Ok(())
}
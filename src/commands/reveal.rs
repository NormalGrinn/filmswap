use crate::{
    Context, Error, database, types::Phase, utilities::ensure_host_role,
};
use poise::CreateReply;
use rusqlite::Result;
use serenity::all::CreateMessage;

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn reveal(
    ctx: Context<'_>,
) -> Result<(), Error> {
    if !ensure_host_role(&ctx, ctx.author()).await? {
        return Ok(());
    }
    if !crate::utilities::ensure_correct_phase(&ctx, vec![Phase::Swap, Phase::Watch]).await? {return Ok(())}

    let users = match database::get_matching_order() {
        Ok(users) => users,
        Err(e) => {
            eprintln!("Error getting matching order: {}", e);

            ctx.send(
                CreateReply::default()
                    .content("Error getting matching order.")
                    .ephemeral(true),
            )
            .await?;

            return Ok(());
        }
    };

    let mut reveal_message = users
        .iter()
        .map(|(_, username)| format!("`{}`", username))
        .collect::<Vec<String>>()
        .join(" -> ");

    if let Some((_, first_username)) = users.first() {
        reveal_message.push_str(&format!(" -> `{}`", first_username));
    }

    let message = format!(
        "The current matching order is:\n\n{}",
        reveal_message
    );

    match ctx.author().direct_message(
        ctx.http(),
        CreateMessage::new().content(message),
    ).await {
        Ok(_) => {
            ctx.send(
                CreateReply::default()
                    .content("The matching order has been sent to your DMs.")
                    .ephemeral(true),
            )
            .await?;
        }
        Err(e) => {
            eprintln!("Error sending reveal DM: {}", e);

            ctx.send(
                CreateReply::default()
                    .content("I couldn't send you a DM. Please make sure your DMs are open.")
                    .ephemeral(true),
            )
            .await?;
        }
    }

    Ok(())
}
use crate::{
    Context,
    Error,
    database,
    types::UserInfo,
    utilities::ensure_host_role,
};
use poise::CreateReply;
use rusqlite::Result;
use serenity::all::{CreateAttachment, CreateMessage};
use tokio::fs;

const PATH: &str = "status.txt";

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn status(
    ctx: Context<'_>,
) -> Result<(), Error> {
    if !ensure_host_role(&ctx, ctx.author()).await? {
        return Ok(());
    }

    let users: Vec<UserInfo> = match database::get_all_users().await {
        Ok(users) => users,
        Err(e) => {
            ctx.send(
                CreateReply::default()
                    .content("Error fetching users")
                    .ephemeral(true),
            )
            .await?;

            eprintln!("Error fetching users: {}", e);
            return Ok(());
        }
    };

    // Users who are banned.
    let banned: Vec<&UserInfo> = users
        .iter()
        .filter(|user| user.is_banned)
        .collect();

    // Users who have joined but have not written a letter.
    let joined_no_letter: Vec<&UserInfo> = users
        .iter()
        .filter(|user| user.letter.is_none() && user.has_joined)
        .collect();

    // Users who have joined and have written a letter.
    let joined: Vec<&UserInfo> = users
        .iter()
        .filter(|user| user.letter.is_some() && user.has_joined)
        .collect();

    let mut buffer = String::new();

    buffer.push_str("=== BANNED USERS ===\n");

    for user in banned {
        buffer.push_str(&format!(
            "{}, {}\n",
            user.username,
            user.discord_id
        ));
    }

    buffer.push_str("\n=== JOINED USERS, LETTER NOT WRITTEN ===\n");

    for user in joined_no_letter {
        buffer.push_str(&format!(
            "{}, {}\n",
            user.username,
            user.discord_id
        ));
    }

    buffer.push_str("\n=== JOINED USERS WITH LETTER ===\n");

    for user in joined {
        buffer.push_str(&format!(
            "{}, {}\n",
            user.username,
            user.discord_id
        ));
    }

    fs::write(PATH, buffer).await?;

    let attachment = CreateAttachment::path(PATH).await?;
    let builder = CreateMessage::new().add_file(attachment);

    match ctx.author().direct_message(ctx.http(), builder).await {
        Ok(_) => {
            ctx.send(
                CreateReply::default()
                    .content("Successfully sent status")
                    .ephemeral(true),
            )
            .await?;
        }
        Err(e) => {
            eprintln!("Error sending status: {}", e);

            ctx.send(
                CreateReply::default()
                    .content("Error sending status")
                    .ephemeral(true),
            )
            .await?;
        }
    }

    // Ignore error if the file doesn't exist.
    let _ = fs::remove_file(PATH).await;

    Ok(())
}
use crate::{utilities::ensure_host_role, Context, Error};
use poise::CreateReply;
use rusqlite::Result;
use serenity::all::{CreateActionRow, CreateButton};

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn create_button(
    ctx: Context<'_>,
) -> Result<(), Error> {
    if !ensure_host_role(&ctx, ctx.author()).await? {return Ok(())}
    let join_button = CreateButton::new("Join")
        .label("Join Film Swap")
        .style(serenity::all::ButtonStyle::Primary);
    let buttons: Vec<CreateButton> = vec![join_button];
    let action_row = vec![CreateActionRow::Buttons(buttons)];
    let message = CreateReply::default().components(action_row);
    ctx.send(message).await?;
    Ok(())
}
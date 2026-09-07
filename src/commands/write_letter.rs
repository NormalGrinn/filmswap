use crate::{Context, Error, database, types::Phase, utilities::{ensure_dm, ensure_embed_field_lenght, ensure_joined, reject_if_already_running, wait_for_message_with_cancel}};
use rusqlite::Result;

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn write_letter(
    ctx: Context<'_>,
) -> Result<(), Error> {
    reject_if_already_running(&ctx, || async {
        if !ensure_joined(&ctx).await? { return Ok(()); }
        if !ensure_dm(&ctx).await? { return Ok(()); }
        if !crate::utilities::ensure_correct_phase(&ctx, vec![Phase::Join]).await? {return Ok(())}

        let prompt = "Send your letter or press cancel to abort the action.";
        match wait_for_message_with_cancel(&ctx, prompt).await? {
            Some(msg) => {
                if !ensure_embed_field_lenght(&ctx, &msg, 2000).await? {
                    return Ok(());
                }
                database::set_letter(ctx.author().id.get(), &msg).await?;
                ctx.say("Letter updated!").await?;
            },
            None => {()},
        }
        Ok(())
    }).await
}

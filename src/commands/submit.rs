use crate::{
    Context, Error, database::{get_giftee, set_submission}, types::Phase, utilities::{
        self, ensure_dm, ensure_embed_field_lenght, ensure_has_giftee, ensure_joined, reject_if_already_running, wait_for_message_with_cancel
    },
};
use rusqlite::Result;
use serenity::all::{CreateMessage, UserId};

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn submit(ctx: Context<'_>) -> Result<(), Error> {
    reject_if_already_running(&ctx, || async {
        if !ensure_joined(&ctx).await? { return Ok(()); }
        if !ensure_dm(&ctx).await? { return Ok(()); }
        if !ensure_has_giftee(&ctx).await? { return Ok(()); }
        if !crate::utilities::ensure_correct_phase(&ctx, vec![Phase::Swap, Phase::Watch]).await? {return Ok(())}

        match wait_for_message_with_cancel(
            &ctx,
            "Send what you want to submit, otherwise press cancel to cancel the action",
        )
        .await?
        {
            Some(message) => {
                if !ensure_embed_field_lenght(&ctx, &message, 2000).await? {
                    return Ok(());
                }

                match set_submission(ctx.author().id.get(), &message).await {
                    Ok(_) => {
                        ctx.say("Submission successfully set").await?;
                    }
                    Err(e) => {
                        eprintln!("Error setting submission: {}", e);
                        ctx.say("Error with setting your submission").await?;
                    }
                }
            }
            None => {()}
        }

        Ok(())
    }).await

}

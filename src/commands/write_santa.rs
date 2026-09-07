use crate::{
    Context, Error, database, types::Phase, utilities::{
        self, ensure_dm, ensure_embed_field_lenght, ensure_has_santa, ensure_joined, reject_if_already_running, wait_for_message_with_cancel
    },
};
use rusqlite::Result;
use serenity::all::{CreateMessage, UserId};

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn write_santa(ctx: Context<'_>) -> Result<(), Error> {
    reject_if_already_running(&ctx, || async {
        if !ensure_joined(&ctx).await? { return Ok(()); }
        if !ensure_dm(&ctx).await? { return Ok(()); }
        if !ensure_has_santa(&ctx).await? { return Ok(()); }
        if !crate::utilities::ensure_correct_phase(&ctx, vec![Phase::Swap, Phase::Watch]).await? {return Ok(())}

        let prompt_message = "
        Press the cancel button to cancel the action, otherwise send a message to send something to your Santa (the person giving you something).
        ";

        let user_id = ctx.author().id.get();

        match wait_for_message_with_cancel(&ctx, prompt_message).await? {
            Some(message) => {
                if !ensure_embed_field_lenght(&ctx, &message, 2000).await? { return Ok(()); }

                match database::get_santa(user_id).await {
                    Ok(santa) => {
                        let santa_id = UserId::new(santa);
                        let embed = utilities::embed_builder(
                            &message,
                            "Your Giftee sent you a message",
                            "Dear Santa",
                            &format!("Love, {}", ctx.author().name),
                        );

                        let santa_message = CreateMessage::new().embed(embed);

                        let _ = match santa_id.dm(&ctx.http(), santa_message).await {
                            Ok(_) => ctx.say("Message sent successfully to your Santa").await?,
                            Err(e) => {
                                eprintln!("Error sending message to Santa: {}", e);
                                ctx.say("An error occurred sending your message").await?
                            }
                        };
                    }
                    Err(e) => {
                        ctx.say("An error occurred getting your Santa").await?;
                        eprintln!("Error fetching Santa: {}", e);
                    }
                }
            }
            None => {()}
        }

        Ok(())
    }).await
}

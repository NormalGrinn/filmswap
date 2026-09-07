use crate::{Context, Error, database::{get_santa, get_userinfo_by_id}, types::Phase, utilities::{self, ensure_dm, ensure_has_santa, ensure_joined}};
use poise::CreateReply;
use rusqlite::Result;

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn receive(
    ctx: Context<'_>,
) -> Result<(), Error> {
    if !ensure_joined(&ctx).await? {return Ok(())}
    if !ensure_dm(&ctx).await? {return Ok(())}
    if !ensure_has_santa(&ctx).await? {return Ok(())}
    if !crate::utilities::ensure_correct_phase(&ctx, vec![Phase::Watch]).await? {return Ok(())}

    let author_id = ctx.author().id.get();

    let santa = get_santa(author_id).await?;
    let potentail_gift = get_userinfo_by_id(santa).await?;
    match potentail_gift.submission {
        Some(gift_message) => {
            let embed = utilities::embed_builder(&gift_message, 
                "Your Santa has given you a gift", 
                &format!("Dear {}", ctx.author().name), 
                "Love, Santa");
            let giftee_message = CreateReply::default().embed(embed);
            ctx.send(giftee_message).await?;
        },
        None => {ctx.say("Your Santa has not submitted a gift yet").await?;},
    }
    Ok(())
}
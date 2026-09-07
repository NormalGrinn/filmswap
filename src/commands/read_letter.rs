use crate::{database, utilities::{embed_builder, ensure_dm, ensure_joined}, Context, Error};
use poise::CreateReply;
use rusqlite::Result;

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn read_letter(
    ctx: Context<'_>,
) -> Result<(), Error> {
    if !ensure_joined(&ctx).await? {return Ok(())}
    if !ensure_dm(&ctx).await? {return Ok(())}

    let user_id = ctx.author().id.get();
    match database::get_userinfo_by_id(user_id).await {
        Ok(user_info) => {
            let letter = user_info.letter;
            match letter {
                Some(l) => {
                    let giftee_name=  ctx.author().name.clone();
                    let embed = embed_builder(
                        &l, 
                        "What your Santa will see:", 
                        "Dear Santa", 
                        &format!("Love, {}", giftee_name));
                    let reply = CreateReply::default().embed(embed);
                    ctx.send(reply).await?;
                },
                None => {
                    ctx.say("You have not set your letter yet").await?;
                },
            }
        },
        Err(e) => {
            ctx.say("Error getting letter").await?;
            eprintln!("Error getting letter: {}", e);
        },
    }
    Ok(())
}
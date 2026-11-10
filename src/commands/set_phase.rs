use crate::database;
use crate::types::Phase::{self, Join, Swap, Watch};
use crate::{Context, Error, utilities::ensure_host_role};
use poise::CreateReply;

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn set_phase(
    ctx: Context<'_>,
    #[description = "The phase you want to set it to"] phase: Phase,
) -> Result<(), Error> {
    if !ensure_host_role(&ctx, ctx.author()).await? {
        return Ok(());
    };

    let current_phase = database::get_phase()?;
    if current_phase == phase {
        let message = format!("You are already in this phase!");
        ctx.send(CreateReply::default().content(message).ephemeral(true))
            .await?;
        return Ok(());
    }

    match phase {
        Phase::Join => {
            if current_phase == Swap {
                let message = format!("You cannot change from the swap to the join phase.");
                ctx.send(CreateReply::default().content(message).ephemeral(true))
                    .await?;
            } else {
                let res = database::reset_giftees_and_submissions();
                match res {
                    Ok(_) => {
                        database::set_phase(phase)?;
                        let message = format!("Succesfully changed phase from watch to join.");
                        ctx.send(CreateReply::default().content(message).ephemeral(true))
                            .await?;
                    }
                    Err(e) => {
                        let message =
                            format!("Error changing from watch to join and resetting gifts.");
                        ctx.send(CreateReply::default().content(message).ephemeral(true))
                            .await?;
                        eprintln!("Error changing phases and ressetting gifts: {}", e);
                    }
                }
            }
        }
        Phase::Swap => {
            if current_phase == Watch {
                let message = format!("You cannot change from the watch to the swap phase.");
                ctx.send(CreateReply::default().content(message).ephemeral(true))
                    .await?;
            } else {
                let res = database::match_users();
                match res {
                    Ok(_) => {
                        database::set_phase(phase)?;
                        let message = format!("Succesfully changed phase from watch to swap.");
                        ctx.send(CreateReply::default().content(message).ephemeral(true))
                            .await?;
                    }
                    Err(e) => {
                        let message =
                            format!("Error changing from watch to join and matching users.");
                        ctx.send(CreateReply::default().content(message).ephemeral(true))
                            .await?;
                        eprintln!("Error changing phases and matching users: {}", e);
                    }
                }
            }
        }
        Phase::Watch => {
            if current_phase == Join {
                let message = format!("You cannot change from the join to the watch phase.");
                ctx.send(CreateReply::default().content(message).ephemeral(true))
                    .await?;
            } else {
                database::set_phase(phase)?;
                let message = format!("Changed phase to watch");
                ctx.send(CreateReply::default().content(message).ephemeral(true))
                    .await?;
            }
        }
    }
    Ok(())
}

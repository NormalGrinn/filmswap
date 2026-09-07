use std::{env, ops::Deref};

use serenity::all::{CreateInteractionResponse, CreateInteractionResponseFollowup, CreateInteractionResponseMessage, CreateMessage, FullEvent, Interaction};

use crate::{Data, Error, database, types::Phase, utilities::ensure_correct_phase};

pub async fn on_component_interaction(
    ctx: &serenity::all::Context,
    event: &FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    _data: &Data,
) -> Result<(), Error> {
    match event {
        FullEvent::InteractionCreate { interaction } => {
            match interaction {
                Interaction::Component(component_interaction) => {
                    if component_interaction.data.custom_id == "Join" {
                        let user_id = component_interaction.user.id.get();
                        let response_date = CreateInteractionResponseMessage::new().ephemeral(true);
                        let interaction_response = CreateInteractionResponse::Defer(response_date);
                        let phase = database::get_phase()?;


                        if phase != Phase::Join {
                            let message = CreateInteractionResponseFollowup::new()
                            .content("You cannot join in this phase!").ephemeral(true);
                            component_interaction.create_followup(&ctx.http, message).await?;
                            return Ok(())
                        }
                        

                        let is_banned = database::is_user_banned(user_id)?;
                        if is_banned {
                            component_interaction
                                .create_response(
                                    &ctx.http,
                                    CreateInteractionResponse::Message(
                                        CreateInteractionResponseMessage::new()
                                            .content("You are currently banned and thus cannot join.")
                                            .ephemeral(true),
                                    ),
                                )
                                .await?;

                            return Ok(());
                        }


                        component_interaction.create_response(&ctx, interaction_response).await?;
                        let info: Result<_, _> = database::get_userinfo_by_id(user_id).await;
                        match info {
                            Ok(user) => {
                                if user.has_joined {
                                    let message = CreateInteractionResponseFollowup::new()
                                        .content("You are already in the event!").ephemeral(true);
                                    component_interaction.create_followup(&ctx.http, message).await?;
                                } else {
                                    let res = database::rejoin_user(user.discord_id);
                                    match res {
                                        Ok(_) => {
                                            let message = CreateInteractionResponseFollowup::new()
                                                .content("Successfully joined the event!").ephemeral(true);
                                             component_interaction.create_followup(&ctx.http, message).await?;
                                        },
                                        Err(_) => {
                                            let message = CreateInteractionResponseFollowup::new()
                                                .content("Error joining the event").ephemeral(true);
                                            component_interaction.create_followup(&ctx.http, message).await?;
                                        },
                                    }
                                }
                            },
                            Err(rusqlite::Error::QueryReturnedNoRows) => {
                                database::create_user(&component_interaction.user.name, user_id).await?;
                                let message = CreateInteractionResponseFollowup::new()
                                    .content("Joined the event!").ephemeral(true);
                                component_interaction.create_followup(&ctx.http, message).await?;
                                let join_dm = format!("You have successfully joined the event!");
                                let message = CreateMessage::default().content(join_dm);
                                component_interaction.user.dm(&ctx.http, message).await?;
                                }
                            _ => {
                                let message = CreateInteractionResponseFollowup::new()
                                .content("Unexpected error").ephemeral(true);
                                component_interaction.create_followup(&ctx.http, message).await?;
                            }
                        }
                        Ok(())
                    } else {
                        Ok(())
                    }
                },
                _ => Ok(()),
            }
        },
        _ => Ok(()),
    }

}
use std::env;
use poise::serenity_prelude as serenity;
use poise::CreateReply;
use poise::futures_util::{future::select, StreamExt, FutureExt};
use ::serenity::all::ButtonStyle;
use ::serenity::all::ComponentInteractionCollector;
use ::serenity::all::CreateActionRow;
use ::serenity::all::CreateButton;
use ::serenity::all::CreateInteractionResponse;
use ::serenity::all::CreateInteractionResponseMessage;
use ::serenity::all::MessageCollector;

use crate::Context;
use crate::Error;
use crate::database;
use crate::database::get_userinfo_by_id;
use crate::types::Phase;


pub async fn ensure_dm(ctx: &Context<'_>) -> Result<bool, serenity::Error> {
    let dm_channel = ctx.author().create_dm_channel(&ctx.serenity_context().http).await?;
    let channel_id = dm_channel.id;

    if ctx.channel_id() != channel_id {
        ctx.send(
            CreateReply::default()
                .content("This command is only allowed to be used in DMs")
                .ephemeral(true),
        )
        .await?;
        return Ok(false);
    }

    Ok(true)
}

pub async fn ensure_joined(ctx: &Context<'_>) -> Result<bool, serenity::Error> {
    let user_id = ctx.author().id.get();
    match get_userinfo_by_id(user_id).await {
        Ok(_) => Ok(true),
        Err(_) => {
            ctx.send(
                CreateReply::default()
                    .content("You have not joined the event yet")
                    .ephemeral(true),
            )
            .await?;
            return Ok(false);
        },
    }
}

pub async fn ensure_embed_field_lenght(ctx: &Context<'_>, message: &str, lenght: usize) -> Result<bool, serenity::Error> {
    let chars: Vec<char> = message.chars().collect();
    if chars.len() > lenght {
        let reply = format!("Your message is over {} characters", lenght);
        ctx.send(
            CreateReply::default()
                .content(reply)
                .ephemeral(true),
        )
        .await?;
        return Ok(false);
    }
    Ok(true)
}

pub fn embed_builder(message: &str, title: &str, hello_message: &str, goodbye_message: &str) -> serenity::CreateEmbed {
    use serenity::builder::{CreateEmbed, CreateEmbedFooter};
    use serenity::model::colour::Colour;

    let footer = CreateEmbedFooter::new("Film Swap");
    let mut embed = CreateEmbed::new()
        .footer(footer)
        .color(Colour::MAGENTA)
        .title(title);

    let chunks = message
        .as_bytes()
        .chunks(1024)
        .map(|chunk| String::from_utf8_lossy(chunk).to_string())
        .collect::<Vec<_>>();

    for (i, chunk) in chunks.iter().enumerate() {
        let name = if i == 0 { hello_message } else { "\u{200B}" };
        embed = embed.field(name, chunk, false);
    }

    embed = embed.field("\u{200B}", goodbye_message, false);

    embed
}

pub async fn ensure_host_role(ctx: &Context<'_>, user: &serenity::User) -> Result<bool, serenity::Error> {
    let guild_id_int: u64 = env::var("GUILD_ID")
    .expect("Missing `GUILD_ID` env var, see README for more information.")
    .parse().expect("Error parsing guild id to int");
    let role_id_int: u64 = env::var("HOST_ROLE")
    .expect("Missing `HOST_ROLE` env var, see README for more information.")
    .parse().expect("Error parsing host id to int");

    let guild_id = serenity::GuildId::new(guild_id_int);
    let role_id = serenity::RoleId::new(role_id_int);

    let res = user.has_role(ctx.http(), guild_id, role_id).await?;

    if !res {
        ctx.send(CreateReply::default().content("You do not have the host role").ephemeral(true)).await?;
    }

    Ok(res)
}

pub async fn ensure_correct_phase(ctx: &Context<'_>, allowed_phase: Vec<Phase>) -> Result<bool, serenity::Error> {
    let phase = database::get_phase().expect("Error getting a phase");
    if !allowed_phase.contains(&phase) {
        ctx.send(
            CreateReply::default()
                .content("You are trying to run a command that is not allowed in this phase!")
                .ephemeral(true),
        )
        .await?;
        return Ok(false)
    } 
    Ok(true)
}

pub async fn reject_if_already_running<F, Fut>(ctx: &Context<'_>, action: F) -> Result<(), Error>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<(), Error>>,
{
    let user_id = ctx.author().id.get();

    {
        let mut pending = ctx.data().pending_users.lock().await;
        if !pending.insert(user_id) {
            ctx.say("You're already running this command. Please finish or cancel it first.").await?;
            return Ok(());
        }
    }

    let result = action().await;

    {
        let mut pending = ctx.data().pending_users.lock().await;
        pending.remove(&user_id);
    }

    result
}

pub async fn wait_for_message_with_cancel(
    ctx: &Context <'_>,
    message_content: &str,
) -> Result<Option<String>, serenity::Error> {
    // Send the message with a cancel button
    let channel_id = ctx.channel_id();
    let user_id = ctx.author().id;
    let cancel_button = CreateButton::new("cancel_btn")
        .label("Cancel")
        .style(ButtonStyle::Danger);

    let action_row = vec![CreateActionRow::Buttons(vec![cancel_button])];
    let message = CreateReply::default().content(message_content).components(action_row);
    ctx.send(message).await?;

    let mut message_stream = MessageCollector::new(ctx)
        .author_id(user_id)
        .channel_id(channel_id)
        .stream();

    let mut cancel_stream = ComponentInteractionCollector::new(ctx)
        .filter(move |interaction| {
            interaction.data.custom_id == "cancel_btn" && interaction.user.id == user_id
        })
        .stream();
        loop {
            tokio::select! {
                Some(msg) = message_stream.next() => {
                    return Ok(Some(msg.content.clone()));
                },
                Some(cancel_interaction) = cancel_stream.next() => {
                    // Acknowledge the interaction first
                    let resp = CreateInteractionResponseMessage::new().content("Command canceled");
                    if let Err(err) = cancel_interaction.create_response(ctx.http(), CreateInteractionResponse::Message(resp)).await {
                        eprintln!("Error acknowledging cancel interaction: {}", err);
                    }
    
                    // Return None to indicate that the user canceled the action
                    return Ok(None);
                }
            }
        }
}

pub async fn ensure_has_giftee(ctx: &Context<'_>) -> Result<bool, serenity::Error> {
    let user_id = ctx.author().id.get();
    match database::check_if_matched(user_id).await {
        Ok(b) => {
            if b {
                Ok(true)
            } else {
                ctx.send(
                    CreateReply::default()
                        .content("You do not have a giftee")
                        .ephemeral(true),
                )
                .await?;
                return Ok(false);
            }
        },
        Err(_) => {
            ctx.send(
                CreateReply::default()
                    .content("An error occured checking if you have a giftee")
                    .ephemeral(true),
            )
            .await?;
            return Ok(false);
        },
    }
}

pub async fn ensure_has_santa(ctx: &Context<'_>) -> Result<bool, serenity::Error> {
    let user_id = ctx.author().id.get();
    match database::check_if_matched(user_id).await {
        Ok(b) => {
            if b {
                Ok(true)
            } else {
                ctx.send(
                    CreateReply::default()
                        .content("You do not have a santa")
                        .ephemeral(true),
                )
                .await?;
                return Ok(false);
            }
        },
        Err(_) => {
            ctx.send(
                CreateReply::default()
                    .content("An error occured checking if you have a santa")
                    .ephemeral(true),
            )
            .await?;
            return Ok(false);
        },
    }
}
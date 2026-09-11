use crate::{
    Context, Error, database, types::GraphLayout, types::Phase, utilities::ensure_host_role,
};
use petgraph::Graph;
use petgraph::dot::{Config, Dot};
use petgraph::graph::NodeIndex;
use poise::CreateReply;
use rusqlite::Result;
use serenity::all::{CreateAttachment, CreateMessage};
use std::process::Stdio;
use tokio::io::AsyncWriteExt;
use tokio::process::Command as TokioCommand;
#[poise::command(prefix_command, slash_command)]
pub async fn reveal_graph(
    ctx: Context<'_>,
    #[description = "Graph layout"] mut layout: GraphLayout,
) -> Result<(), Error> {
    if !ensure_host_role(&ctx, ctx.author()).await? {
        return Ok(());
    }
    if !crate::utilities::ensure_correct_phase(&ctx, vec![Phase::Swap, Phase::Watch]).await? {
        return Ok(());
    }
    let users = match database::get_matching_order() {
        Ok(users) => users,
        Err(e) => {
            eprintln!("Error getting matching order: {}", e);

            ctx.send(
                CreateReply::default()
                    .content("Error getting matching order.")
                    .ephemeral(true),
            )
            .await?;

            return Ok(());
        }
    };
    layout = match layout {
        GraphLayout::Default => GraphLayout::Circo,
        GraphLayout::Random => {
            let layouts = [
                GraphLayout::Dot,
                GraphLayout::Neato,
                GraphLayout::Fdp,
                GraphLayout::Circo,
                GraphLayout::Twopi,
                GraphLayout::Osage,
                GraphLayout::Patchwork,
            ];
            layouts[rand::random_range(0..layouts.len())]
        }
        _ => { layout }
    };
    let message = format!("heres the graph :happy: ({:?})", layout);
    let graph_creation_status = TokioCommand::new(format!("{:?}", layout).to_lowercase())
        .args(["-Tpng", "graph.dot", "-o", "graph.png", "-Nshape=none"])
        .status()
        .await;

    if let Err(e) = graph_creation_status {
        eprintln!("Command failed: {}", e);
        ctx.send(
            CreateReply::default()
                .content("Command failed, feature may not be available")
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    }

    let attachment = match CreateAttachment::path("graph.png").await {
        Ok(attachment) => attachment,
        Err(e) => {
            eprintln!("Error creating attachment: {}", e);
            ctx.send(
                CreateReply::default()
                    .content("Couldnt create graph attachment")
                    .ephemeral(true),
            )
            .await?;
            return Ok(());
        }
    };
    match ctx.author().direct_message(
        ctx.http(),
        CreateMessage::new()
            .content(message)
            .add_file(attachment),
    ).await {
        Ok(_) => {
            ctx.send(
                CreateReply::default()
                    .content("The graph(s) have been sent to your DMs.")
                    .ephemeral(true),
            )
            .await?;
        }
        Err(e) => {
            eprintln!("Error sending reveal DM: {}", e);

            ctx.send(
                CreateReply::default()
                    .content("I couldn't send you a DM. Please make sure your DMs are open.")
                    .ephemeral(true),
            )
            .await?;
        }
    }
    Ok(())
}

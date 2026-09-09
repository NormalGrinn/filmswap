use crate::{
    Context, Error, database, types::Phase, utilities::ensure_host_role,types::GraphLayout
};
use poise::CreateReply;
use rusqlite::Result;
use serenity::all::{ CreateMessage, CreateAttachment };
use petgraph::Graph;
use petgraph::graph::NodeIndex;
use petgraph::dot::{Dot, Config};
use std::process::Command;
#[poise::command(prefix_command, slash_command)]
pub async fn reveal_graph(
    ctx: Context<'_>,
    #[description = "Graph layout"]
    layout: GraphLayout,
) -> Result<(), Error> {
    if !ensure_host_role(&ctx, ctx.author()).await? {
        return Ok(());
    }
    if !crate::utilities::ensure_correct_phase(&ctx, vec![Phase::Swap, Phase::Watch]).await? {return Ok(())}
    
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

    let mut graph = Graph::<&str, &str>::new();

    let mut user_nodes: Vec<NodeIndex> = Vec::new();

    for user in users.iter() {
        user_nodes.push(graph.add_node(user.1.as_str()));
    }
    // create edges
    let mut edges: Vec<(NodeIndex, NodeIndex)> = Vec::new();
    // connect users sequenitally then wrap last user.
    for i in 0..user_nodes.len() {
        edges.push((user_nodes[i], user_nodes[(i + 1) % user_nodes.len()]));
    }
    // add edges to graph
    graph.extend_with_edges(&edges);
        let dot_output = format!("{}", Dot::with_config(&graph, &[Config::EdgeNoLabel]));

    std::fs::write("graph.dot", &dot_output).unwrap();


    let mut layout = layout;
    if format!("{:?}", layout) == "default" {
        layout = GraphLayout::circo;
    }
    println!("Using layout: {:?}", layout);
    Command::new(format!("{:?}",layout))
        .args(["-Tpng", "graph.dot", "-o", "graph.png", "-Nshape=none"])
        .status()
        .expect("failed to run graphviz `dot` — is it installed?");
    //TODO: implement way for host to ask for a random one or just give a random one lol.
    let message = format!("heres the graph :happy:");
    
    match ctx.author().direct_message(
        ctx.http(),
        CreateMessage::new()
            .content(message)
            .add_file(CreateAttachment::path("graph.png").await.unwrap()),
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
    tokio::fs::remove_file("graph.png").await?;
    tokio::fs::remove_file("graph.dot").await?;
    Ok(())
}

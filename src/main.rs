use std::{collections::HashSet, env, sync::Arc};

use poise::serenity_prelude as serenity;
use dotenvy::dotenv;
use tokio::sync::Mutex;

mod commands;
mod utilities;
mod database;
mod components;
mod types;

struct Data {
    pub pending_users: Arc<Mutex<HashSet<u64>>>,
} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let token = env::var("TOKEN")
        .expect("Missing `TOKEN` env var, see README for more information.");
    let intents =
        serenity::GatewayIntents::non_privileged() 
        | serenity::GatewayIntents::MESSAGE_CONTENT;
    let data = Data {
        pending_users: Arc::new(Mutex::new(HashSet::new())),
    };

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                commands::create_button::create_button(),
                commands::leave::leave(),
                commands::read_letter::read_letter(),
                commands::write_letter::write_letter(),
                commands::submit::submit(),
                commands::read::read(),
                commands::receive::receive(),
                commands::set_phase::set_phase(),
                commands::write_giftee::write_giftee(),
                commands::write_santa::write_santa(),
                commands::match_users::match_users(),
                commands::status::status(),
                commands::unmatch_users::unmatch_users(),
                commands::ban_user::ban_user(),
                commands::unban_user::unban_user(),
                commands::reveal::reveal(),
            ],
            event_handler: |ctx, event, framework, data| {
                Box::pin(components::button_interaction::on_component_interaction(ctx, event, framework, data))
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(data)
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;
    client.unwrap().start().await.unwrap();
}

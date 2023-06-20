use serenity::async_trait;
use serenity::prelude::*;
use serenity::model::channel::Message;

use std::fs;

use crate::config::{init_config, PREFIX};
use crate::commands::*;

mod config;
mod commands;

struct Handler {}

async fn command_handler(ctx: &Context, msg: &Message) -> Result<(), Box<dyn std::error::Error>> {
    // split the args into command and its arguments
    let arg = msg.content.split_once(" ").unwrap_or(("", ""));

    // match the args to existing commands
    match arg.0 {
        "!bolt" => bolt_cmd(&ctx, &msg).await?,
        "!rm" => remove_cmd(&ctx, &msg, &arg.1).await?,
        "!r" => roll_cmd(&ctx, &msg, &arg.1).await?,
        "!join" => join_cmd(&ctx, &msg).await?,
        "!lv" => leave_cmd(&ctx, &msg).await?,
        "!pts" => points_cmd(&ctx, &msg, arg.1).await?,
        "!teams" => teams_cmd(&ctx, &msg, arg.1).await?,
        "!conv" => convert_cmd(&ctx, &msg, arg.1).await?,
        "!ls" => ls_cmd(&ctx, &msg).await?,
        &_ => (),
    }
    Ok(())
}

#[async_trait] 
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        // check if a message begins with the prefix, and handle command errors 
        if msg.content.starts_with(*PREFIX.get().unwrap()) {
            if let Err(why) = command_handler(&ctx, &msg).await {
                eprintln!("Command Err!: {}", why);
            }
        }
    }
}

#[tokio::main]
async fn main() {
    // initialize the config
    init_config().await.expect("failed to initialize config!");

    // read token 
    let token = fs::read_to_string("token")
        .expect("Could not read the token file!");

    let handler = Handler { };

    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(token, intents)
        .event_handler(handler)
        .await
        .expect("Err Creating Client!");

    if let Err(why) = client.start().await { 
        eprintln!("Client Err: {:?}", why);
    }
}

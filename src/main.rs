use serenity::async_trait;
use serenity::prelude::*;
use serenity::model::channel::Message;

use std::fs;

use crate::config::init_config;
use crate::commands::*;

mod config;
mod commands;

struct Handler {}

#[async_trait] 
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        // bolt teams roll
        if msg.content.trim() == "!bolt" {
            if let Err(why) = bolt_cmd(&ctx, &msg).await {
                eprintln!("Failed to run the Bolt Command!: {}", why);
            }
            return;
        }

        // dice roll command
        if let Some(arg) = msg.content.strip_prefix("!r") {
            roll_cmd(&ctx, &msg, &arg).await;
            return;
        }

        // join the bolt
        if msg.content.trim() == "!join" {
            if let Err(why) = join_cmd(&ctx, &msg).await {
                eprintln!("Failed to run the Join Command!: {}", why);
            }
            return;
        }

        // leave the bolt
        if msg.content.trim() == "!lv" {
            if let Err(why) = leave_cmd(&ctx, &msg).await {
                eprintln!("Failed to run the Leave Command!: {}", why);
            }
            return;
        }

        // set the min/max values for points
        if let Some(arg) = msg.content.strip_prefix("!points") {
            if let Err(why) = points_cmd(&ctx, &msg, &arg).await {
                eprintln!("Failed to run the Points Command!: {}", why);
            }
            return;
        }

        // change the num of teams
        if let Some(arg) = msg.content.strip_prefix("!teams") {
            if let Err(why) = teams_cmd(&ctx, &msg, &arg).await {
                eprintln!("Failed to run the Teams Command!: {}", why);
            }
            return;
        }

        // list all players
        if msg.content.trim() == "!ls" {
            if let Err(why) = ls_cmd(&ctx, &msg).await {
                eprintln!("Failed to run the List Command!: {}", why);
            }
            return;
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

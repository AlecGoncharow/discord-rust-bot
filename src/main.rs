#[macro_use]
extern crate serenity;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;
extern crate chrono;
extern crate artifact_lib;
extern crate rand;
extern crate regex;
extern crate reqwest;
extern crate serde_json;

use serenity::{
    framework::StandardFramework,
    model::{channel::Message, gateway::Ready, id::UserId},
    prelude::*,
};
use std::env;
use artifact_lib::Artifact;
mod commands;
struct Handler;



lazy_static! {
    static ref ARTIFACT: Artifact = Artifact::new();
}

static DEV_ID: u64 = 198321762537177088;

impl EventHandler for Handler {
    fn message(&self, _ctx: Context, _new_message: Message) {
        println!("{}", _new_message.content);
    }
    fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
        println!("{:?}", ARTIFACT.card_from_name("drow ranger").unwrap());
    }
}

fn main() {
    // Login with a bot token from the environment
    let mut client = Client::new(&env::var("DISCORD_BOT_TOKEN").expect("token"), Handler)
        .expect("Error creating client");
    client.with_framework(
        StandardFramework::new()
            .configure(|c| {
                c.prefix("-")
                    .owners(vec![UserId(DEV_ID)].into_iter().collect())
            }) // set the bot's prefix to "-"
            .cmd("ping", commands::misc::ping)
            .cmd("mort", commands::misc::mort)
            .cmd("morton", commands::misc::mort)
            .cmd("multiply", commands::misc::multiply)
            .cmd("poll", commands::poll::poll)
            .cmd("random", commands::dota::random)
            .group("Tips", |g| {
                g.prefix("tips")
                    .command("profile", |c| c.cmd(commands::tip::profile))
            })
            .group("Admin", |g| {
                g.prefix("admin")
                    .command("quit", |c| c.cmd(commands::admin::quit).owners_only(true))
            })
            .cmd("tip", commands::tip::handle_tip)
            .cmd("antitip", commands::tip::handle_tip)
            .cmd("profile", commands::tip::profile)
            .cmd("card", commands::artifact::get_card)
            .cmd("deck", commands::artifact::get_deck)
            .group("Artifact", |g| {
                g.prefix("artifact")
                    .command("card", |c| c.cmd(commands::artifact::get_card))
                    .command("deck", |c| c.cmd(commands::artifact::get_deck))
            }),
    );
    let user = serenity::model::id::UserId(DEV_ID).to_user().unwrap();

    let _ = user.direct_message(|m| m.content("ONLINE"));
    // start listening for events by starting a single shard
    if let Err(why) = client.start() {
        println!("An error occurred while running the client: {:?}", why);
        let _ = user.direct_message(|m| m.content(why.to_string()));
    }
}

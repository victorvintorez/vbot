use poise::serenity_prelude as serenity;
use std::{collections::HashMap, error, sync::Mutex};
use tracing::{info, warn};

mod commands;
mod listeners;

const GUILD_ID: u64 = 1347211535554183298;
const VERIFIED_ROLE_ID: u64 = 1347975552262340630;

pub type Error = Box<dyn error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

pub struct Data {
    unverified_members: Mutex<HashMap<u64, String>>, // [ username, userid ]
    custom_roles: Mutex<HashMap<u32, u32>>,          // [ roleid, userid ]
}

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    match error {
        poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {:?}", error),
        poise::FrameworkError::Command { error, ctx, .. } => {
            warn!("Error in command `{}`: {:?}", ctx.command().name, error);
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                warn!("Unhandled error: {}", e)
            }
        }
    }
}

#[shuttle_runtime::main]
async fn main(
    #[shuttle_runtime::Secrets] secrets: shuttle_runtime::SecretStore,
) -> shuttle_serenity::ShuttleSerenity {
    let options = poise::FrameworkOptions {
        commands: vec![commands::help(), commands::verify()],
        prefix_options: poise::PrefixFrameworkOptions {
            prefix: None,
            ..Default::default()
        },
        on_error: |error| Box::pin(on_error(error)),
        pre_command: |ctx| {
            Box::pin(async move {
                info!(
                    "User {} executing Command {}...",
                    ctx.author().name,
                    ctx.command().qualified_name
                )
            })
        },
        post_command: |ctx| {
            Box::pin(async move {
                info!(
                    "User {} executed Command {}!",
                    ctx.author().name,
                    ctx.command().qualified_name
                )
            })
        },
        event_handler: |ctx, event, framework, state| {
            Box::pin(listeners::listen(ctx, event, framework, state))
        },
        ..Default::default()
    };

    let framework = poise::Framework::builder()
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                info!("Logged in as {}", _ready.user.name);
                poise::builtins::register_in_guild(
                    ctx,
                    &framework.options().commands,
                    GUILD_ID.into(),
                )
                .await?;

                Ok(Data {
                    unverified_members: Mutex::new(HashMap::new()),
                    custom_roles: Mutex::new(HashMap::new()),
                })
            })
        })
        .options(options)
        .build();

    let token = secrets
        .get("DISCORD_TOKEN")
        .expect("Missing `DISCORD_TOKEN` env var");
    let intents = serenity::GatewayIntents::privileged();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await
        .expect("error creating client");

    Ok(client.into())
}

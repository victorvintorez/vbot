use poise::serenity_prelude as serenity;
use tracing::warn;

use crate::{Context, Error, VERIFIED_ROLE_ID, is_mod};

#[poise::command(slash_command)]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Choose command for relevant help"]
    #[autocomplete = "poise::builtins::autocomplete_command"]
    command: Option<String>,
) -> Result<(), Error> {
    poise::builtins::help(
        ctx,
        command.as_deref(),
        poise::builtins::HelpConfiguration {
            ..Default::default()
        },
    )
    .await?;
    Ok(())
}

#[poise::command(slash_command, subcommands("show", "verify"), subcommand_required)]
pub async fn waitlist(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(slash_command)]
pub async fn show(ctx: Context<'_>) -> Result<(), Error> {
    let mut response = String::new();
    if let Ok(hash_map) = ctx.data().unverified_members.lock() {
        response += &"Users on Waitlist";
        if hash_map.len() > 0 {
            for (_id, name) in hash_map.iter() {
                response += &format!("\n- {}", name)
            }
        } else {
            response += &"\nNone!";
        };
    } else {
        response = "Couldn't fetch the list of users on the waitlist!".to_string();
    };
    ctx.say(response).await?;
    Ok(())
}

#[poise::command(slash_command, check = "is_mod")]
pub async fn verify(
    ctx: Context<'_>,
    #[description = "Choose member to allow access to server"]
    #[autocomplete = "poise::builtins::autocomplete_command"]
    member: serenity::Member,
) -> Result<(), Error> {
    let response: String;

    if !member.roles.contains(&VERIFIED_ROLE_ID) {
        let role_add_success = match member.add_role(ctx, VERIFIED_ROLE_ID).await {
            Ok(_) => true,
            Err(_err) => false,
        };

        if role_add_success {
            match ctx.data().unverified_members.lock() {
                Ok(mut hash_map) => {
                    hash_map.remove(&member.user.id.into());
                    response = format!("{} now has access to the server!", member.user.name);
                }
                Err(_err) => {
                    response = format!(
                        "Failed to remove {} from waitlist (user may still have been verified)...",
                        member.user.name
                    )
                }
            }
        } else {
            response = format!("Failed to add Verified Role to {}...", member.user.name);
        }
    } else {
        response = format!("{} is already verified!", member.user.name);
    };

    ctx.say(response).await?;
    Ok(())
}

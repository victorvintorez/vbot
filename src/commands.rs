use poise::serenity_prelude as serenity;

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
    if let Ok(hash_map) = ctx.data().waitlist.lock() {
        response += &"Users on Waitlist";
        if hash_map.len() > 0 {
            for (id, _name) in hash_map.iter() {
                response += &format!("\n- <@{}>", id)
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
            match ctx.data().waitlist.lock() {
                Ok(mut hash_map) => {
                    hash_map.remove(&member.user.id.into());
                    response = format!("<@{}> now has access to the server!", member.user.id);
                }
                Err(_err) => {
                    response = format!(
                        "Failed to remove <@{}> from waitlist (user may still have been verified)...",
                        member.user.id
                    )
                }
            }
        } else {
            response = format!("Failed to add Verified Role to <@{}>...", member.user.id);
        }
    } else {
        response = format!("<@{}> is already verified!", member.user.id);
    };

    ctx.say(response).await?;
    Ok(())
}

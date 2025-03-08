use poise::serenity_prelude as serenity;
use tracing::warn;

use crate::{Context, Error, VERIFIED_ROLE_ID};

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

#[poise::command(slash_command)]
pub async fn verify(
    ctx: Context<'_>,
    #[description = "Choose member to allow access to server"]
    #[autocomplete = "poise::builtins::autocomplete_command"]
    member: Option<serenity::Member>,
) -> Result<(), Error> {
    if let Some(member) = member {
        let member_exists = match ctx.data().unverified_members.lock() {
            Ok(hash_map) => hash_map.contains_key(&member.user.id.into()),
            Err(err) => {
                warn!("Could not allow member access to server: {:?}", err);
                false
            }
        };
        if member_exists {
            member
                .add_role(ctx, serenity::RoleId::new(VERIFIED_ROLE_ID))
                .await?;
            let success = match ctx.data().unverified_members.lock() {
                Ok(mut hash_map) => {
                    hash_map.remove(&member.user.id.into());
                    true
                }
                Err(err) => {
                    warn!("Could not allow member access to server: {:?}", err);
                    false
                }
            };

            if success {
                ctx.say(format!(
                    "{} should now be able to access the server!",
                    member.user.name,
                ))
                .await?;
            } else {
                ctx.say(format!(
                    "Could not allow {} access to server: A server error occurred!",
                    member.user.name
                ))
                .await?;
            }
        } else {
            ctx.say(format!("{} is already verified!", member.user.name))
                .await?;
        }
    } else {
        // show list of users with buttons to verify
    };

    Ok(())
}

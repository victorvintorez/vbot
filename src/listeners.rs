use poise::serenity_prelude as serenity;
use tracing::warn;

use crate::{Data, Error, GUILD_ID, VERIFIED_ROLE_ID};

pub async fn listen(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::Ready { data_about_bot: _ } => {
            onready_waitlist(ctx, data).await;
        }
        serenity::FullEvent::GuildMemberAddition { new_member } => {
            guildmemberaddition_waitlist(data, new_member).await;
        }
        serenity::FullEvent::GuildMemberRemoval {
            guild_id: _,
            user,
            member_data_if_available: _,
        } => {
            guildmemberremoval_waitlist(data, user).await;
        }
        _ => {}
    };
    Ok(())
}

async fn onready_waitlist(ctx: &serenity::Context, data: &Data) {
    let guild = serenity::GuildId::from(GUILD_ID);
    match guild.members(ctx, None, None).await {
        Ok(guild_members) => match data.waitlist.lock() {
            Ok(mut hash_map) => {
                guild_members.iter().for_each(|member| {
                    if !member.roles.contains(&VERIFIED_ROLE_ID) {
                        hash_map.insert(member.user.id.into(), member.user.name.clone());
                    }
                });
            }
            Err(err) => warn!("Could not load initial unverified_member list: {:?}", err),
        },
        Err(err) => warn!("Could not fetch guild members in Ready: {:?}", err),
    };
}

async fn guildmemberaddition_waitlist(data: &Data, new_member: &serenity::Member) {
    match data.waitlist.lock() {
        Ok(mut hash_map) => {
            hash_map.insert(new_member.user.id.into(), new_member.user.name.clone());
        }
        Err(err) => warn!(
            "Could not add new member to unverified_member list: {:?}",
            err
        ),
    };
}

async fn guildmemberremoval_waitlist(data: &Data, user: &serenity::User) {
    match data.waitlist.lock() {
        Ok(mut hash_map) => {
            if hash_map.contains_key(&user.id.into()) {
                hash_map.remove(&user.id.into());
            }
        }
        Err(err) => warn!(
            "Could not remove member from unverified_member list: {:?}",
            err
        ),
    };
}

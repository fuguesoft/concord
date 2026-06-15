use crate::Result;
use crate::discord::ids::{Id, marker::GuildMarker};

use super::DiscordRest;

impl DiscordRest {
    pub async fn leave_guild(&self, guild_id: Id<GuildMarker>) -> Result<()> {
        self.send_unit(
            self.raw_http.delete(format!(
                "https://discord.com/api/v9/users/@me/guilds/{}",
                guild_id.get()
            )),
            "leave guild",
        )
        .await
    }
}

use serde_json::json;

use crate::{Result, discord::PresenceStatus};

use super::DiscordRest;

impl DiscordRest {
    pub async fn update_current_user_status(&self, status: PresenceStatus) -> Result<()> {
        self.send_unit(
            self.raw_http
                .patch("https://discord.com/api/v9/users/@me/settings")
                .json(&json!({ "status": status.gateway_status() })),
            "status settings update",
        )
        .await
    }
}

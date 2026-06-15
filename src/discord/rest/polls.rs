use crate::Result;
use crate::discord::ids::{
    Id,
    marker::{ChannelMarker, MessageMarker},
};

use super::DiscordRest;

impl DiscordRest {
    pub async fn vote_poll(
        &self,
        channel_id: Id<ChannelMarker>,
        message_id: Id<MessageMarker>,
        answer_ids: &[u8],
    ) -> Result<()> {
        let url = format!(
            "https://discord.com/api/v9/channels/{}/polls/{}/answers/@me",
            channel_id.get(),
            message_id.get()
        );
        self.send_unit(
            self.raw_http
                .put(url)
                .json(&poll_vote_request_body(answer_ids)),
            "poll vote",
        )
        .await
    }
}

pub(super) fn poll_vote_request_body(answer_ids: &[u8]) -> serde_json::Value {
    serde_json::json!({ "answer_ids": answer_ids })
}

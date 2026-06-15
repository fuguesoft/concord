use std::sync::Arc;

use tokio::sync::{Semaphore, mpsc};

use crate::{DiscordClient, discord::AppCommand, error::AppError, logging};

use super::{
    gateway_commands, history_commands, media_commands, message_commands, notification_commands,
    read_state_commands, user_commands, voice_commands,
};

const MAX_CONCURRENT_ATTACHMENT_PREVIEWS: usize = 4;
const MAX_CONCURRENT_ATTACHMENT_DOWNLOADS: usize = 2;

pub(super) fn start_command_loop(
    client: DiscordClient,
    mut commands: mpsc::Receiver<AppCommand>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let attachment_preview_permits =
            Arc::new(Semaphore::new(MAX_CONCURRENT_ATTACHMENT_PREVIEWS));
        let attachment_download_permits =
            Arc::new(Semaphore::new(MAX_CONCURRENT_ATTACHMENT_DOWNLOADS));
        // Spawn slow commands independently so REST and media work do not block
        // the whole UI command queue. Cheap control-plane commands stay inline
        // because their order matters when the user navigates quickly.
        while let Some(command) = commands.recv().await {
            if is_ordered_control_command(&command) {
                handle_command(
                    client.clone(),
                    command,
                    attachment_preview_permits.clone(),
                    attachment_download_permits.clone(),
                )
                .await;
                continue;
            }
            let client = client.clone();
            let attachment_preview_permits = attachment_preview_permits.clone();
            let attachment_download_permits = attachment_download_permits.clone();
            tokio::spawn(async move {
                handle_command(
                    client,
                    command,
                    attachment_preview_permits,
                    attachment_download_permits,
                )
                .await;
            });
        }
    })
}

async fn handle_command(
    client: DiscordClient,
    command: AppCommand,
    attachment_preview_permits: Arc<Semaphore>,
    attachment_download_permits: Arc<Semaphore>,
) {
    match command {
        command @ (AppCommand::LoadMessageHistory { .. }
        | AppCommand::RefreshMessageHistory { .. }
        | AppCommand::LoadMessageHistoryAfter { .. }
        | AppCommand::CatchUpMessageHistoryAfter { .. }
        | AppCommand::LoadMessageHistoryAround { .. }
        | AppCommand::LoadThreadPreview { .. }
        | AppCommand::LoadForumPosts { .. }
        | AppCommand::SearchMessages { .. }) => {
            history_commands::handle(client, command).await;
        }
        command @ (AppCommand::LoadGuildMembers { .. }
        | AppCommand::LoadGuildMembersByIds { .. }
        | AppCommand::SearchGuildMembers { .. }
        | AppCommand::SetSelectedGuild { .. }
        | AppCommand::SetSelectedMessageChannel { .. }
        | AppCommand::SubscribeDirectMessage { .. }
        | AppCommand::SubscribeGuildChannel { .. }
        | AppCommand::UpdateMemberListSubscription { .. }) => {
            gateway_commands::handle(client, command).await;
        }
        command @ (AppCommand::JoinVoiceChannel { .. }
        | AppCommand::UpdateVoiceState { .. }
        | AppCommand::UpdateVoiceCapturePermission { .. }
        | AppCommand::LeaveVoiceChannel { .. }) => {
            voice_commands::handle(client, command).await;
        }
        command @ (AppCommand::LoadAttachmentPreview { .. }
        | AppCommand::LoadProfileAvatarPreview { .. }
        | AppCommand::OpenUrl { .. }
        | AppCommand::PlayMedia { .. }
        | AppCommand::DownloadAttachment { .. }) => {
            media_commands::handle(
                client,
                command,
                attachment_preview_permits,
                attachment_download_permits,
            )
            .await;
        }
        command @ (AppCommand::SendMessage { .. }
        | AppCommand::SendTtsMessage { .. }
        | AppCommand::LoadApplicationCommands { .. }
        | AppCommand::RunApplicationCommand { .. }
        | AppCommand::EditMessage { .. }
        | AppCommand::DeleteMessage { .. }
        | AppCommand::LeaveGuild { .. }
        | AppCommand::AddReaction { .. }
        | AppCommand::RemoveReaction { .. }
        | AppCommand::LoadReactionUsers { .. }
        | AppCommand::LoadPinnedMessages { .. }
        | AppCommand::SetMessagePinned { .. }
        | AppCommand::VotePoll { .. }) => {
            message_commands::handle(client, command).await;
        }
        command @ (AppCommand::LoadUserProfile { .. }
        | AppCommand::LoadUserNote { .. }
        | AppCommand::UpdateUserProfile { .. }
        | AppCommand::UpdateCurrentUserStatus { .. }
        | AppCommand::UpdateCurrentUserActivity { .. }) => {
            user_commands::handle(client, command).await;
        }
        command @ (AppCommand::AckChannel { .. }
        | AppCommand::ScheduleAckChannel { .. }
        | AppCommand::AckChannels { .. }) => {
            read_state_commands::handle(client, command).await;
        }
        command @ (AppCommand::SetGuildMuted { .. } | AppCommand::SetChannelMuted { .. }) => {
            notification_commands::handle(client, command).await;
        }
    }
}

fn is_ordered_control_command(command: &AppCommand) -> bool {
    matches!(
        command,
        AppCommand::SetSelectedGuild { .. }
            | AppCommand::SetSelectedMessageChannel { .. }
            | AppCommand::UpdateVoiceCapturePermission { .. }
    )
}

pub(super) fn log_app_error(context: &str, error: &AppError) {
    logging::error(
        "app",
        format!("{context}: {}; detail={}", error, error.log_detail()),
    );
}

#[cfg(test)]
mod tests {
    use crate::{
        config::{MicrophoneSensitivityDb, VoiceVolumePercent},
        discord::ids::Id,
    };

    use super::*;

    #[test]
    fn only_order_sensitive_control_commands_run_inline() {
        assert!(is_ordered_control_command(&AppCommand::SetSelectedGuild {
            guild_id: Some(Id::new(1)),
        }));
        assert!(is_ordered_control_command(
            &AppCommand::SetSelectedMessageChannel {
                channel_id: Some(Id::new(2)),
            }
        ));
        assert!(is_ordered_control_command(
            &AppCommand::UpdateVoiceCapturePermission {
                guild_id: Id::new(1),
                channel_id: Id::new(2),
                allow_microphone_transmit: true,
                microphone_sensitivity: MicrophoneSensitivityDb::default(),
                microphone_volume: VoiceVolumePercent::default(),
                voice_output_volume: VoiceVolumePercent::default(),
            }
        ));

        assert!(!is_ordered_control_command(
            &AppCommand::LoadMessageHistory {
                channel_id: Id::new(2),
                before: None,
            }
        ));
        assert!(!is_ordered_control_command(
            &AppCommand::LoadAttachmentPreview {
                url: "https://cdn.discordapp.com/avatar.png".to_owned(),
            }
        ));
    }
}

use crate::discord::ids::{
    Id,
    marker::{GuildMarker, UserMarker},
};
use crate::discord::{ActivityInfo, AppCommand, PresenceStatus, UserProfileInfo};
use crate::tui::keybindings::KeyChord;

use super::super::model::{FocusPane, MemberActionItem, MemberActionKind};
use super::super::{ActiveGuildScope, DashboardState};
use super::{
    ActiveModalPopupKind, LeaderActionState, LeaderMode, LeaderPopupState, MemberLeaderActionState,
    ModalPopup, UserProfilePopupState,
};

impl DashboardState {
    pub fn is_member_leader_action_active(&self) -> bool {
        self.popups.member_leader_action().is_some()
    }

    /// Direct shortcut from the member pane: open the profile popup for the
    /// currently selected member without going through Leader Actions.
    pub fn show_selected_member_profile(&mut self) -> Option<AppCommand> {
        if self.navigation.focus != FocusPane::Members {
            return None;
        }
        let entries = self.flattened_members();
        let entry = entries.get(self.selected_member())?;
        let user_id = entry.user_id();
        let guild_id = match self.navigation.active_guild {
            ActiveGuildScope::Guild(guild_id) => Some(guild_id),
            ActiveGuildScope::DirectMessages | ActiveGuildScope::Unset => None,
        };
        self.open_user_profile_popup(user_id, guild_id)
    }

    pub fn open_selected_member_actions(&mut self) {
        if let Some(action) = self.selected_member_action_context() {
            self.popups.modal = Some(ModalPopup::Leader(LeaderPopupState {
                mode: LeaderMode::Actions,
                keymap_prefix: Vec::new(),
                action: Some(LeaderActionState::Member(action)),
            }));
        }
    }

    pub(super) fn selected_member_action_context(&self) -> Option<MemberLeaderActionState> {
        if self.navigation.focus != FocusPane::Members {
            return None;
        }
        let entries = self.flattened_members();
        let entry = entries.get(self.selected_member())?;
        let user_id = entry.user_id();
        // For DM/group-DM panes there is no guild context. Pass it through so
        // the profile fetch can omit `guild_id` and skip the guild_member
        // section gracefully.
        let guild_id = match self.navigation.active_guild {
            ActiveGuildScope::Guild(guild_id) => Some(guild_id),
            ActiveGuildScope::DirectMessages | ActiveGuildScope::Unset => None,
        };
        Some(MemberLeaderActionState {
            user_id,
            guild_id,
            selection: Default::default(),
        })
    }

    pub fn close_member_leader_action(&mut self) {
        if self.is_member_leader_action_active() {
            self.popups.clear_modal();
        }
    }

    pub fn selected_member_action_items(&self) -> Vec<MemberActionItem> {
        if self.popups.member_leader_action().is_none() {
            return Vec::new();
        }
        vec![MemberActionItem {
            kind: MemberActionKind::ShowProfile,
            label: "Show profile".to_owned(),
            enabled: true,
        }]
    }

    pub fn select_member_action_row(&mut self, row: usize) -> bool {
        if row >= self.selected_member_action_items().len() {
            return false;
        }
        if let Some(action) = self.popups.member_leader_action_mut() {
            action.selection.select(row);
            return true;
        }
        false
    }

    pub fn activate_selected_member_action(&mut self) -> Option<AppCommand> {
        let action = self.popups.member_leader_action().cloned()?;
        let items = self.selected_member_action_items();
        let item = items
            .get(action.selection.selected_for_len(items.len()))?
            .clone();
        if !item.enabled {
            return None;
        }
        match item.kind {
            MemberActionKind::ShowProfile => {
                self.close_member_leader_action();
                self.open_user_profile_popup(action.user_id, action.guild_id)
            }
        }
    }

    pub fn activate_member_action_shortcut(&mut self, shortcut: KeyChord) -> Option<AppCommand> {
        let actions = self.selected_member_action_items();
        let index = self.options.key_bindings().matching_action_shortcut_index(
            &actions,
            shortcut,
            |key_bindings, actions, index| key_bindings.member_action_shortcuts(actions, index),
            |action| action.enabled,
        )?;
        self.select_member_action_row(index);
        self.activate_selected_member_action()
    }

    /// Opens the profile popup for `user_id`. The returned command is a profile
    /// open intent. Backend request lifecycle decides whether profile or note
    /// data is already cached or in flight.
    pub fn open_user_profile_popup(
        &mut self,
        user_id: Id<UserMarker>,
        guild_id: Option<Id<GuildMarker>>,
    ) -> Option<AppCommand> {
        self.popups.modal = Some(ModalPopup::UserProfile(UserProfilePopupState {
            user_id,
            guild_id,
            load_error: None,
            scroll: Default::default(),
        }));
        Some(AppCommand::LoadUserProfile { user_id, guild_id })
    }

    pub fn close_user_profile_popup(&mut self) {
        if self.is_active_modal_popup(ActiveModalPopupKind::UserProfile) {
            self.popups.clear_modal();
        }
    }

    pub fn user_profile_popup_data(&self) -> Option<&UserProfileInfo> {
        let popup = self.popups.user_profile_popup()?;
        self.discord
            .cache
            .user_profile(popup.user_id, popup.guild_id)
    }

    pub fn user_profile_popup_load_error(&self) -> Option<&str> {
        self.popups
            .user_profile_popup()
            .and_then(|popup| popup.load_error.as_deref())
    }

    pub fn user_profile_popup_status(&self) -> PresenceStatus {
        let Some(popup) = self.popups.user_profile_popup() else {
            return PresenceStatus::Unknown;
        };

        if let Some(guild_id) = popup.guild_id
            && let Some(status) = self
                .discord
                .members_for_guild(guild_id)
                .into_iter()
                .find(|member| member.user_id == popup.user_id)
                .map(|member| member.status)
        {
            return status;
        }

        if let Some(guild_id) = popup.guild_id
            && let Some(status) = self
                .discord
                .user_presence_for_guild(Some(guild_id), popup.user_id)
        {
            return status;
        }

        let recipient_status = self
            .discord
            .channels_for_guild(None)
            .into_iter()
            .flat_map(|channel| channel.recipients.iter())
            .find(|recipient| recipient.user_id == popup.user_id)
            .map(|recipient| recipient.status);

        recipient_status
            .filter(|status| *status != PresenceStatus::Unknown)
            .or_else(|| self.discord.cache.user_presence(popup.user_id))
            .unwrap_or(PresenceStatus::Unknown)
    }

    /// URL of the avatar image to render into the open profile popup. None
    /// when the popup is closed, the profile has not loaded yet, or the user
    /// has no avatar attachment.
    pub fn user_profile_popup_avatar_url(&self) -> Option<&str> {
        self.user_profile_popup_data()?.avatar_url.as_deref()
    }

    pub fn user_profile_popup_activities(&self) -> &[ActivityInfo] {
        let Some(popup) = self.popups.user_profile_popup() else {
            return &[];
        };
        self.discord
            .cache
            .user_activities_for_guild(popup.guild_id, popup.user_id)
    }

    /// Top-of-viewport row for the popup body. Used by the renderer.
    pub fn user_profile_popup_scroll(&self) -> usize {
        self.popups
            .user_profile_popup()
            .map(|popup| popup.scroll.scroll())
            .unwrap_or(0)
    }

    /// Renderer hook: passes the latest viewport height back so scroll
    /// methods can clamp without snapping past the last visible row.
    pub fn set_user_profile_popup_view_height(&mut self, height: usize) {
        if let Some(popup) = self.popups.user_profile_popup_mut() {
            popup.scroll.set_view_height(height);
        }
    }

    /// Renderer hook: stash the laid-out content height so scroll
    /// clamping is a constant-time check instead of recomputing layout.
    pub fn set_user_profile_popup_total_lines(&mut self, total_lines: usize) {
        if let Some(popup) = self.popups.user_profile_popup_mut() {
            popup.scroll.set_total_lines(total_lines);
        }
    }

    pub fn scroll_user_profile_popup_down(&mut self) {
        if let Some(popup) = self.popups.user_profile_popup_mut() {
            popup.scroll.scroll_down();
        }
    }

    pub fn scroll_user_profile_popup_up(&mut self) {
        if let Some(popup) = self.popups.user_profile_popup_mut() {
            popup.scroll.scroll_up();
        }
    }
}

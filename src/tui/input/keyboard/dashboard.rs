use crate::discord::AppCommand;
use crate::tui::keybindings::{
    DashboardAction, MessageShortcutAction, OptionsCategoryShortcut, SelectionAction, UiAction,
};
use crate::tui::state::{DashboardState, FocusPane, MessageActionKind};

pub(super) fn handle_dashboard_action(
    state: &mut DashboardState,
    focus: FocusPane,
    action: DashboardAction,
) -> Option<AppCommand> {
    match action {
        DashboardAction::Select(SelectionAction::Next) => {
            state.move_down();
            None
        }
        DashboardAction::Select(SelectionAction::Previous) => {
            state.move_up();
            state.next_older_history_command()
        }
        DashboardAction::MessageShortcut(action) => handle_message_shortcut_action(state, action),
        DashboardAction::Back => {
            if !state.return_from_pinned_message_view() {
                state.return_from_opened_thread();
            }
            None
        }
        DashboardAction::Quit => {
            state.open_quit_confirmation();
            None
        }
        DashboardAction::StartComposer => {
            state.start_composer();
            None
        }
        DashboardAction::FocusPane(pane) => {
            state.show_and_focus_pane(pane);
            None
        }
        DashboardAction::CycleFocusBackward => {
            state.cycle_focus_backward();
            None
        }
        DashboardAction::CycleFocusForward => {
            state.cycle_focus();
            None
        }
        DashboardAction::OpenFocusedPaneFilter => {
            state.open_pane_filter(focus);
            None
        }
        DashboardAction::ResizePaneLeft => {
            state.adjust_focused_pane_width(-1);
            None
        }
        DashboardAction::ResizePaneRight => {
            state.adjust_focused_pane_width(1);
            None
        }
        DashboardAction::HalfPageDown => {
            state.half_page_down();
            None
        }
        DashboardAction::HalfPageUp => {
            state.half_page_up();
            state.next_older_history_command()
        }
        DashboardAction::JumpTop => {
            state.jump_top();
            None
        }
        DashboardAction::JumpBottom => {
            state.jump_bottom();
            None
        }
        DashboardAction::ScrollMessageViewportDown => {
            state.scroll_message_viewport_down();
            None
        }
        DashboardAction::ScrollMessageViewportUp => {
            state.scroll_message_viewport_up();
            None
        }
        DashboardAction::ScrollHorizontalLeft => {
            state.scroll_focused_pane_horizontal_left();
            None
        }
        DashboardAction::ScrollHorizontalRight => {
            state.scroll_focused_pane_horizontal_right();
            None
        }
        DashboardAction::ActivateFocused => match focus {
            FocusPane::Guilds => {
                if state.confirm_selected_guild() {
                    state.focus_pane(FocusPane::Channels);
                }
                None
            }
            FocusPane::Channels => {
                let command = state.confirm_selected_channel_command();
                if command.is_some() {
                    state.focus_pane(FocusPane::Messages);
                }
                command
            }
            FocusPane::Members => {
                state.open_selected_member_actions();
                None
            }
            FocusPane::Messages => state.activate_selected_message_pane_item(),
        },
    }
}

fn handle_message_shortcut_action(
    state: &mut DashboardState,
    action: MessageShortcutAction,
) -> Option<AppCommand> {
    match action {
        MessageShortcutAction::CopyContent => {
            state.activate_message_action_kind(MessageActionKind::CopyContent)
        }
        MessageShortcutAction::OpenReactionPicker => {
            state.activate_message_action_kind(MessageActionKind::OpenReactionPicker)
        }
        MessageShortcutAction::Reply => {
            state.activate_message_action_kind(MessageActionKind::Reply)
        }
        MessageShortcutAction::OpenDeleteConfirmation => {
            state.activate_message_action_kind(MessageActionKind::OpenDeleteConfirmation)
        }
        MessageShortcutAction::Edit => state.activate_message_action_kind(MessageActionKind::Edit),
        MessageShortcutAction::OpenUrl => {
            state.activate_message_action_kind(MessageActionKind::OpenUrl)
        }
        MessageShortcutAction::ViewAttachment => {
            state.activate_message_action_kind(MessageActionKind::ViewAttachment)
        }
        MessageShortcutAction::ShowProfile => {
            state.activate_message_action_kind(MessageActionKind::ShowProfile)
        }
        MessageShortcutAction::OpenPinConfirmation => {
            state.activate_message_action_kind(MessageActionKind::OpenPinConfirmation)
        }
        MessageShortcutAction::OpenThread => {
            state.activate_message_action_kind(MessageActionKind::OpenThread)
        }
        MessageShortcutAction::ShowReactionUsers => {
            state.activate_message_action_kind(MessageActionKind::ShowReactionUsers)
        }
        MessageShortcutAction::OpenPollVotePicker => {
            state.activate_message_action_kind(MessageActionKind::OpenPollVotePicker)
        }
    }
}

pub(super) fn execute_ui_action(
    state: &mut DashboardState,
    focus: FocusPane,
    action: UiAction,
) -> Option<AppCommand> {
    if let Some(dashboard_action) = state
        .key_bindings()
        .dashboard_action_for_ui_action(action, focus)
    {
        return handle_dashboard_action(state, focus, dashboard_action);
    }

    match action {
        UiAction::ToggleGuildPane => state.toggle_pane_visibility(FocusPane::Guilds),
        UiAction::ToggleChannelPane => state.toggle_pane_visibility(FocusPane::Channels),
        UiAction::ToggleMemberPane => state.toggle_pane_visibility(FocusPane::Members),
        UiAction::OpenFocusedPaneAction => state.open_leader_actions_for_focused_target(),
        UiAction::OpenOptions => state.open_options_category_picker(),
        UiAction::ChannelSwitcher => state.open_channel_switcher(),
        UiAction::OpenDisplayOptions => {
            state.open_options_category_from_shortcut(OptionsCategoryShortcut::Display)
        }
        UiAction::OpenNotificationOptions => {
            state.open_options_category_from_shortcut(OptionsCategoryShortcut::Notifications)
        }
        UiAction::OpenVoiceOptions => {
            state.open_options_category_from_shortcut(OptionsCategoryShortcut::Voice)
        }
        UiAction::VoiceDeafen => state.toggle_voice_deafen(),
        UiAction::VoiceMute => state.toggle_voice_mute(),
        UiAction::VoiceLeave => return state.leave_current_voice_channel_command(),
        _ => {}
    }
    None
}

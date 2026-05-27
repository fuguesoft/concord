use crossterm::event::KeyEvent;

use crate::discord::AppCommand;
use crate::tui::keybindings::{KeyChord, KeyMapLookup, LeaderActionMenuAction};
use crate::tui::state::{ActiveModalPopupKind, DashboardState};

use super::execute_ui_action;

pub(super) fn handle_leader_key(state: &mut DashboardState, key: KeyEvent) -> Option<AppCommand> {
    if state.is_leader_action_mode() {
        return handle_leader_action_key(state, key);
    }

    if let Some(command) = handle_leader_keymap_key(state, key) {
        return command;
    }

    state.close_leader();

    None
}

fn handle_leader_keymap_key(
    state: &mut DashboardState,
    key: KeyEvent,
) -> Option<Option<AppCommand>> {
    let focus = state.focus();
    let lookup = state
        .key_bindings()
        .keymap_lookup_with_key(state.leader_keymap_prefix(), key);
    match lookup {
        Some(KeyMapLookup::Pending) => {
            let chord = state.key_bindings().keymap_chord_for_event(key);
            state.push_leader_keymap_key(chord);
            Some(None)
        }
        Some(KeyMapLookup::Action(action)) => {
            state.close_leader();
            Some(execute_ui_action(state, focus, action))
        }
        None if state.leader_keymap_prefix().len() > 1 => {
            state.close_leader();
            Some(None)
        }
        None => None,
    }
}

fn handle_leader_action_key(state: &mut DashboardState, key: KeyEvent) -> Option<AppCommand> {
    match state.key_bindings().leader_action_menu_action(key) {
        LeaderActionMenuAction::BackOrClose => {
            if state.is_active_modal_popup(ActiveModalPopupKind::MessageUrlPicker) {
                state.close_message_url_picker();
                return None;
            }
            if state.back_channel_leader_action() || state.back_guild_leader_action() {
                return None;
            }
            state.close_all_action_contexts();
            state.close_leader();
            None
        }
        LeaderActionMenuAction::Close => {
            state.close_all_action_contexts();
            state.close_leader();
            None
        }
        LeaderActionMenuAction::ActivateShortcut(shortcut) => {
            let (matched, command) = activate_leader_action_shortcut(state, shortcut);
            if !matched || !state.is_any_action_context_active() {
                state.close_all_action_contexts();
                state.close_leader();
            }
            command
        }
        LeaderActionMenuAction::UnknownClose => {
            state.close_all_action_contexts();
            state.close_leader();
            None
        }
    }
}

fn activate_leader_action_shortcut(
    state: &mut DashboardState,
    shortcut: KeyChord,
) -> (bool, Option<AppCommand>) {
    if leader_message_action_matches(state, shortcut) {
        return (true, state.activate_message_action_shortcut(shortcut));
    }
    if leader_channel_action_matches(state, shortcut) {
        return (true, state.activate_channel_action_shortcut(shortcut));
    }
    if leader_guild_action_matches(state, shortcut) {
        return (true, state.activate_guild_action_shortcut(shortcut));
    }
    if leader_member_action_matches(state, shortcut) {
        return (true, state.activate_member_action_shortcut(shortcut));
    }
    (false, None)
}

fn leader_message_action_matches(state: &DashboardState, shortcut: KeyChord) -> bool {
    if !state.is_message_action_context_active() {
        return false;
    }
    let actions = state.selected_message_action_items();
    action_shortcut_matches(
        state,
        &actions,
        shortcut,
        |key_bindings, actions, index| key_bindings.message_action_shortcuts(actions, index),
        |action| action.enabled,
    )
}

fn leader_channel_action_matches(state: &DashboardState, shortcut: KeyChord) -> bool {
    if !state.is_channel_leader_action_active() {
        return false;
    }
    if state.is_channel_action_threads_phase() {
        return indexed_shortcut_matches(
            state,
            shortcut,
            state.channel_action_thread_items().len(),
        );
    }
    if state.is_channel_action_mute_duration_phase() {
        return indexed_shortcut_matches(
            state,
            shortcut,
            state.selected_channel_mute_duration_items().len(),
        );
    }
    let actions = state.selected_channel_action_items();
    action_shortcut_matches(
        state,
        &actions,
        shortcut,
        |key_bindings, actions, index| key_bindings.channel_action_shortcuts(actions, index),
        |action| action.enabled,
    )
}

fn leader_guild_action_matches(state: &DashboardState, shortcut: KeyChord) -> bool {
    if !state.is_guild_leader_action_active() {
        return false;
    }
    if state.is_guild_action_mute_duration_phase() {
        return indexed_shortcut_matches(
            state,
            shortcut,
            state.selected_guild_mute_duration_items().len(),
        );
    }
    let actions = state.selected_guild_action_items();
    action_shortcut_matches(
        state,
        &actions,
        shortcut,
        |key_bindings, actions, index| key_bindings.guild_action_shortcuts(actions, index),
        |action| action.enabled,
    )
}

fn leader_member_action_matches(state: &DashboardState, shortcut: KeyChord) -> bool {
    if !state.is_member_leader_action_active() {
        return false;
    }
    let actions = state.selected_member_action_items();
    action_shortcut_matches(
        state,
        &actions,
        shortcut,
        |key_bindings, actions, index| key_bindings.member_action_shortcuts(actions, index),
        |action| action.enabled,
    )
}

fn action_shortcut_matches<A>(
    state: &DashboardState,
    actions: &[A],
    shortcut: KeyChord,
    shortcuts: impl Fn(&crate::tui::keybindings::KeyBindings, &[A], usize) -> Vec<KeyChord>,
    is_enabled: impl Fn(&A) -> bool,
) -> bool {
    state
        .key_bindings()
        .matching_action_shortcut_index(actions, shortcut, shortcuts, is_enabled)
        .is_some()
}

fn indexed_shortcut_matches(state: &DashboardState, shortcut: KeyChord, len: usize) -> bool {
    (0..len).any(|index| {
        state
            .key_bindings()
            .indexed_shortcut(index)
            .is_some_and(|candidate| shortcut.matches_char(candidate))
    })
}

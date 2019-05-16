//! # UI
//!
//! The UI module contains logic for matching keyboard and system events

use sdl2::event::Event;
use sdl2::mouse::MouseButton;

/// Action represents the possible actions that could result from an event
#[derive(Clone)]
pub enum Action {
    /// Quit indicates the app should quit in response to this event
    Quit,
    /// Toggle Fullscreen State
    ToggleFullscreen,
    /// ReRender indicates the app should re-render in response to this event (such as a window
    /// resize)
    ReRender,
    /// The app should switch its current image viewing preference of fitting the
    /// image to screen or displaying the actual size as actual size
    ToggleFit,
    /// Next indicates the app should move to the next image in response to this event
    Next,
    /// Prev indicates the app should move to the previous image in response to this event
    Prev,
    /// First indicates the app should move to the first image in response to this event
    First,
    /// Last indicates the app should move to the last image in response to this event
    Last,
    /// SkipForward advances the list of images by x%
    SkipForward,
    /// SkipBack rewinds the list of images by x%
    SkipBack,
    /// Copy indicates the app should copy the image in response to this event
    Copy,
    /// Move indicates the app should move the image in response to this event
    Move,
    /// Delete indicates the app should delete the image in response to this event
    Delete,
    /// Noop indicates the app should not respond to this event
    Noop,
}

/// State tracks events that will change the behaviour of future events. Such as key modifiers.
pub struct State {
    /// left_shift tracks whether or not the left shift key is pressed.
    pub left_shift: bool,
    /// right_shift tracks whether or not the right shift key is pressed.
    pub right_shift: bool,
    /// render_infobar determines whether or not the info bar should be rendered.
    pub render_infobar: bool,
    /// render_help determines whether or not the help info should be rendered.
    pub render_help: bool,
    /// Should the image shown be shown in actual pixel dimensions
    pub actual_size: bool,
    /// Tracks fullscreen state of app.
    pub fullscreen: bool,
    /// last_action records the last action performed. Used for repeating that action
    pub last_action: Action,
}

impl State {
    /// update_last_action takes an action, sets the last_action to said action, and returns the Action
    fn process_action(&mut self, a: Action) -> Action {
        match a {
            Action::Quit | Action::ToggleFullscreen | Action::ReRender => {
                self.last_action = Action::Noop;
                a
            }
            _ => {
                self.last_action = a.clone();
                a
            }
        }
    }
}

/// event_action returns which action should be performed in response to this event
pub fn event_action(state: &mut State, event: &Event) -> Action {
    // Bring variants in function namespace for reduced typing.
    use sdl2::event::WindowEvent::*;
    use sdl2::keyboard::Keycode::*;

    match event {
        Event::Quit { .. } => Action::Quit,

        Event::KeyDown {
            keycode: Some(k), ..
        } => match k {
            C => state.process_action(Action::Copy),
            D | Delete => state.process_action(Action::Delete),
            F | F11 => state.process_action(Action::ToggleFullscreen),
            G => {
                if state.left_shift || state.right_shift {
                    state.process_action(Action::Last)
                } else {
                    state.process_action(Action::First)
                }
            }
            H => {
                state.render_help = !state.render_help;
                state.process_action(Action::ReRender)
            }
            J | Right => state.process_action(Action::Next),
            K | Left => state.process_action(Action::Prev),
            M => state.process_action(Action::Move),
            Q | Escape => Action::Quit,
            T => {
                state.render_infobar = !state.render_infobar;
                state.process_action(Action::ReRender)
            }

            W | PageUp => state.process_action(Action::SkipForward),
            B | PageDown => state.process_action(Action::SkipBack),
            Z => state.process_action(Action::ToggleFit),
            Home => state.process_action(Action::First),
            End => state.process_action(Action::Last),
            LShift => {
                state.left_shift = true;
                Action::Noop
            }
            RShift => {
                state.right_shift = true;
                Action::Noop
            }
            _ => Action::Noop,
        },

        Event::KeyUp {
            keycode: Some(k), ..
        } => match k {
            LShift => {
                state.left_shift = false;
                Action::Noop
            }
            RShift => {
                state.right_shift = false;
                Action::Noop
            }
            _ => Action::Noop,
        },

        Event::Window { win_event, .. } => match win_event {
            // Exposed: Rerender if the window was not changed by us.
            Exposed | Resized(..) | SizeChanged(..) | Maximized => Action::ReRender,
            _ => Action::Noop,
        },

        Event::MouseButtonUp { mouse_btn: btn, .. } => match btn {
            MouseButton::Left => state.process_action(Action::ToggleFit),
            _ => Action::Noop,
        },
        _ => Action::Noop,
    }
}

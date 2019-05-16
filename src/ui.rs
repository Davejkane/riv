//! # UI
//!
//! The UI module contains logic for matching keyboard and system events

use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
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
    match event {
        Event::Quit { .. }
        | Event::KeyDown {
            keycode: Some(Keycode::Escape),
            ..
        }
        | Event::KeyDown {
            keycode: Some(Keycode::Q),
            ..
        } => state.process_action(Action::Quit),
        Event::KeyDown {
            keycode: Some(Keycode::F),
            ..
        }
        | Event::KeyDown {
            keycode: Some(Keycode::F11),
            ..
        } => state.process_action(Action::ToggleFullscreen),
        Event::Window {
            win_event: WindowEvent::Resized(_, _),
            ..
        }
        | Event::Window {
            win_event: WindowEvent::SizeChanged(_, _),
            ..
        }
        // Rerender if the window was not changed by us.
        | Event::Window {
            win_event: WindowEvent::Exposed,
            ..
        }
        | Event::Window {
            win_event: WindowEvent::Maximized,
            ..
        } => state.process_action(Action::ReRender),
        Event::KeyDown {
            keycode: Some(Keycode::Z),
            ..
        }
        | Event::MouseButtonUp {
            mouse_btn: MouseButton::Left,
            ..
        } => state.process_action(Action::ToggleFit),
        Event::KeyDown {
            keycode: Some(Keycode::Right),
            ..
        }
        | Event::KeyDown {
            keycode: Some(Keycode::J),
            ..
        } => state.process_action(Action::Next),
        Event::KeyDown {
            keycode: Some(Keycode::Left),
            ..
        }
        | Event::KeyDown {
            keycode: Some(Keycode::K),
            ..
        } => state.process_action(Action::Prev),
        Event::KeyDown {
            keycode: Some(Keycode::G),
            ..
        } => {
            if state.left_shift || state.right_shift {
                state.process_action(Action::Last)
            } else {
                state.process_action(Action::First)
            }
        }
        Event::KeyDown {
            keycode: Some(Keycode::End),
            ..
        } => state.process_action(Action::Last),
        Event::KeyDown {
            keycode: Some(Keycode::Home),
            ..
        } => state.process_action(Action::First),
        Event::KeyDown {
            keycode: Some(Keycode::C),
            ..
        } => state.process_action(Action::Copy),
        Event::KeyDown {
            keycode: Some(Keycode::M),
            ..
        } => state.process_action(Action::Move),
        Event::KeyDown {
            keycode: Some(Keycode::W),
            ..
        }
        | Event::KeyDown {
            keycode: Some(Keycode::PageUp),
            ..
        } => state.process_action(Action::SkipForward),
        Event::KeyDown {
            keycode: Some(Keycode::B),
            ..
        }
        | Event::KeyDown {
            keycode: Some(Keycode::PageDown),
            ..
        } => state.process_action(Action::SkipBack),
        Event::KeyDown {
            keycode: Some(Keycode::D),
            ..
        }
        | Event::KeyDown {
            keycode: Some(Keycode::Delete),
            ..
        } => state.process_action(Action::Delete),
        Event::KeyDown {
            keycode: Some(Keycode::T),
            ..
        } => {
            state.render_infobar = !state.render_infobar;
            state.process_action(Action::ReRender)
        }
        Event::KeyDown {
            keycode: Some(Keycode::H),
            ..
        } => {
            state.render_help = !state.render_help;
            state.process_action(Action::ReRender)
        }
        Event::KeyDown {
            keycode: Some(Keycode::Period),
            ..
        } => {
            state.last_action.clone()
        }
        Event::KeyDown {
            keycode: Some(Keycode::LShift),
            ..
        } => {
            state.left_shift = true;
            Action::Noop
        }
        Event::KeyDown {
            keycode: Some(Keycode::RShift),
            ..
        } => {
            state.right_shift = true;
            Action::Noop
        }
        Event::KeyUp {
            keycode: Some(Keycode::LShift),
            ..
        } => {
            state.left_shift = false;
            Action::Noop
        }
        Event::KeyUp {
            keycode: Some(Keycode::RShift),
            ..
        } => {
            state.right_shift = false;
            Action::Noop
        }
        _ => Action::Noop,
    }
}

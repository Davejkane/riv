//! # UI
//!
//! The UI module contains logic for matching keyboard and system events

use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;

/// Action represents the possible actions that could result from an event
pub enum Action {
    /// Quit indicates the app should quit in response to this event
    Quit,
    /// ReRender indicates the app should re-render in response to this event (such as a window resize)
    ReRender,
    /// Next indicates the app should move to the next image in response to this event
    Next,
    /// Prev indicates the app should move to the previous image in response to this event
    Prev,
    /// First indicates the app should move to the first image in response to this event
    First,
    /// Last indicates the app should move to the last image in response to this event
    Last,
    /// Copy indicates the app should copy the image in response to this event
    Copy,
    /// Move indicates the app should move the image in response to this event
    Move,
    /// Noop indicates the app should not respond to this event
    Noop,
}

/// State tracks events that will change the behaviour of future events. Such as key modifiers.
pub struct State {
    /// left_shift tracks whether or not the left shift key is pressed
    pub left_shift: bool,
    /// right_shift tracks whether or not the right shift key is pressed
    pub right_shift: bool,
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
        } => Action::Quit,
        Event::Window {
            win_event: WindowEvent::Resized(_, _),
            ..
        }
        | Event::Window {
            win_event: WindowEvent::SizeChanged(_, _),
            ..
        }
        | Event::Window {
            win_event: WindowEvent::Maximized,
            ..
        } => Action::ReRender,
        Event::KeyDown {
            keycode: Some(Keycode::Right),
            ..
        }
        | Event::KeyDown {
            keycode: Some(Keycode::J),
            ..
        } => Action::Next,
        Event::KeyDown {
            keycode: Some(Keycode::Left),
            ..
        }
        | Event::KeyDown {
            keycode: Some(Keycode::K),
            ..
        } => Action::Prev,
        Event::KeyDown {
            keycode: Some(Keycode::G),
            ..
        } => {
            if state.left_shift || state.right_shift {
                Action::Last
            } else {
                Action::First
            }
        }
        Event::KeyDown {
            keycode: Some(Keycode::End),
            ..
        } => Action::Last,
        Event::KeyDown {
            keycode: Some(Keycode::Home),
            ..
        } => Action::First,
        Event::KeyDown {keycode: Some(Keycode::C), ..} => Action::Copy,
        Event::KeyDown {
            keycode: Some(Keycode::M),
            ..
        } => Action::Move,
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

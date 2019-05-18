//! # UI
//!
//! The UI module contains logic for matching keyboard and system events

use sdl2::event::Event;
use sdl2::mouse::MouseButton;

/// Action represents the possible actions that could result from an event
pub enum Action<'a> {
    /// Quit indicates the app should quit in response to this event
    Quit,
    /// Toggle Fullscreen State
    ToggleFullscreen,
    /// ReRender indicates the app should re-render in response to this event (such as a window
    /// resize)
    ReRender,
    /// Switches modes from normal to command mode to enter queries such as "newglob"/"ng"
    SwitchCommandMode,
    /// Indicates user hit the backspace, program input should be truncated accordinly
    Backspace,
    /// User entered input from the keyboard
    KeyboardInput(&'a str),
    /// switches modes back to normal mode
    SwitchNormalMode,
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

/// Modal setting for Program, this dictates the commands that are available to the user
#[derive(PartialEq)]
pub enum Mode {
    /// Default mode, allows the removal, traversal, move, and copy of images
    Normal,
    /// Mode that is built off of user input, allows switching the current glob
    Command,
    /// Terminate condition, if this mode is set the program will stop execution
    Exit,
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
    /// current mode of the application, changes how input is interpreted
    pub mode: Mode,
}

/// event_action returns which action should be performed in response to this event
pub fn process_normal_mode<'a>(state: &mut State, event: &Event) -> Action<'a> {
    // Bring variants in function namespace for reduced typing.
    use sdl2::event::WindowEvent::*;
    use sdl2::keyboard::Keycode::*;

    match event {
        Event::Quit { .. } => Action::Quit,

        Event::KeyDown {
            keycode: Some(k), ..
        } => match k {
            C => Action::Copy,
            D | Delete => Action::Delete,
            F | F11 => Action::ToggleFullscreen,
            G => {
                if state.left_shift || state.right_shift {
                    Action::Last
                } else {
                    Action::First
                }
            }
            H => {
                state.render_help = !state.render_help;
                Action::ReRender
            }
            J | Right => Action::Next,
            K | Left => Action::Prev,
            M => Action::Move,
            Q | Escape => Action::Quit,
            T => {
                state.render_infobar = !state.render_infobar;
                Action::ReRender
            }

            W | PageUp => Action::SkipForward,
            B | PageDown => Action::SkipBack,
            Z => Action::ToggleFit,
            Home => Action::First,
            End => Action::Last,
            Semicolon => {
                if state.left_shift || state.right_shift {
                    Action::SwitchCommandMode
                } else {
                    // placeholder for any feature that uses ';'
                    Action::Noop
                }
            }
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
            MouseButton::Left => Action::ToggleFit,
            _ => Action::Noop,
        },
        _ => Action::Noop,
    }
}

/// Processes event information for Command mode, and returns them as Actions
pub fn process_command_mode<'a>(event: &'a Event) -> Action<'a> {
    use sdl2::event::WindowEvent;
    use sdl2::keyboard::Keycode;

    match event {
        Event::TextInput { text, .. } => Action::KeyboardInput(text),
        // Handle backspace, escape, and returns
        Event::KeyDown {
            keycode: Some(code),
            ..
        } => match code {
            Keycode::Backspace => Action::Backspace,
            Keycode::Escape => Action::SwitchNormalMode,
            // User is done entering input
            Keycode::Return | Keycode::Return2 | Keycode::KpEnter => Action::SwitchNormalMode,
            _ => Action::Noop,
        },
        Event::Window { win_event, .. } => match win_event {
            // Exposed: Rerender if the window was not changed by us.
            WindowEvent::Exposed
            | WindowEvent::Resized(..)
            | WindowEvent::SizeChanged(..)
            | WindowEvent::Maximized => Action::ReRender,
            _ => Action::Noop,
        },
        _ => Action::Noop,
    }
}

//! # UI
//!
//! The UI module contains logic for matching keyboard and system events

use sdl2::event::Event;
use sdl2::keyboard::Keycode::{
    Delete, End, Escape, Home, Kp0, Kp1, Kp2, Kp3, Kp4, Kp5, Kp6, Kp7, Kp8, Kp9, Left, Num0, Num1,
    Num2, Num3, Num4, Num5, Num6, Num7, Num8, Num9, PageDown, PageUp, Period, Right, B, C, D, F,
    F11, G, H, J, K, M, Q, T, W, Z,
};
use sdl2::mouse::MouseButton;

/// Action represents the possible actions that could result from an event
#[derive(Clone)]
pub enum Action<'a> {
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
    /// Put number on Screen for Numeric operations
    NumericOp(Token),
    /// Copy multiple images
    CopyMultiple(&'a Vec<String>),
    /// Cancel operation
    Cancel,
    /// Noop indicates the app should not respond to this event
    Noop,
}

/// State tracks events that will change the behaviour of future events. Such as key modifiers.
pub struct State<'a> {
    /// render_infobar determines whether or not the info bar should be rendered.
    pub render_infobar: bool,
    /// render_help determines whether or not the help info should be rendered.
    pub render_help: bool,
    /// Should the image shown be shown in actual pixel dimensions
    pub actual_size: bool,
    /// Tracks fullscreen state of app.
    pub fullscreen: bool,
    /// last_action records the last action performed. Used for repeating that action
    pub last_action: Action<'a>,
    /// Input Event processing state
    pub input_state: Mode,
    /// Each character of input text for extended operation
    pub unprocessed_input: Vec<String>,
}

impl<'a> Default for State<'a> {
    fn default() -> Self {
        Self {
            render_infobar: true,
            render_help: false,
            actual_size: false,
            fullscreen: false,
            last_action: Action::Noop,
            input_state: Mode::Normal,
            unprocessed_input: Vec::new(),
        }
    }
}

impl<'a> State<'a> {
    /// update_last_action takes an action, sets the last_action to said action, and returns the Action
    fn process_action(&mut self, a: Action<'a>) -> Action {
        match a {
            Action::Quit | Action::ReRender => a,
            _ => {
                self.last_action = a.clone();
                a
            }
        }
    }
}

/// Pieces of info parsed out of each character of gotten input
#[derive(Clone, Debug)]
pub enum Token {
    /// A digit [0-9]
    Digit(String),
    /// Operation to perform on digits
    Operation(String),
}

/// The state of the Input processor. Determines
/// Different actions are taken on the same key depending on the StateEntry
#[derive(Debug)]
pub enum Mode {
    /// Process input normally
    Normal,
    /// Process when expecting numeric commands
    Numeric,
}

/// Process Numeric keypresses
pub fn process_numeric<'a>(state: &'a mut State, event: &Event) -> Action<'a> {
    use sdl2::event::WindowEvent::*;
    use sdl2::keyboard::Mod;

    println!("inside numeric");

    match event {
        // Still quit out of program if closed explicitly
        Event::Quit { .. } => Action::Quit,

        Event::TextInput { text: t, .. } => match t.as_ref() {
            "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" => {
                //let token = Token::Number(n);
                state.unprocessed_input.push(t.to_owned());
                println!("got digit: {}", t);
                Action::NumericOp(Token::Digit(t.to_string()))
            }
            "c" => {
                state.input_state = Mode::Normal;
                println!("Switched state to : {:?}", state.input_state);
                Action::CopyMultiple(&state.unprocessed_input)
            }
            _ => {
                println!("got character: {}", t);
                Action::Noop
            }
        },

        Event::Window { win_event, .. } => match win_event {
            // Exposed: Rerender if the window was not changed by us.
            Exposed | Resized(..) | SizeChanged(..) | Maximized => Action::ReRender,
            _ => Action::Noop,
        },

        Event::KeyDown {
            keycode: Some(k),
            keymod: m,
            ..
        } => match (k, m) {
            (_, &Mod::NOMOD) => match k {
                Escape => {
                    state.input_state = Mode::Normal;
                    println!("Switched state to : {:?}", state.input_state);
                    Action::Cancel
                }
                _ => Action::Noop,
            },
            (_, _) => Action::Noop,
        },

        Event::MouseButtonUp { mouse_btn: btn, .. } => match btn {
            MouseButton::Left => Action::ToggleFit,
            _ => Action::Noop,
        },
        _ => Action::Noop,
    }
}

// Scannerless parsing

fn process_normal<'a>(state: &'a mut State, event: &Event) -> Action<'a> {
    use sdl2::event::WindowEvent::*;
    use sdl2::keyboard::Mod;

    println!("Inside normal");
    match event {
        Event::Quit { .. } => Action::Quit,

        Event::KeyDown {
            keycode: Some(k),
            keymod: m,
            ..
        } => match (k, m) {
            (_, &Mod::NOMOD) => match k {
                // NOMOD are numbers on numpad
                Num1 | Num2 | Num3 | Num4 | Num5 | Num6 | Num7 | Num8 | Num9 | Num0 | Kp0 | Kp1
                | Kp2 | Kp3 | Kp4 | Kp5 | Kp6 | Kp7 | Kp8 | Kp9 => {
                    state.input_state = Mode::Numeric;
                    Action::Noop
                }

                C => state.process_action(Action::Copy),
                D | Delete => state.process_action(Action::Delete),
                F | F11 => state.process_action(Action::ToggleFullscreen),
                G => state.process_action(Action::First),
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
                Period => state.last_action.clone(),
                Home => state.process_action(Action::First),
                End => state.process_action(Action::Last),
                _ => Action::Noop,
            },
            (_, &Mod::LSHIFTMOD) => match k {
                G => state.process_action(Action::Last),
                _ => Action::Noop,
            },
            (_, &Mod::RSHIFTMOD) => match k {
                G => state.process_action(Action::Last),
                _ => Action::Noop,
            },
            (_, _) => Action::Noop,
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

/// event_action returns which action should be performed in response to this event
pub fn event_action<'a>(state: &'a mut State, event: &Event) -> Action<'a> {
    // Bring variants in function namespace for reduced typing.
    //println!("{:?}", event);

    match state.input_state {
        Mode::Normal => process_normal(state, event),
        Mode::Numeric => process_numeric(state, event),
    }
}

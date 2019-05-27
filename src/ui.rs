//! # UI
//!
//! The UI module contains logic for matching keyboard and system events

use sdl2::event::Event;
use sdl2::mouse::MouseButton;
use std::time::Instant;

/// Action represents the possible actions that could result from an event
#[derive(Debug, Clone)]
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
    /// Centres the image
    CenterImage,
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
    /// Zoom zooms in or out depending on the ZoomAction variant
    Zoom(ZoomAction),
    /// Pan pans the picture in the direction of the PanAction variant
    Pan(PanAction),
    /// Copy indicates the app should copy the image in response to this event
    Copy,
    /// Move indicates the app should move the image in response to this event
    Move,
    /// Delete indicates the app should delete the image in response to this event
    Delete,
    /// Noop indicates the app should not respond to this event
    Noop,
}

struct MultiAction<'a> {
    action: &'a Action<'a>,
    times: u32,
}


/// ZoomAction contains the variants of a possible zoom action. In | Out
#[derive(Debug, Clone)]
pub enum ZoomAction {
    /// In zooms in
    In,
    /// Out zooms out
    Out,
}

/// PanAction contains the variants of a possible pan action. Left | Right | Up | Down
#[derive(Debug, Clone)]
pub enum PanAction {
    /// Left pans left
    Left,
    /// Right pans right
    Right,
    /// Up pans up
    Up,
    /// Down pans down
    Down,
}

/// Modal setting for Program, this dictates the commands that are available to the user
#[derive(Debug, PartialEq, Clone)]
pub enum Mode {
    /// Default mode, allows the removal, traversal, move, and copy of images
    Normal,
    /// Mode that is built off of user input, allows switching the current glob
    /// string is the input to display on the infobar
    Command(String),
    /// Mode that is meant to display errors to the user through the infobar
    /// string is the input to display on the infobar
    Error(String),
    /// Mode that is used to display success messages
    Success(String),
    /// Terminate condition, if this mode is set the program will stop execution
    Exit,
}

/// Determines which form of help message to render
#[derive(PartialEq, Clone)]
pub enum HelpRender {
    /// Should not be rendered
    None,
    /// Should render normal mode help
    Normal,
    /// Should render command mode help
    Command,
}

/// State tracks events that will change the behaviour of future events. Such as key modifiers.
pub struct State<'a> {
    /// render_infobar determines whether or not the info bar should be rendered.
    pub render_infobar: bool,
    /// render_help determines whether or not the help info should be rendered.
    pub render_help: HelpRender,
    /// Tracks fullscreen state of app.
    pub fullscreen: bool,
    /// current mode of the application, changes how input is interpreted
    pub mode: Mode,
    /// last_action records the last action performed. Used for repeating that action
    pub last_action: Action<'a>,
    /// scale represents the scale of the image with 1.0 being the actual size of the image
    pub scale: f32,
    /// pan_x is the degree of pan in the x axis
    pub pan_x: f32,
    /// pan_y is the degree of pan in the y axis
    pub pan_y: f32,
    /// The time, from which to do a re-render will be base on.
    /// Use to clear infobar messages after inactivity
    pub rerender_time: Option<Instant>,
    /// Unprocessed input from user
    pub current_input: String,
}

impl<'a> Default for State<'a> {
    fn default() -> Self {
        Self{
            render_infobar: true,
            render_help: false,
            fullscreen: false,
            mode: Mode::Normal,
            last_action: Action::Noop,
            scale: 1.0,
            pan_x: 0.0,
            pan_y: 0.0,
            current_input: String::new(),
        }
    }
}

impl<'a> State<'a> {
    /// update_last_action takes an action, sets the last_action to said action, and returns the Action
    fn process_action(&mut self, a: Action<'a>) -> Action<'a> {
        let times = if self.current_input.len() == 0 {
            1
        }
        else {
            self.current_input.parse::<u64>().expect("invalid number")
        };
        println!("perform {:?} {:?} times", a, times);
        match a {
            Action::Quit | Action::ReRender => a,
            _ => {
                self.last_action = a.clone();
                a
            }
        }
    }
}

/// event_action returns which action should be performed in response to this event
pub fn process_normal_mode<'a>(state: &mut State<'a>, event: &Event) -> Action<'a> {
    // Bring variants in function namespace for reduced typing.
    use sdl2::event::WindowEvent::*;
    use sdl2::keyboard::Keycode::*;
    use sdl2::keyboard::Mod;

    match event {
        Event::Quit { .. } => Action::Quit,

        Event::TextInput { text, .. } => match text.as_str() {
            // Number of times to repeat operation
            "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" => {
                state.current_input.push_str(text);
                Action::Noop
            }
            "c" => state.process_action(Action::Copy),
            "d" => state.process_action(Action::Delete),
            "f" => state.process_action(Action::ToggleFullscreen),
            "g" => state.process_action(Action::First),
            "G" => state.process_action(Action::Last),
            "?" => {
                match state.render_help {
                    HelpRender::Normal => state.render_help = HelpRender::None,
                    _ => state.render_help = HelpRender::Normal,
                }
                state.process_action(Action::ReRender)
            }
            "H" => state.process_action(Action::Pan(PanAction::Left)),
            "i" => state.process_action(Action::Zoom(ZoomAction::In)),
            "j" => state.process_action(Action::Next),
            "J" => state.process_action(Action::Pan(PanAction::Down)),
            "k" => state.process_action(Action::Prev),
            "K" => state.process_action(Action::Pan(PanAction::Up)),
            "L" => state.process_action(Action::Pan(PanAction::Right)),
            "m" => state.process_action(Action::Move),
            "o" => state.process_action(Action::Zoom(ZoomAction::Out)),
            "q" => Action::Quit,
            "t" => {
                state.render_infobar = !state.render_infobar;
                state.process_action(Action::ReRender)
            }
            "w" => state.process_action(Action::SkipForward),
            "b" => state.process_action(Action::SkipBack),
            "z" => state.process_action(Action::ToggleFit),
            "Z" => state.process_action(Action::CenterImage),
            ":" => Action::SwitchCommandMode,
            _ => Action::Noop,
        },

        Event::KeyDown {
            keycode: Some(k),
            keymod: m,
            ..
        } => match (k, m) {
            (k, &Mod::LSHIFTMOD) | (k, &Mod::RSHIFTMOD) => match k {
                Left => state.process_action(Action::Pan(PanAction::Left)),
                Right => state.process_action(Action::Pan(PanAction::Right)),
                Up => state.process_action(Action::Pan(PanAction::Up)),
                Down => state.process_action(Action::Pan(PanAction::Down)),
                _ => Action::Noop,
            },
            (k, &Mod::NOMOD) | (k, _) => match k {
                Delete => state.process_action(Action::Delete),
                F11 => state.process_action(Action::ToggleFullscreen),
                Escape => Action::Quit,
                PageUp => state.process_action(Action::SkipForward),
                PageDown => state.process_action(Action::SkipBack),
                Home => Action::First,
                End => Action::Last,
                Period => state.last_action.clone(),
                Right => state.process_action(Action::Next),
                Left => state.process_action(Action::Prev),
                Up => state.process_action(Action::Zoom(ZoomAction::In)),
                Down => state.process_action(Action::Zoom(ZoomAction::Out)),
                _ => Action::Noop,
            },
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

/// Processes event information for Command mode, and returns them as Actions
pub fn process_command_mode(event: &Event) -> Action {
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

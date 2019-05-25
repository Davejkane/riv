//! # UI
//!
//! The UI module contains logic for matching keyboard and system events

use sdl2::event::Event;
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

/// ZoomAction contains the variants of a possible zoom action. In | Out
#[derive(Clone)]
pub enum ZoomAction {
    /// In zooms in
    In,
    /// Out zooms out
    Out,
}

/// PanAction contains the variants of a possible pan action. Left | Right | Up | Down
#[derive(Clone)]
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
#[derive(PartialEq, Clone)]
pub enum Mode {
    /// Default mode, allows the removal, traversal, move, and copy of images
    Normal,
    /// Mode that is built off of user input, allows switching the current glob
    /// string is the input to display on the infobar
    Command(String),
    /// Mode that is meant to display errors to the user through the infobar
    /// string is the input to display on the infobar
    Error(String),
    /// Terminate condition, if this mode is set the program will stop execution
    Exit,
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
    /// current mode of the application, changes how input is interpreted
    pub mode: Mode,
    /// last_action records the last action performed. Used for repeating that action
    pub last_action: Action<'a>,
    /// scale represents the scale of the image with 1.0 being the actual size of the image
    pub scale: f32,
}

impl<'a> State<'a> {
    /// update_last_action takes an action, sets the last_action to said action, and returns the Action
    fn process_action(&mut self, a: Action<'a>) -> Action<'a> {
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

    match event {
        Event::Quit { .. } => Action::Quit,

        Event::TextInput { text, .. } => match text.as_str() {
            "c" => state.process_action(Action::Copy),
            "d" => state.process_action(Action::Delete),
            "f" => state.process_action(Action::ToggleFullscreen),
            "g" => state.process_action(Action::First),
            "G" => state.process_action(Action::Last),
            "?" => {
                state.render_help = !state.render_help;
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
            ":" => Action::SwitchCommandMode,
            _ => Action::Noop,
        },

        Event::KeyDown {
            keycode: Some(k), ..
        } => match k {
            Delete => state.process_action(Action::Delete),
            F11 => state.process_action(Action::ToggleFullscreen),
            Right => state.process_action(Action::Next),
            Left => state.process_action(Action::Prev),
            Up => state.process_action(Action::Zoom(ZoomAction::In)),
            Down => state.process_action(Action::Zoom(ZoomAction::Out)),
            Escape => Action::Quit,
            PageUp => state.process_action(Action::SkipForward),
            PageDown => state.process_action(Action::SkipBack),
            Home => Action::First,
            End => Action::Last,
            Period => state.last_action.clone(),
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

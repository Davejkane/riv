//! # UI
//!
//! The UI module contains logic for matching keyboard and system events

use sdl2::event::Event;
use sdl2::keyboard::Mod;
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
    /// Indicates user hit the backspace, program input should be truncated accordingly
    Backspace,
    /// User entered input from the keyboard
    KeyboardInput(&'a str),
    /// switches modes back to normal mode
    SwitchNormalMode,
    /// Switches to MultiNormalMode for bulk actions
    SwitchMultiNormalMode,
    /// The app should switch its current image viewing preference of fitting the
    /// image to screen or displaying the actual size as actual size
    ToggleFit,
    /// Centres the image
    CenterImage,
    /// Flip the image horizontally
    FlipHorizontal,
    /// Flip the image vertically
    FlipVertical,
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
    /// Which direction to rotate image
    Rotate(RotationDirection),
    /// Pan pans the picture in the direction of the PanAction variant
    Pan(PanAction),
    /// Copy indicates the app should copy the image in response to this event
    Copy,
    /// Move indicates the app should move the image in response to this event
    Move,
    /// Delete indicates the app should delete the image in response to this event
    Delete,
    /// Trash indicates the app should move the image to a trash folder
    Trash,
    /// Noop indicates the app should not respond to this event
    Noop,
}

/// Direction to rotate image
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RotationDirection {
    /// Instruct to rotate image clockwise
    Clockwise,
    /// Instruct to rotate image counterclockwise
    CounterClockwise,
}

impl<'a> Default for Action<'a> {
    fn default() -> Self {
        Action::Noop
    }
}

/// Actions to perform while in MultiNormal Mode
#[derive(Clone, Debug)]
pub enum MultiNormalAction<'a> {
    /// Rerender screen
    ReRender,
    /// Keep listening for input. Update display with current input
    MoreInput,
    /// Done getting Input. Repeat the command the specified times
    Repeat(ProcessAction<'a>),
    /// Switch back to normal mode
    SwitchBackNormalMode,
    /// Cancels input for entering how many times to repeat
    /// Switches back to Normal mode as well.
    Cancel,
    /// Notify to quit out of program
    Quit,
    /// Do not respond to event
    Noop,
}

impl<'a> From<ProcessAction<'a>> for MultiNormalAction<'a> {
    fn from(item: ProcessAction<'a>) -> Self {
        MultiNormalAction::Repeat(item)
    }
}

impl<'a> From<Action<'a>> for MultiNormalAction<'a> {
    fn from(item: Action<'a>) -> Self {
        MultiNormalAction::Repeat(ProcessAction::new(item, 1))
    }
}

impl<'a> From<(Action<'a>, usize)> for MultiNormalAction<'a> {
    fn from(item: (Action<'a>, usize)) -> Self {
        MultiNormalAction::Repeat((item.0, item.1).into())
    }
}

/// Perform an Action `times` times
#[derive(Clone, Debug)]
pub struct ProcessAction<'a> {
    /// The action to perform
    pub action: Action<'a>,
    /// The amount of times to perform
    pub times: usize,
}

impl<'a> From<Action<'a>> for ProcessAction<'a> {
    fn from(item: Action<'a>) -> Self {
        ProcessAction::new(item, 1)
    }
}

impl<'a> From<(Action<'a>, usize)> for ProcessAction<'a> {
    fn from(item: (Action<'a>, usize)) -> Self {
        ProcessAction::new(item.0, item.1)
    }
}

impl<'a> ProcessAction<'a> {
    fn new(action: Action<'a>, times: usize) -> Self {
        Self { action, times }
    }
}

impl<'a> Default for ProcessAction<'a> {
    fn default() -> Self {
        Self {
            action: Action::Noop,
            times: 1,
        }
    }
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
    /// Normal mode is switched to receiving the amount of times to perform
    /// the same action
    MultiNormal,
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

/// Storage for state across functions
pub struct Register<'a> {
    /// Current action to perform later
    pub cur_action: ProcessAction<'a>,
}

impl<'a> Default for Register<'a> {
    fn default() -> Self {
        Self {
            cur_action: ProcessAction::new(Action::Noop, 1),
        }
    }
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
    pub last_action: ProcessAction<'a>,
    /// scale represents the scale of the image with 1.0 being the actual size of the image
    pub scale: f32,
    /// pan_x is the degree of pan in the x axis
    pub pan_x: f32,
    /// pan_y is the degree of pan in the y axis
    pub pan_y: f32,
    /// Image is flipped horizontally from original state
    pub flip_horizontal: bool,
    /// Image is flipped vertically from original state
    pub flip_vertical: bool,
    /// Angle to rotate original image at
    /// Only supports 90 degree increments specified in `RotAngle` enum
    pub rot_angle: RotAngle,
    /// The time, from which to do a re-render will be base on.
    /// Use to clear infobar messages after inactivity
    pub rerender_time: Option<Instant>,
    /// Store
    pub register: Register<'a>,
}

/// Rotation angle for image
pub enum RotAngle {
    /// 0 degree rotation
    Up,
    /// 90 degree rotation
    Right,
    /// 180 degree rotation
    Down,
    /// 270 degree rotation
    Left,
}

impl RotAngle {
    /// Next state of rotation when rotated clockwise
    pub fn rot_clockwise(&self) -> RotAngle {
        match self {
            RotAngle::Up => RotAngle::Right,
            RotAngle::Right => RotAngle::Down,
            RotAngle::Down => RotAngle::Left,
            RotAngle::Left => RotAngle::Up,
        }
    }
    /// Next state of rotation when rotated counterclockwise
    pub fn rot_clockclockwise(&self) -> RotAngle {
        match self {
            RotAngle::Up => RotAngle::Left,
            RotAngle::Left => RotAngle::Down,
            RotAngle::Down => RotAngle::Right,
            RotAngle::Right => RotAngle::Up,
        }
    }
}

impl<'a> Default for State<'a> {
    fn default() -> Self {
        Self {
            render_infobar: true,
            render_help: HelpRender::None,
            fullscreen: false,
            mode: Mode::Normal,
            last_action: ProcessAction::default(),
            scale: 1.0,
            pan_x: 0.0,
            pan_y: 0.0,
            flip_horizontal: false,
            flip_vertical: false,
            rot_angle: RotAngle::Up,
            rerender_time: None,
            register: Register {
                ..Default::default()
            },
        }
    }
}

impl<'a> State<'a> {
    /// Increases zoom scale. Does not render image
    pub fn zoom_in(&mut self, times: usize) {
        let zoom_factor: f32 = 1.1;
        let zoom_times = cap_zoom_times(times);

        self.scale *= zoom_factor.powi(zoom_times);
    }

    /// Decreases zoom scale. Does not render image
    pub fn zoom_out(&mut self, times: usize) {
        let zoom_factor: f32 = 1.1;
        let zoom_times = cap_zoom_times(times);

        self.scale /= zoom_factor.powi(zoom_times);
    }
}

impl<'a> State<'a> {
    /// update_last_action takes an action, sets the last_action to said action, and returns the Action
    pub fn process_action(&mut self, pa: ProcessAction<'a>) -> ProcessAction<'a> {
        match &pa {
            ProcessAction { action: a, .. } => match a {
                Action::Noop | Action::Quit | Action::ReRender | Action::SwitchMultiNormalMode => {}
                _ => {
                    self.last_action = pa.clone();
                }
            },
        }
        pa
    }
}

/// Process SDL2 events while getting number of times to repeat action
pub fn process_multi_normal_mode<'a>(
    state: &mut State<'a>,
    event: &Event,
) -> MultiNormalAction<'a> {
    use sdl2::event::WindowEvent::*;
    use sdl2::keyboard::Keycode::*;

    let times = state.register.cur_action.times;
    match event {
        Event::Quit { .. } => MultiNormalAction::Quit,

        Event::TextInput { text, .. } => match text.as_str() {
            // Number of times to repeat operation
            "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" => {
                let previous_count = state.register.cur_action.times;
                // Safe to unwrap as only digits were matched
                let next_digit = text.parse::<usize>().unwrap();
                // Cap at highest possible value if overflow would occur
                let new_count = (previous_count.saturating_mul(10)).saturating_add(next_digit);
                // Save new count
                state.register.cur_action.times = new_count;
                MultiNormalAction::MoreInput
            }
            "c" => (Action::Copy, times).into(),
            "d" => (Action::Trash, times).into(),
            "D" => (Action::Delete, times).into(),
            "f" => (Action::ToggleFullscreen, times).into(),
            "g" => (Action::First, times).into(),
            "G" => (Action::Last, times).into(),
            "h" => (Action::FlipHorizontal, times).into(),
            "?" => {
                match state.render_help {
                    HelpRender::Normal => state.render_help = HelpRender::None,
                    _ => state.render_help = HelpRender::Normal,
                }
                (Action::ReRender, times).into()
            }
            "H" => (Action::Pan(PanAction::Left), times).into(),
            "i" => (Action::Zoom(ZoomAction::In), times).into(),
            "j" => (Action::Next, times).into(),
            "J" => (Action::Pan(PanAction::Down), times).into(),
            "k" => (Action::Prev, times).into(),
            "K" => (Action::Pan(PanAction::Up), times).into(),
            "L" => (Action::Pan(PanAction::Right), times).into(),
            "m" => (Action::Move, times).into(),
            "o" => (Action::Zoom(ZoomAction::Out), times).into(),
            "q" => MultiNormalAction::Quit,
            "r" => (Action::Rotate(RotationDirection::Clockwise), times).into(),
            "R" => (Action::Rotate(RotationDirection::CounterClockwise), times).into(),

            "t" => {
                state.render_infobar = !state.render_infobar;
                (Action::ReRender, times).into()
            }
            "v" => (Action::FlipVertical, times).into(),
            "w" => (Action::SkipForward, times).into(),
            "b" => (Action::SkipBack, times).into(),
            "z" => (Action::ToggleFit, times).into(),
            "Z" => (Action::CenterImage, times).into(),
            _ => MultiNormalAction::Noop,
        },

        Event::KeyDown {
            keycode: Some(k),
            keymod: m,
            ..
        } => match (k, m) {
            (k, &Mod::LSHIFTMOD) | (k, &Mod::RSHIFTMOD) => match k {
                Left => (Action::Pan(PanAction::Left), times).into(),
                Right => (Action::Pan(PanAction::Right), times).into(),
                Up => (Action::Pan(PanAction::Up), times).into(),
                Down => (Action::Pan(PanAction::Down), times).into(),
                _ => MultiNormalAction::Noop,
            },
            (k, &Mod::NOMOD) | (k, _) => match k {
                Delete => (Action::Delete, times).into(),
                Escape => MultiNormalAction::Cancel,
                PageUp => (Action::SkipForward, times).into(),
                PageDown => (Action::SkipBack, times).into(),
                Period => {
                    // Replace times of last action with new
                    state.last_action.times = times;
                    state.last_action.clone().into()
                }
                Right => (Action::Next, times).into(),
                Left => (Action::Prev, times).into(),
                Up => (Action::Zoom(ZoomAction::In), times).into(),
                Down => (Action::Zoom(ZoomAction::Out), times).into(),
                Backspace => (Action::Backspace, 1).into(),
                _ => MultiNormalAction::Noop,
            },
        },

        Event::Window { win_event, .. } => match win_event {
            // Exposed: Rerender if the window was not changed by us.
            Exposed | Resized(..) | SizeChanged(..) | Maximized => MultiNormalAction::ReRender,
            _ => MultiNormalAction::Noop,
        },

        _ => MultiNormalAction::Noop,
    }
}

/// event_action returns which action should be performed in response to this event
pub fn process_normal_mode<'a>(state: &mut State<'a>, event: &Event) -> ProcessAction<'a> {
    // Bring variants in function namespace for reduced typing.
    use sdl2::event::WindowEvent::*;
    use sdl2::keyboard::Keycode::*;

    match event {
        Event::Quit { .. } => Action::Quit.into(),

        Event::TextInput { text, .. } => match text.as_str() {
            // Number of times to repeat operation
            // 0 is not captured for first digit as it does not impact counts
            "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" => {
                // Safe to unwrap as only digits were matched
                let first_digit = text.parse::<usize>().unwrap();
                // Save the first digit before switching
                state.register.cur_action.times = first_digit;
                Action::SwitchMultiNormalMode.into()
            }
            "c" => Action::Copy.into(),
            "d" => Action::Trash.into(),
            "D" => Action::Delete.into(),
            "f" => Action::ToggleFullscreen.into(),
            "g" => Action::First.into(),
            "G" => Action::Last.into(),
            "h" => Action::FlipHorizontal.into(),
            "?" => {
                match state.render_help {
                    HelpRender::Normal => state.render_help = HelpRender::None,
                    _ => state.render_help = HelpRender::Normal,
                }
                Action::ReRender.into()
            }
            "H" => Action::Pan(PanAction::Left).into(),
            "i" => Action::Zoom(ZoomAction::In).into(),
            "j" => Action::Next.into(),
            "J" => Action::Pan(PanAction::Down).into(),
            "k" => Action::Prev.into(),
            "K" => Action::Pan(PanAction::Up).into(),
            "L" => Action::Pan(PanAction::Right).into(),
            "m" => Action::Move.into(),
            "o" => Action::Zoom(ZoomAction::Out).into(),
            "q" => Action::Quit.into(),
            "r" => Action::Rotate(RotationDirection::Clockwise).into(),
            "R" => Action::Rotate(RotationDirection::CounterClockwise).into(),
            "t" => {
                state.render_infobar = !state.render_infobar;
                Action::ReRender.into()
            }
            "v" => Action::FlipVertical.into(),
            "w" => Action::SkipForward.into(),
            "b" => Action::SkipBack.into(),
            "z" => Action::ToggleFit.into(),
            "Z" => Action::CenterImage.into(),
            ":" => Action::SwitchCommandMode.into(),
            _ => Action::Noop.into(),
        },

        Event::KeyDown {
            keycode: Some(k),
            keymod: m,
            ..
        } => match (k, m) {
            (k, &Mod::LSHIFTMOD) | (k, &Mod::RSHIFTMOD) => match k {
                Left => Action::Pan(PanAction::Left).into(),
                Right => Action::Pan(PanAction::Right).into(),
                Up => Action::Pan(PanAction::Up).into(),
                Down => Action::Pan(PanAction::Down).into(),
                _ => Action::Noop.into(),
            },
            (k, &Mod::NOMOD) | (k, _) => match k {
                Delete => Action::Delete.into(),
                F11 => Action::ToggleFullscreen.into(),
                Escape => Action::Quit.into(),
                PageUp => Action::SkipForward.into(),
                PageDown => Action::SkipBack.into(),
                Home => Action::First.into(),
                End => Action::Last.into(),
                Period => state.last_action.clone(),
                Right => Action::Next.into(),
                Left => Action::Prev.into(),
                Up => Action::Zoom(ZoomAction::In).into(),
                Down => Action::Zoom(ZoomAction::Out).into(),
                _ => Action::Noop.into(),
            },
        },

        Event::Window { win_event, .. } => match win_event {
            // Exposed: Rerender if the window was not changed by us.
            Exposed | Resized(..) | SizeChanged(..) | Maximized => Action::ReRender.into(),
            _ => Action::Noop.into(),
        },

        Event::MouseButtonUp { mouse_btn: btn, .. } => match btn {
            MouseButton::Left => Action::ToggleFit.into(),
            _ => Action::Noop.into(),
        },
        _ => Action::Noop.into(),
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

/// Set zoom times to 1 if times is too big for i32 value or times is 0
fn cap_zoom_times(times: usize) -> i32 {
    let zoom_times = (times) as i32;
    // Malicious huge numbers overflow and 0 check
    if zoom_times.is_positive() {
        zoom_times
    } else {
        1
    }
}

#[cfg(test)]
mod tests {
    use super::State;
    #[test]
    fn test_zoom_in_and_then_out_gives_same_zoom_factor() {
        let mut state = State {
            ..Default::default()
        };
        state.zoom_in(1);
        state.zoom_out(1);
        assert_eq!(state.scale, 1.0);
    }

    #[test]
    fn test_mutliple_zoom_in_and_then_out_gives_same_zoom_factor() {
        let mut state = State {
            ..Default::default()
        };
        state.zoom_in(2);
        state.zoom_out(1);
        state.zoom_out(1);
        state.zoom_out(2);
        state.zoom_in(1);
        state.zoom_in(1);
        assert_eq!(state.scale, 1.0);
    }
}

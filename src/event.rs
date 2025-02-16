use std::path::PathBuf;

use vello_svg::vello::kurbo::Point;
use winit::{
    event::{ElementState, Modifiers, MouseButton, MouseScrollDelta, TouchPhase},
    keyboard::Key,
};

/// The event associated with a touch at a single point.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TouchEvent {
    /// The unique ID associated with this touch, e.g. useful for distinguishing between fingers.
    pub id: u64,
    /// The state of the touch.
    pub phase: TouchPhase,
    /// The position of the touch.
    pub position: Point,
}

/// Pressure on a touch pad.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TouchpadPressure {
    /// The unique ID associated with the device that emitted this event.
    pub device_id: winit::event::DeviceId,
    /// The amount of pressure applied.
    pub pressure: f32,
    /// Integer representing the click level.
    pub stage: i64,
}

/// A simplified version of winit's `WindowEvent` type to make it easier to get started.
///
/// All co-ordinates and dimensions are DPI-agnostic scalar values.
///
/// Co-ordinates for each window are as follows:
///
/// - `(0.0, 0.0)` is the centre of the window.
/// - positive `x` points to the right, negative `x` points to the left.
/// - positive `y` points upwards, negative `y` points downwards.
/// - positive `z` points into the screen, negative `z` points out of the screen.
#[derive(Clone, Debug, PartialEq)]
pub enum WindowEvent {
    /// The window has been moved to a new position.
    Moved(Point),

    /// The given keyboard key was pressed.
    KeyPressed(Key),

    /// The given keyboard key was released.
    KeyReleased(Key),

    /// The mouse moved to the given x, y position.
    MouseMoved(Point),

    /// The given mouse button was pressed.
    MousePressed(MouseButton),

    /// The given mouse button was released.
    MouseReleased(MouseButton),

    /// The mouse entered the window.
    MouseEntered,

    /// The mouse exited the window.
    MouseExited,

    /// A mouse wheel movement or touchpad scroll occurred.
    MouseWheel(MouseScrollDelta, TouchPhase),

    /// The window was resized to the given dimensions (in DPI-agnostic points, not pixels).
    Resized(Point),

    /// A file at the given path was hovered over the window.
    HoveredFile(PathBuf),

    /// A file at the given path was dropped onto the window.
    DroppedFile(PathBuf),

    /// A file at the given path that was hovered over the window was cancelled.
    HoveredFileCancelled,

    /// Received a touch event.
    Touch(TouchEvent),

    /// Touchpad pressure event.
    ///
    /// At the moment, only supported on Apple forcetouch-capable macbooks.
    /// The parameters are: pressure level (value between 0 and 1 representing how hard the touchpad
    /// is being pressed) and stage (integer representing the click level).
    TouchPressure(TouchpadPressure),

    /// The window gained focus.
    Focused,

    /// The window lost focus.
    Unfocused,

    /// The window was closed and is no longer stored in the `App`.
    Closed,

    ScaleFactorChanged(f64),

    ModifiersChanged(Modifiers),

    RedrawRequested,
}

impl WindowEvent {
    /// Produce a simplified, new-user-friendly version of the given `winit::event::WindowEvent`.
    ///
    /// This strips rarely needed technical information from the event type such as information
    /// about the source device, scancodes for keyboard events, etc to make the experience of
    /// pattern matching on window events nicer for new users.
    ///
    /// This also interprets the raw pixel positions and dimensions of the raw event into a
    /// dpi-agnostic scalar value where (0, 0, 0) is the centre of the screen with the `y` axis
    /// increasing in the upwards direction.
    ///
    /// If the user requires this extra information, they should use the `raw` field of the
    /// `WindowEvent` type rather than the `simple` one.
    pub fn from_winit_window_event(event: winit::event::WindowEvent) -> Option<Self> {
        use self::WindowEvent::*;
        let event = match event {
            winit::event::WindowEvent::Resized(new_size) => Resized(Point {
                x: new_size.width as f64,
                y: new_size.height as f64,
            }),

            winit::event::WindowEvent::Moved(new_pos) => Moved(Point {
                x: new_pos.x as f64,
                y: new_pos.y as f64,
            }),

            winit::event::WindowEvent::CloseRequested | winit::event::WindowEvent::Destroyed => {
                Closed
            }

            winit::event::WindowEvent::DroppedFile(path) => DroppedFile(path.clone()),

            winit::event::WindowEvent::HoveredFile(path) => HoveredFile(path.clone()),

            winit::event::WindowEvent::HoveredFileCancelled => HoveredFileCancelled,

            winit::event::WindowEvent::Focused(b) => {
                if b {
                    Focused
                } else {
                    Unfocused
                }
            }

            winit::event::WindowEvent::CursorMoved { position, .. } => MouseMoved(Point {
                x: position.x,
                y: position.y,
            }),

            winit::event::WindowEvent::CursorEntered { .. } => MouseEntered,

            winit::event::WindowEvent::CursorLeft { .. } => MouseExited,

            winit::event::WindowEvent::MouseWheel { delta, phase, .. } => MouseWheel(delta, phase),

            winit::event::WindowEvent::MouseInput { state, button, .. } => match state {
                ElementState::Pressed => MousePressed(button),
                ElementState::Released => MouseReleased(button),
            },

            winit::event::WindowEvent::Touch(winit::event::Touch {
                phase,
                location,
                id,
                ..
            }) => {
                let position = Point {
                    x: location.x,
                    y: location.y,
                };
                let touch = TouchEvent {
                    phase,
                    position,
                    id,
                };
                WindowEvent::Touch(touch)
            }

            winit::event::WindowEvent::TouchpadPressure {
                device_id,
                pressure,
                stage,
            } => TouchPressure(TouchpadPressure {
                device_id,
                pressure,
                stage,
            }),

            winit::event::WindowEvent::KeyboardInput { event, .. } => match event.state {
                ElementState::Pressed => KeyPressed(event.logical_key),
                ElementState::Released => KeyReleased(event.logical_key),
            },

            winit::event::WindowEvent::ModifiersChanged(modifiers) => ModifiersChanged(modifiers),

            winit::event::WindowEvent::RedrawRequested => Self::RedrawRequested,

            winit::event::WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                Self::ScaleFactorChanged(scale_factor)
            }

            winit::event::WindowEvent::AxisMotion { .. }
            | winit::event::WindowEvent::ThemeChanged(_) => {
                return None;
            }

            // new 0.28 events
            winit::event::WindowEvent::Ime(_)
            | winit::event::WindowEvent::Occluded(_)
            | winit::event::WindowEvent::ActivationTokenDone { .. }
            | winit::event::WindowEvent::PinchGesture { .. }
            | winit::event::WindowEvent::PanGesture { .. }
            | winit::event::WindowEvent::DoubleTapGesture { .. }
            | winit::event::WindowEvent::RotationGesture { .. } => return None,
        };

        Some(event)
    }
}

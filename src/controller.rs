use crate::{RtsCamera, RtsCameraSystemSet};
use bevy::input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel};
use bevy::input::ButtonInput;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use std::f32::consts::PI;

pub struct RtsCameraControlsPlugin;

impl Plugin for RtsCameraControlsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (zoom, pan, rotate).before(RtsCameraSystemSet));
    }
}

/// Optional camera controller. If you want to use an input manager, don't use this and instead
/// control the camera yourself by updating `RtsCamera.target_focus` and `RtsCamera.target_zoom`.
/// # Example
/// ```no_run
/// # use bevy::prelude::*;
/// # use bevy_rts_camera::{RtsCameraPlugin, RtsCamera, RtsCameraControls};
/// # fn main() {
/// #     App::new()
/// #         .add_plugins(DefaultPlugins)
/// #         .add_plugins(RtsCameraPlugin)
/// #         .add_systems(Startup, setup)
/// #         .run();
/// # }
/// fn setup(mut commands: Commands) {
///     commands
///         .spawn((
///             Camera3dBundle::default(),
///             RtsCamera::default(),
///             RtsCameraControls::default(),
///         ));
///  }
/// ```
#[derive(Component, Debug, PartialEq, Clone)]
pub struct RtsCameraControls {
    /// The key that will pan the camera up (or forward).
    /// Defaults to `KeyCode::ArrowUp`.
    pub key_up: KeyCode,
    /// The key that will pan the camera down (or backward).
    /// Defaults to `KeyCode::ArrowDown`.
    pub key_down: KeyCode,
    /// The key that will pan the camera left.
    /// Defaults to `KeyCode::ArrowLeft`.
    pub key_left: KeyCode,
    /// The key that will pan the camera right.
    /// Defaults to `KeyCode::ArrowRight`.
    pub key_right: KeyCode,
    /// The mouse button used to rotate the camera.
    /// Defaults to `MouseButton::Middle`.
    pub button_rotate: MouseButton,
    /// How far away from the side of the screen edge pan will kick in, defined as a percentage
    /// of the window's height. Set to `0.0` to disable edge panning.
    /// Defaults to `0.05` (5%).
    pub edge_pan_width: f32,
    /// Speed of camera pan (either via keyboard controls or edge panning).
    /// Defaults to `1.0`.
    pub pan_speed: f32,
    /// Whether these controls are enabled.
    /// Defaults to `true`.
    pub enabled: bool,
}

impl Default for RtsCameraControls {
    fn default() -> Self {
        RtsCameraControls {
            key_up: KeyCode::ArrowUp,
            key_down: KeyCode::ArrowDown,
            key_left: KeyCode::ArrowLeft,
            key_right: KeyCode::ArrowRight,
            button_rotate: MouseButton::Middle,
            edge_pan_width: 0.05,
            pan_speed: 15.0,
            enabled: true,
        }
    }
}

pub fn zoom(
    mut mouse_wheel: EventReader<MouseWheel>,
    mut cam_q: Query<(&mut RtsCamera, &RtsCameraControls)>,
) {
    for (mut cam, _) in cam_q.iter_mut().filter(|(_, ctrl)| ctrl.enabled) {
        let zoom_amount = mouse_wheel
            .read()
            .map(|event| match event.unit {
                MouseScrollUnit::Line => event.y,
                MouseScrollUnit::Pixel => event.y * 0.001,
            })
            .fold(0.0, |acc, val| acc + val);
        let new_zoom = (cam.target_zoom + zoom_amount * 0.5).clamp(0.0, 1.0);
        cam.target_zoom = new_zoom;
    }
}

pub fn pan(
    mut cam_q: Query<(&mut RtsCamera, &RtsCameraControls)>,
    button_input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    primary_window_q: Query<&Window, With<PrimaryWindow>>,
    time: Res<Time>,
) {
    for (mut cam, controller) in cam_q.iter_mut().filter(|(_, ctrl)| ctrl.enabled) {
        let mut delta = Vec3::ZERO;

        // Keyboard pan
        if button_input.pressed(controller.key_up) {
            delta += Vec3::from(cam.target_focus.forward())
        }
        if button_input.pressed(controller.key_down) {
            delta += Vec3::from(cam.target_focus.back())
        }
        if button_input.pressed(controller.key_left) {
            delta += Vec3::from(cam.target_focus.left())
        }
        if button_input.pressed(controller.key_right) {
            delta += Vec3::from(cam.target_focus.right())
        }

        // Edge pan
        if delta.length_squared() == 0.0 && !mouse_input.pressed(controller.button_rotate) {
            if let Ok(primary_window) = primary_window_q.get_single() {
                if let Some(cursor_position) = primary_window.cursor_position() {
                    let win_w = primary_window.width();
                    let win_h = primary_window.height();
                    let pan_width = win_h * controller.edge_pan_width;
                    // Pan left
                    if cursor_position.x < pan_width {
                        delta += Vec3::from(cam.target_focus.left())
                    }
                    // Pan right
                    if cursor_position.x > win_w - pan_width {
                        delta += Vec3::from(cam.target_focus.right())
                    }
                    // Pan up
                    if cursor_position.y < pan_width {
                        delta += Vec3::from(cam.target_focus.forward())
                    }
                    // Pan down
                    if cursor_position.y > win_h - pan_width {
                        delta += Vec3::from(cam.target_focus.back())
                    }
                }
            }
        }

        let new_target = cam.target_focus.translation
            + delta.normalize_or_zero()
            * time.delta_seconds()
            * controller.pan_speed
            // Scale based on zoom so it (roughly) feels the same speed at different zoom levels
            * cam.target_zoom.remap(0.0, 1.0, 1.0, 0.5);
        cam.target_focus.translation = new_target;
    }
}

pub fn rotate(
    mut cam_q: Query<(&mut RtsCamera, &RtsCameraControls)>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: EventReader<MouseMotion>,
    primary_window_q: Query<&Window, With<PrimaryWindow>>,
) {
    for (mut cam, controller) in cam_q.iter_mut().filter(|(_, ctrl)| ctrl.enabled) {
        if mouse_input.pressed(controller.button_rotate) {
            let mouse_delta = mouse_motion.read().map(|e| e.delta).sum::<Vec2>();
            if let Ok(primary_window) = primary_window_q.get_single() {
                // Adjust based on window size, so that moving mouse entire width of window
                // will be one half rotation (180 degrees)
                let delta_x = mouse_delta.x / primary_window.width() * PI;
                cam.target_focus.rotate_local_y(-delta_x);
            }
        }
    }
}
use crate::renderer::*;

///
/// A control that makes the camera orbit around a target.
///
pub struct PanOrbitZoomControl {
    min_distance: f32,
    max_distance: f32,
}

impl PanOrbitZoomControl {
    /// Creates a new orbit control with the given target and minimum and maximum distance to the target.
    pub fn new(min_distance: f32, max_distance: f32) -> Self {
        Self {
            min_distance,
            max_distance,
        }
    }

    /// Handles the events. Must be called each frame.
    pub fn handle_events(&mut self, camera: &mut Camera, events: &mut [Event]) -> bool {
        let origin = Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        let target = *camera.target();
        let distance = target.distance(*camera.position());
        let speed = 0.01 * distance + 0.001;
        let mut control = CameraControl {
            right_drag_horizontal: CameraAction::OrbitLeft {
                target: origin,
                speed,
            },
            right_drag_vertical: CameraAction::OrbitUp {
                target: origin,
                speed,
            },
            middle_drag_horizontal: CameraAction::Left { speed },
            middle_drag_vertical: CameraAction::Up { speed },
            scroll_vertical: CameraAction::Zoom {
                min: self.min_distance,
                max: self.max_distance,
                speed,
                target,
            },
            ..Default::default()
        };
        control.handle_events(camera, events)
    }
}

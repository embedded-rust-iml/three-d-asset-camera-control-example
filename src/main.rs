mod pan_orbit_zoom_control;

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

// Entry point for non-wasm
#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() {
    run().await;
}

use three_d::*;

pub async fn run() {
    let window = Window::new(WindowSettings {
        title: "Point Cloud Viewer".to_string(),
        max_size: Some((1280, 720)),
        ..Default::default()
    })
    .unwrap();
    let context = window.gl();

    let mut camera = Camera::new_perspective(
        window.viewport(),
        vec3(0.25, 0.25, 1.0), // where is the camera
        vec3(0.0, 0.0, 0.0),   // which point does the camera look at
        vec3(0.0, 1.0, 0.0),   // which direction is rendered as "upwards"
        degrees(45.0),         // field of view
        0.01,
        100.0,
    );
    let mut control = pan_orbit_zoom_control::PanOrbitZoomControl::new(0.1, 3.0);

    let point_receiver = Arc::new(Mutex::new(None));

    let start = Instant::now();
    let point_writer = point_receiver.clone();
    std::thread::spawn(move || {
        loop {
            let angle = Instant::now().duration_since(start).subsec_millis() as f32
                * 2.0
                * std::f32::consts::PI
                / 1000.0;
            let x = angle.cos() * 0.2;
            let z = angle.sin() * 0.2;

            // Custom point "cloud"
            let cpu_point_cloud = PointCloud {
                positions: Positions::F32(vec![Vector3 { x, y: 0.2, z }]),
                colors: Some(vec![Srgba {
                    r: 255,
                    g: 0,
                    b: 0,
                    a: 255,
                }]),
            };

            *point_writer.lock().unwrap() = Some(cpu_point_cloud);

            thread::sleep(Duration::from_millis(30));
        }
    });

    let mut point_mesh = CpuMesh::sphere(4);
    point_mesh.transform(&Mat4::from_scale(0.01)).unwrap();

    // TODO That's pretty nasty here: In order to not handle Options properly, we wait some time
    // here to guarantee that the producer thread has generated the first point. Afterwards, we can
    // be sure that our point is never None. Later, we do a point.clone().unwrap().
    thread::sleep(Duration::from_millis(30));
    let mut point = None;

    // main loop
    window.render_loop(move |mut frame_input| {
        let mut redraw = frame_input.first_frame;
        redraw |= camera.set_viewport(frame_input.viewport);
        redraw |= control.handle_events(&mut camera, &mut frame_input.events);

        let new_point = point_receiver.lock().unwrap().take();
        if let Some(new_point) = new_point {
            redraw = true;
            point = Some(new_point);
        }

        if redraw {
            let cpu_point_cloud = point.clone().unwrap();

            let point_cloud = Gm {
                geometry: InstancedMesh::new(&context, &cpu_point_cloud.into(), &point_mesh),
                material: ColorMaterial::default(),
            };

            frame_input
                .screen()
                .clear(ClearState::color_and_depth(1.0, 1.0, 1.0, 1.0, 1.0))
                .render(
                    &camera,
                    point_cloud
                        .into_iter()
                        .chain(&Axes::new(&context, 0.01, 1.0)),
                    &[],
                );
        }

        FrameOutput {
            swap_buffers: redraw,
            ..Default::default()
        }
    });
}

extern crate kiss3d;
extern crate nalgebra;

use kiss3d::camera::FirstPerson;
use kiss3d::light::Light;
use kiss3d::window::Window;
use nalgebra::{Point3, Translation3, Vector3};

fn main() {
    let mut window = Window::new("Planets");
    window.set_light(Light::StickToCamera);
    window.set_background_color(1.0, 1.0, 1.0);

    let mut camera = FirstPerson::new(Point3::new(0.0, 0.0, 15.0), Point3::new(0.0, 0.0, 0.0));

    let mut planet1 = window.add_sphere(1.0);
    planet1.set_color(1.0, 0.0, 0.0);

    let pos1 = Vector3::new(4.0, 0.0, 0.0);
    let vel1 = Vector3::new(0.0, 0.025, 0.0);
    planet1.set_local_translation(Translation3::from(pos1));

    let mut planet2 = window.add_sphere(1.0);
    planet2.set_color(1.0, 1.0, 0.0);

    let pos2 = Vector3::new(-4.0, 0.0, 0.0);
    let vel2 = Vector3::new(0.0, -0.025, 0.0);
    planet2.set_local_translation(Translation3::from(pos2));

    let mut planet3 = window.add_sphere(1.0);
    planet3.set_color(1.0, 0.0, 1.0);

    let pos3 = Vector3::new(0.0, 0.0, 5.0);
    let vel3 = Vector3::new(0.025, 0.0, 0.0);
    planet3.set_local_translation(Translation3::from(pos3));

    let mut planets = [
        (planet1, pos1, vel1),
        (planet2, pos2, vel2),
        (planet3, pos3, vel3),
    ];
    let mut new_states = [(Vector3::<f32>::zeros(), Vector3::<f32>::zeros()); 3];

    /* runs at 60Hz */
    while window.render_with_camera(&mut camera) {
        /* Calculate new states. */
        for (i, (_, pos, vel)) in planets.iter().enumerate() {
            /* Calculate total acceleration caused by other planets. */
            let mut acc = Vector3::<f32>::zeros();
            for (_, target_pos, _) in &planets {
                if pos != target_pos {
                    let displacement = *target_pos - *pos;
                    acc += 0.02 * displacement / displacement.norm().powi(3);
                }
            }

            let new_vel = *vel + acc;
            let new_pos = *pos + new_vel;
            new_states[i] = (new_pos, new_vel);
        }

        /* Apply new states to planets. */
        for (i, (planet, pos, vel)) in planets.iter_mut().enumerate() {
            let (new_pos, new_vel) = new_states[i];
            *pos = new_pos;
            *vel = new_vel;
            planet.set_local_translation(Translation3::from(*pos));
        }
    }
}

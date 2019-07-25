extern crate kiss3d;
extern crate nalgebra;

use kiss3d::camera::FirstPerson;
use kiss3d::light::Light;
use kiss3d::scene::SceneNode;
use kiss3d::window::Window;
use nalgebra::{Point3, Translation3, Vector3};

fn main() {
    let mut window = Window::new("Planets");
    window.set_light(Light::StickToCamera);
    window.set_background_color(1.0, 1.0, 1.0);

    let mut camera = FirstPerson::new(Point3::new(0.0, 0.0, 15.0), Point3::new(0.0, 0.0, 0.0));

    let mut ball1 = window.add_sphere(1.0);
    ball1.set_color(1.0, 0.0, 0.0);

    let pos1 = Vector3::new(4.0, 0.0, 0.0);
    let vel1 = Vector3::new(0.0, 0.025, 0.0);

    let mut ball2 = window.add_sphere(1.0);
    ball2.set_color(1.0, 1.0, 0.0);

    let pos2 = Vector3::new(-4.0, 0.0, 0.0);
    let vel2 = Vector3::new(0.0, -0.025, 0.0);

    let mut ball3 = window.add_sphere(1.0);
    ball3.set_color(1.0, 0.0, 1.0);

    let pos3 = Vector3::new(0.0, 0.0, 5.0);
    let vel3 = Vector3::new(0.025, 0.0, 0.0);

    let mut ball4 = window.add_sphere(1.0);
    ball4.set_color(0.0, 0.0, 1.0);

    let pos4 = Vector3::new(0.0, 6.0, 1.0);
    let vel4 = Vector3::new(0.04, 0.0, 0.0);

    let mut planets = [
        create_planet(ball1, pos1, vel1),
        create_planet(ball2, pos2, vel2),
        create_planet(ball3, pos3, vel3),
        create_planet(ball4, pos4, vel4),
    ];
    let mut new_states = [(Vector3::<f64>::zeros(), Vector3::<f64>::zeros()); 4];

    /* runs at 60Hz */
    while window.render_with_camera(&mut camera) {
        /* Calculate new states. */
        for (i, (_, pos, vel)) in planets.iter().enumerate() {
            /* Calculate total acceleration caused by other planets. */
                let mut acc = Vector3::<f64>::zeros();
            for (_, target_pos, _) in &planets {
                if pos != target_pos {
                        let displacement: Vector3<f64> = *target_pos - *pos;
                    acc += 0.02 * displacement / displacement.norm().powi(3);
                }
            }

            let new_vel = *vel + acc;
            let new_pos = *pos + new_vel;
            new_states[i] = (new_pos, new_vel);
        }

        /* Apply new states to planets. */
            for (i, (ball, pos, vel)) in planets.iter_mut().enumerate() {
            let (new_pos, new_vel) = new_states[i];
            *pos = new_pos;
            *vel = new_vel;
                ball.set_local_translation(Translation3::new(
                    pos[0] as f32,
                    pos[1] as f32,
                    pos[2] as f32,
                ));
        }
    }
}

fn create_planet(
    mut ball: SceneNode,
    pos: Vector3<f64>,
    vel: Vector3<f64>,
) -> (SceneNode, Vector3<f64>, Vector3<f64>) {
    ball.set_local_translation(Translation3::new(
        pos[0] as f32,
        pos[1] as f32,
        pos[2] as f32,
    ));
    return (ball, pos, vel);
}

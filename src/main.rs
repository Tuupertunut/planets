extern crate kiss3d;
extern crate nalgebra;
extern crate rand;

use kiss3d::camera::FirstPerson;
use kiss3d::light::Light;
use kiss3d::scene::SceneNode;
use kiss3d::window::Window;
use nalgebra::{Point3, Translation3, Vector3};
use rand::rngs::StdRng;
use rand::{Rng, RngCore, SeedableRng};
use std::time::SystemTime;

struct Planet {
    ball: SceneNode,
    mass: f64,
    pos: Vector3<f64>,
    vel: Vector3<f64>,
}

fn main() {
    let mut window = Window::new_with_size("Planets", 1000, 750);
    window.set_light(Light::StickToCamera);
    window.set_background_color(1.0, 1.0, 1.0);

    let mut camera = FirstPerson::new(Point3::new(0.0, 0.0, 50.0), Point3::new(0.0, 0.0, 0.0));

    let color1 = (1.0, 0.0, 0.0);
    let mass1 = 0.02;
    let pos1 = Vector3::new(4.0, 0.0, 0.0);
    let vel1 = Vector3::new(0.0, 0.025, 0.0);

    let color2 = (1.0, 1.0, 0.0);
    let mass2 = 0.02;
    let pos2 = Vector3::new(-4.0, 0.0, 0.0);
    let vel2 = Vector3::new(0.0, -0.025, 0.0);

    let color3 = (1.0, 0.0, 1.0);
    let mass3 = 0.02;
    let pos3 = Vector3::new(0.0, 0.0, 5.0);
    let vel3 = Vector3::new(0.025, 0.0, 0.0);

    let color4 = (0.0, 0.0, 1.0);
    let mass4 = 0.02;
    let pos4 = Vector3::new(0.0, 6.0, 1.0);
    let vel4 = Vector3::new(0.04, 0.0, 0.0);

    let mut rng = StdRng::seed_from_u64(1);

    let mut planets = [
        create_planet(&mut window, color1, mass1, pos1, vel1),
        create_planet(&mut window, color2, mass2, pos2, vel2),
        create_planet(&mut window, color3, mass3, pos3, vel3),
        create_planet(&mut window, color4, mass4, pos4, vel4),
        create_random_planet(&mut window, &mut rng),
        create_random_planet(&mut window, &mut rng),
        create_random_planet(&mut window, &mut rng),
        create_random_planet(&mut window, &mut rng),
        create_random_planet(&mut window, &mut rng),
        create_random_planet(&mut window, &mut rng),
        create_random_planet(&mut window, &mut rng),
        create_random_planet(&mut window, &mut rng),
        create_random_planet(&mut window, &mut rng),
        create_random_planet(&mut window, &mut rng),
        create_random_planet(&mut window, &mut rng),
        create_random_planet(&mut window, &mut rng),
        create_random_planet(&mut window, &mut rng),
        create_random_planet(&mut window, &mut rng),
        create_random_planet(&mut window, &mut rng),
        create_random_planet(&mut window, &mut rng),
        create_random_planet(&mut window, &mut rng),
        create_random_planet(&mut window, &mut rng),
        create_random_planet(&mut window, &mut rng),
        create_random_planet(&mut window, &mut rng),
        create_random_planet(&mut window, &mut rng),
        create_random_planet(&mut window, &mut rng),
        create_random_planet(&mut window, &mut rng),
        create_random_planet(&mut window, &mut rng),
        create_random_planet(&mut window, &mut rng),
        create_random_planet(&mut window, &mut rng),
    ];
    let mut new_states = [(Vector3::<f64>::zeros(), Vector3::<f64>::zeros()); 30];

    let starttime = SystemTime::now();

    /* runs at 60Hz */
    while window.render_with_camera(&mut camera) {
        let n = 800;
        for _ in 0..n {
            /* Calculate new states. */
            for (i, Planet { pos, vel, .. }) in planets.iter().enumerate() {
                /* Calculate total acceleration caused by other planets. */
                let mut acc = Vector3::<f64>::zeros();
                for Planet {
                    mass: target_mass,
                    pos: target_pos,
                    ..
                } in &planets
                {
                    if pos != target_pos {
                        let displacement = *target_pos - *pos;
                        acc += *target_mass / displacement.norm().powi(3) * displacement;
                    }
                }

                /* Leapfrog integration. The new_vel is actually the velocity half a timestep after
                 * calculating the acceleration, while new_pos in the position one full timestep
                 * after. */
                let new_vel = *vel + acc / (n as f64);
                let new_pos = *pos + new_vel / (n as f64);
                new_states[i] = (new_pos, new_vel);
            }

            /* Apply new states to planets. */
            for (i, Planet { ball, pos, vel, .. }) in planets.iter_mut().enumerate() {
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

        println!("{:?}", starttime.elapsed().unwrap());
    }
}

fn create_planet(
    window: &mut Window,
    (r, g, b): (f32, f32, f32),
    mass: f64,
    pos: Vector3<f64>,
    vel: Vector3<f64>,
) -> Planet {
    let radius = (mass / 0.02).cbrt();
    let mut ball = window.add_sphere(radius as f32);
    ball.set_color(r, g, b);
    ball.set_local_translation(Translation3::new(
        pos[0] as f32,
        pos[1] as f32,
        pos[2] as f32,
    ));
    return Planet {
        ball,
        mass,
        pos,
        vel,
    };
}

fn create_random_planet(window: &mut Window, rng: &mut RngCore) -> Planet {
    let color = (
        rng.gen_range(0.0, 1.0),
        rng.gen_range(0.0, 1.0),
        rng.gen_range(0.0, 1.0),
    );
    let mass = rng.gen_range(0.01, 0.03);
    let pos = Vector3::new(
        rng.gen_range(-25.0, 25.0),
        rng.gen_range(-25.0, 25.0),
        rng.gen_range(-25.0, 25.0),
    );
    let vel = Vector3::new(
        rng.gen_range(-0.04, 0.04),
        rng.gen_range(-0.04, 0.04),
        rng.gen_range(-0.04, 0.04),
    );

    return create_planet(window, color, mass, pos, vel);
}

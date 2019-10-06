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
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::SystemTime;

struct Planet {
    mass: f64,
    pos: Vector3<f64>,
    vel: Vector3<f64>,

    /* Temporary storage fields */
    old_pos: Vector3<f64>,
    old_vel: Vector3<f64>,
    old_a0: Vector3<f64>,
    a0: Vector3<f64>,
    a1: Vector3<f64>,
    a2: Vector3<f64>,
    a3: Vector3<f64>,
}

struct FrameData {
    positions: Vec<Vector3<f32>>,
    total_energy: f64,
    simulation_time: f64,
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

    let planet_data: Vec<(Planet, SceneNode)> = vec![
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

    let (planets, mut balls): (Vec<Planet>, Vec<SceneNode>) = planet_data.into_iter().unzip();

    let starttime = SystemTime::now();
    let initial_energy = measure_total_energy(&planets);

    let (sender, receiver) = mpsc::sync_channel(10000);
    let queue_length = Arc::new(AtomicUsize::new(0));

    let queue_length_clone = queue_length.clone();
    thread::spawn(move || run_physics_thread(planets, sender, queue_length_clone));

    /* runs at 60Hz */
    while window.render_with_camera(&mut camera) {
        if let Ok(frame_data) = receiver.try_recv() {
            let old_queue_length = queue_length.fetch_sub(1, Ordering::Relaxed);

            for (ball, pos) in balls.iter_mut().zip(frame_data.positions) {
                ball.set_local_translation(Translation3::from(pos));
            }

            println!(
                "{:?} {:?} {:?} {:?}",
                starttime.elapsed().unwrap(),
                frame_data.total_energy - initial_energy,
                frame_data.simulation_time,
                old_queue_length - 1
            );
        }
    }
}

fn run_physics_thread(
    mut planets: Vec<Planet>,
    sender: mpsc::SyncSender<FrameData>,
    queue_length: Arc<AtomicUsize>,
) {
    let mut simulation_time = 0.0;
    let mut next_frame_time = 1.0;
    let mut timestep = 1.0;

    let error_tolerance = 1.0e-8;
    let max_timestep = 1.0;

    /* Precalculating a0 for the first step. */
    for i in 0..planets.len() {
        let Planet { pos, .. } = planets[i];

        planets[i].a0 = calculate_acceleration(pos, &planets);
    }

    loop {
        /* Adaptive Runge-Kutta-Nyström 4(5) integrator. Calculating intermediate states. At this
         * point a0 should already be calculated. */
        for Planet {
            pos,
            vel,
            old_pos,
            old_vel,
            a0,
            ..
        } in planets.iter_mut()
        {
            *old_pos = *pos;
            *old_vel = *vel;

            *pos = *old_pos + timestep * 1.0 / 3.0 * *old_vel + timestep.powi(2) * 1.0 / 18.0 * *a0;
        }

        for i in 0..planets.len() {
            let Planet { pos, .. } = planets[i];

            planets[i].a1 = calculate_acceleration(pos, &planets);
        }

        for Planet {
            pos,
            old_pos,
            old_vel,
            a1,
            ..
        } in planets.iter_mut()
        {
            *pos = *old_pos + timestep * 2.0 / 3.0 * *old_vel + timestep.powi(2) * 2.0 / 9.0 * *a1;
        }

        for i in 0..planets.len() {
            let Planet { pos, .. } = planets[i];

            planets[i].a2 = calculate_acceleration(pos, &planets);
        }

        for Planet {
            pos,
            old_pos,
            old_vel,
            a0,
            a2,
            ..
        } in planets.iter_mut()
        {
            *pos = *old_pos
                + timestep * *old_vel
                + timestep.powi(2) * (1.0 / 3.0 * *a0 + 1.0 / 6.0 * *a2);
        }

        for i in 0..planets.len() {
            let Planet { pos, .. } = planets[i];

            planets[i].a3 = calculate_acceleration(pos, &planets);
        }

        /* Setting new position and velocity. */
        for Planet {
            pos,
            vel,
            old_pos,
            old_vel,
            a0,
            a1,
            a2,
            a3,
            ..
        } in planets.iter_mut()
        {
            *pos = *old_pos
                + timestep * *old_vel
                + timestep.powi(2)
                    * (13.0 / 120.0 * *a0 + 3.0 / 10.0 * *a1 + 3.0 / 40.0 * *a2 + 1.0 / 60.0 * *a3);
            *vel = *old_vel
                + timestep
                    * (1.0 / 8.0 * *a0 + 3.0 / 8.0 * *a1 + 3.0 / 8.0 * *a2 + 1.0 / 8.0 * *a3);
        }

        /* Calculating highest error among planets. */
        let mut highest_error = 0.0;
        for i in 0..planets.len() {
            let Planet { pos, a0, a3, .. } = planets[i];

            /* Calculating new a0 for error measurement as well as for use in the next step. */
            planets[i].old_a0 = a0;
            let new_a0 = calculate_acceleration(pos, &planets);
            planets[i].a0 = new_a0;

            let error = (timestep.powi(2) * 1.0 / 60.0 * (a3 - new_a0)).norm();
            highest_error = f64::max(highest_error, error);
        }
        let tolerance = timestep * error_tolerance;

        if highest_error > tolerance {
            /* If error was above tolerance, roll back changes. */
            for Planet {
                pos,
                vel,
                old_pos,
                old_vel,
                old_a0,
                a0,
                ..
            } in planets.iter_mut()
            {
                *pos = *old_pos;
                *vel = *old_vel;
                *a0 = *old_a0;
            }
        } else {
            simulation_time += timestep;

            if simulation_time >= next_frame_time {
                /* Calculating and sending frame data to drawing thread. */
                let positions: Vec<Vector3<f32>> = planets
                    .iter()
                    .map(|Planet { pos, .. }| {
                        Vector3::new(pos[0] as f32, pos[1] as f32, pos[2] as f32)
                    })
                    .collect();

                let total_energy = measure_total_energy(&planets);
                let frame_data = FrameData {
                    positions,
                    total_energy,
                    simulation_time,
                };

                /* Blocks if the frame queue is full. This means that the simulation will just stop
                 * until there is space. */
                sender.send(frame_data).unwrap();
                queue_length.fetch_add(1, Ordering::Relaxed);

                next_frame_time += 1.0;
            }
        }

        /* Estimating an ideal next timestep size based on the error. */
        if highest_error != 0.0 {
            let ideal_timestep = timestep * (0.5 * tolerance / highest_error).powf(1.0 / 4.0);
            timestep = f64::min(max_timestep, ideal_timestep);
        } else {
            timestep = max_timestep;
        }
    }
}

fn create_planet(
    window: &mut Window,
    (r, g, b): (f32, f32, f32),
    mass: f64,
    pos: Vector3<f64>,
    vel: Vector3<f64>,
) -> (Planet, SceneNode) {
    let radius = (mass / 0.02).cbrt();
    let mut ball = window.add_sphere(radius as f32);
    ball.set_color(r, g, b);
    ball.set_local_translation(Translation3::new(
        pos[0] as f32,
        pos[1] as f32,
        pos[2] as f32,
    ));
    return (
        Planet {
            mass,
            pos,
            vel,

            old_pos: Vector3::<f64>::zeros(),
            old_vel: Vector3::<f64>::zeros(),
            old_a0: Vector3::<f64>::zeros(),
            a0: Vector3::<f64>::zeros(),
            a1: Vector3::<f64>::zeros(),
            a2: Vector3::<f64>::zeros(),
            a3: Vector3::<f64>::zeros(),
        },
        ball,
    );
}

fn create_random_planet(window: &mut Window, rng: &mut RngCore) -> (Planet, SceneNode) {
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

fn measure_total_energy(planets: &[Planet]) -> f64 {
    let mut energy = 0.0;
    for (i, Planet { mass, pos, vel, .. }) in planets.iter().enumerate() {
        /* Kinetic energy */
        energy += mass * vel.norm_squared() / 2.0;

        /* Potential energy */
        for Planet {
            mass: target_mass,
            pos: target_pos,
            ..
        } in &planets[(i + 1)..]
        {
            let displacement = target_pos - pos;
            energy -= target_mass * mass / displacement.norm();
        }
    }
    return energy;
}

fn calculate_acceleration(pos: Vector3<f64>, planets: &[Planet]) -> Vector3<f64> {
    let mut acc = Vector3::<f64>::zeros();
    for Planet {
        mass: target_mass,
        pos: target_pos,
        ..
    } in planets.iter()
    {
        if pos != *target_pos {
            let displacement = target_pos - pos;
            acc += target_mass / displacement.norm().powi(3) * displacement;
        }
    }
    return acc;
}

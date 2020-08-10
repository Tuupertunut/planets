extern crate kiss3d;
extern crate nalgebra;
extern crate rand;

use kiss3d::camera::FirstPerson;
use kiss3d::light::Light;
use kiss3d::scene::SceneNode;
use kiss3d::window::Window;
use nalgebra::{Point3, Translation3, Vector3};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::iter;
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
    old_a3: Vector3<f64>,
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
    test();
}

fn test() {
    let total_starttime = SystemTime::now();

    let mut rng = StdRng::seed_from_u64(2);

    for _ in 0..100 {
        let planets = iter::repeat_with(|| create_random_planet(&mut rng))
            .take(30)
            .collect::<Vec<Planet>>();

        let starttime = SystemTime::now();
        let initial_energy = measure_total_energy(&planets);

        let frame_interval = 1.0;

        let (sender, receiver) = mpsc::sync_channel(1);
        let queue_length = Arc::new(AtomicUsize::new(0));

        thread::spawn(move || run_physics_thread(planets, sender, queue_length, frame_interval));

        for _ in 0..1999 {
            receiver.recv().unwrap();
        }
        let frame_data = receiver.recv().unwrap();

        println!(
            "{:?} {:?} {:?}",
            starttime.elapsed().unwrap(),
            frame_data.total_energy - initial_energy,
            frame_data.simulation_time
        );
    }

    println!("{:?}", total_starttime.elapsed().unwrap());
}

fn run() {
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
        create_visible_planet(&mut window, color1, mass1, pos1, vel1),
        create_visible_planet(&mut window, color2, mass2, pos2, vel2),
        create_visible_planet(&mut window, color3, mass3, pos3, vel3),
        create_visible_planet(&mut window, color4, mass4, pos4, vel4),
        create_random_visible_planet(&mut window, &mut rng),
        create_random_visible_planet(&mut window, &mut rng),
        create_random_visible_planet(&mut window, &mut rng),
        create_random_visible_planet(&mut window, &mut rng),
        create_random_visible_planet(&mut window, &mut rng),
        create_random_visible_planet(&mut window, &mut rng),
        create_random_visible_planet(&mut window, &mut rng),
        create_random_visible_planet(&mut window, &mut rng),
        create_random_visible_planet(&mut window, &mut rng),
        create_random_visible_planet(&mut window, &mut rng),
        create_random_visible_planet(&mut window, &mut rng),
        create_random_visible_planet(&mut window, &mut rng),
        create_random_visible_planet(&mut window, &mut rng),
        create_random_visible_planet(&mut window, &mut rng),
        create_random_visible_planet(&mut window, &mut rng),
        create_random_visible_planet(&mut window, &mut rng),
        create_random_visible_planet(&mut window, &mut rng),
        create_random_visible_planet(&mut window, &mut rng),
        create_random_visible_planet(&mut window, &mut rng),
        create_random_visible_planet(&mut window, &mut rng),
        create_random_visible_planet(&mut window, &mut rng),
        create_random_visible_planet(&mut window, &mut rng),
        create_random_visible_planet(&mut window, &mut rng),
        create_random_visible_planet(&mut window, &mut rng),
        create_random_visible_planet(&mut window, &mut rng),
        create_random_visible_planet(&mut window, &mut rng),
    ];

    let (planets, mut balls): (Vec<Planet>, Vec<SceneNode>) = planet_data.into_iter().unzip();

    let starttime = SystemTime::now();
    let initial_energy = measure_total_energy(&planets);

    let frame_interval = 1.0;

    let (sender, receiver) = mpsc::sync_channel(10000);
    let queue_length = Arc::new(AtomicUsize::new(0));

    let queue_length_clone = queue_length.clone();
    thread::spawn(move || run_physics_thread(planets, sender, queue_length_clone, frame_interval));

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
    frame_interval: f64,
) {
    let mut simulation_time = 0.0;
    let mut next_frame_time = frame_interval;
    let mut timestep = 1.0;

    let error_tolerance = 1.0e-4;
    let max_timestep = 1.0;

    /* Precalculating a3 for the first step. */
    for i in 0..planets.len() {
        let Planet { pos, .. } = planets[i];

        planets[i].a3 = calculate_acceleration(pos, &planets);
    }

    loop {
        /* Adaptive Runge-Kutta-Nyström 4(3) integrator (RKN4(3)4FM from
         * https://doi.org/10.1093/imanum/7.2.235). Calculating intermediate states. */
        for Planet {
            pos,
            vel,
            a3,
            old_pos,
            old_vel,
            old_a3,
            ..
        } in planets.iter_mut()
        {
            *old_pos = *pos;
            *old_vel = *vel;
            *old_a3 = *a3;

            *pos = *old_pos
                + timestep * 1.0 / 4.0 * *old_vel
                + timestep.powi(2) * 1.0 / 32.0 * *old_a3;
        }

        for i in 0..planets.len() {
            let Planet { pos, .. } = planets[i];

            planets[i].a1 = calculate_acceleration(pos, &planets);
        }

        for Planet {
            pos,
            old_pos,
            old_vel,
            old_a3,
            a1,
            ..
        } in planets.iter_mut()
        {
            *pos = *old_pos
                + timestep * 7.0 / 10.0 * *old_vel
                + timestep.powi(2) * (7.0 / 1000.0 * *old_a3 + 119.0 / 500.0 * *a1);
        }

        for i in 0..planets.len() {
            let Planet { pos, .. } = planets[i];

            planets[i].a2 = calculate_acceleration(pos, &planets);
        }

        for Planet {
            pos,
            old_pos,
            old_vel,
            old_a3,
            a1,
            a2,
            ..
        } in planets.iter_mut()
        {
            *pos = *old_pos
                + timestep * *old_vel
                + timestep.powi(2) * (1.0 / 14.0 * *old_a3 + 8.0 / 27.0 * *a1 + 25.0 / 189.0 * *a2);
        }

        let mut highest_error = 0.0;

        for i in 0..planets.len() {
            let Planet {
                pos,
                old_vel,
                old_a3,
                a1,
                a2,
                ..
            } = planets[i];

            let a3 = calculate_acceleration(pos, &planets);
            planets[i].a3 = a3;

            planets[i].vel = old_vel
                + timestep
                    * (1.0 / 14.0 * old_a3
                        + 32.0 / 81.0 * a1
                        + 250.0 / 567.0 * a2
                        + 5.0 / 54.0 * a3);
            let pos_error = timestep.powi(2)
                * ((-7.0 / 150.0 - 1.0 / 14.0) * old_a3
                    + (67.0 / 150.0 - 8.0 / 27.0) * a1
                    + (3.0 / 20.0 - 25.0 / 189.0) * a2
                    + -1.0 / 20.0 * a3);
            let vel_error = timestep
                * ((13.0 / 21.0 - 1.0 / 14.0) * old_a3
                    + (-20.0 / 27.0 - 32.0 / 81.0) * a1
                    + (275.0 / 189.0 - 250.0 / 567.0) * a2
                    + (-1.0 / 3.0 - 5.0 / 54.0) * a3);

            let error = f64::max(pos_error.norm(), vel_error.norm());
            highest_error = f64::max(highest_error, error);
        }

        let step_error_tolerance = timestep * error_tolerance;

        if highest_error > step_error_tolerance {
            /* If error was above tolerance, roll back changes. */
            for Planet {
                pos,
                vel,
                a3,
                old_pos,
                old_vel,
                old_a3,
                ..
            } in planets.iter_mut()
            {
                *pos = *old_pos;
                *vel = *old_vel;
                *a3 = *old_a3;
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
                let result = sender.send(frame_data);
                if result.is_err() {
                    break;
                }
                queue_length.fetch_add(1, Ordering::Relaxed);

                next_frame_time += frame_interval;
            }
        }

        /* Estimating an ideal next timestep size based on the error. */
        if highest_error != 0.0 {
            let ideal_timestep =
                timestep * (0.5 * step_error_tolerance / highest_error).powf(1.0 / 4.0);
            timestep = f64::min(max_timestep, ideal_timestep);
        } else {
            timestep = max_timestep;
        }
    }
}

fn create_visible_planet(
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
    return (create_planet(mass, pos, vel), ball);
}

fn create_planet(mass: f64, pos: Vector3<f64>, vel: Vector3<f64>) -> Planet {
    return Planet {
        mass,
        pos,
        vel,

        old_pos: Vector3::<f64>::zeros(),
        old_vel: Vector3::<f64>::zeros(),
        old_a3: Vector3::<f64>::zeros(),
        a1: Vector3::<f64>::zeros(),
        a2: Vector3::<f64>::zeros(),
        a3: Vector3::<f64>::zeros(),
    };
}

fn create_random_visible_planet(window: &mut Window, rng: &mut StdRng) -> (Planet, SceneNode) {
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

    return create_visible_planet(window, color, mass, pos, vel);
}

fn create_random_planet(rng: &mut StdRng) -> Planet {
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

    return create_planet(mass, pos, vel);
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

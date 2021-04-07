use kiss3d::camera::FirstPerson;
use kiss3d::light::Light;
use kiss3d::scene::SceneNode;
use kiss3d::window::Window;
use nalgebra::{Point3, Translation3, Vector3};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::SystemTime;

struct Planet {
    mass: f64,
    pos: Vector3<f64>,
    vel: Vector3<f64>,
    acc: Vector3<f64>,
}

struct FrameData {
    positions: Vec<Vector3<f32>>,
    total_energy: f64,
    simulation_time: f64,
}

fn main() {
    run();
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
    let timestep = 0.0005;

    /* Precalculating acceleration for the first timestep. */
    for i in 0..planets.len() {
        planets[i].acc = calculate_acceleration(planets[i].pos, &planets);
    }

    loop {
        /* Velocity Verlet integrator. */
        for Planet { pos, vel, acc, .. } in planets.iter_mut() {
            *pos = *pos + timestep * (*vel + 0.5 * timestep * *acc);
        }

        for i in 0..planets.len() {
            let new_acc = calculate_acceleration(planets[i].pos, &planets);
            planets[i].vel = planets[i].vel + 0.5 * timestep * (planets[i].acc + new_acc);
            planets[i].acc = new_acc;
        }

        simulation_time += timestep;

        if simulation_time >= next_frame_time {
            /* Calculating and sending frame data to drawing thread. */
            let positions: Vec<Vector3<f32>> = planets
                .iter()
                .map(|Planet { pos, .. }| Vector3::new(pos[0] as f32, pos[1] as f32, pos[2] as f32))
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
        acc: Vector3::<f64>::zeros(),
    };
}

fn create_random_visible_planet(window: &mut Window, rng: &mut StdRng) -> (Planet, SceneNode) {
    let color = (
        rng.gen_range(0.0..1.0),
        rng.gen_range(0.0..1.0),
        rng.gen_range(0.0..1.0),
    );
    let mass = rng.gen_range(0.01..0.03);
    let pos = Vector3::new(
        rng.gen_range(-25.0..25.0),
        rng.gen_range(-25.0..25.0),
        rng.gen_range(-25.0..25.0),
    );
    let vel = Vector3::new(
        rng.gen_range(-0.04..0.04),
        rng.gen_range(-0.04..0.04),
        rng.gen_range(-0.04..0.04),
    );

    return create_visible_planet(window, color, mass, pos, vel);
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

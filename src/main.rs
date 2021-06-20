mod components;

use bevy::prelude::*;

use crate::components::*;

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::rgb(0.9, 0.9, 0.9)))
        .add_startup_system(setup.system())
        .add_system(player_input_system.system())
        .add_system(thrust_system.system())
        .add_system(gravity_system.system())
        .add_system(velocity_system.system())
        .run();
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>
) {

    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    let player_radius = 10.0;
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(Color::rgb(140.0, 140.0, 140.0).into()),
            transform: Transform::from_xyz(0.0, 150.0, 0.0),
            sprite: Sprite::new(Vec2::new(player_radius, player_radius)),
            ..Default::default()
        })
        .insert(entities::Player {})
        .insert(physics::Thrust {
            thrust: 0.0,
            facing: 0.0,
        })
        .insert(physics::Velocity {
            velocity: Vec3::new(150.0, 0.0, 0.0),
        })
        .insert(physics::Gravity::Movable(physics::MassRadius {
            radius: player_radius,
            mass: 0.001
        }));
        
    let planet_radius = 100.0;
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(Color::rgb(0.0, 0.0, 0.0).into()),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            sprite: Sprite::new(Vec2::new(planet_radius, planet_radius)),
            ..Default::default()
        })
        .insert(entities::Planet {
        })
        .insert(physics::Gravity::Immovable(physics::MassRadius {
            radius: planet_radius,
            mass: 500.0 * (10f32).powi(14)
        }));
}


const PLAYER_ACCELERATION_RATE: f32 = 50.0;
const PLAYER_TURN_RATE: f32 = 10.0;

fn player_input_system(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&entities::Player, &mut physics::Thrust)>,
) {
    if let Ok((_player, mut thrust)) = query.single_mut() {
        if keyboard_input.pressed(KeyCode::Left) {
            thrust.facing -= PLAYER_TURN_RATE * time.delta_seconds();
        }

        if keyboard_input.pressed(KeyCode::Right) {
            thrust.facing += PLAYER_TURN_RATE * time.delta_seconds();
        }

        if keyboard_input.pressed(KeyCode::Up) {
            thrust.thrust += PLAYER_ACCELERATION_RATE * time.delta_seconds();
        }

        if keyboard_input.pressed(KeyCode::Down) {
            thrust.thrust -= PLAYER_ACCELERATION_RATE * time.delta_seconds();
            
        }
        if thrust.thrust < 0.0 {
            thrust.thrust = 0.0;
        }
    }
}

fn gravity_system(
    time: Res<Time>,
    mut set: QuerySet<(
        Query<(&physics::Gravity, &Transform, &entities::Planet)>,
        Query<(&physics::Gravity, &Transform, &mut physics::Velocity)>
    )>
) {
    let mut planet_mass_radius = physics::MassRadius { mass: 0.0, radius: 0.0 };
    let mut planet_translation = Vec3::new(0.0, 0.0, 0.0);
    if let Ok((planet_gravity, planet_transform, _planet)) = set.q0().single() {
        //Some cheating is going on here:
        // - Only one planet
        if let physics::Gravity::Immovable(pmr) = planet_gravity {
            planet_mass_radius.mass = pmr.mass;
            planet_mass_radius.radius = pmr.radius;
            planet_translation.x = planet_transform.translation.x;
            planet_translation.y = planet_transform.translation.y;
        }
    }
    for (object_gravity, object_transform, mut velocity) in set.q1_mut().iter_mut() {
        if let physics::Gravity::Movable(object_mass_radius) = object_gravity {
            let distance_between_objects = distance_between_two_vec(planet_translation, object_transform.translation);
            if distance_between_objects <= (planet_mass_radius.radius + object_mass_radius.radius) {
                // object has crashed - it needs to stop moving
                velocity.velocity.x = 0.0;
                velocity.velocity.y = 0.0;
                velocity.velocity.z = 0.0;
                continue;
            }
            //We should definitely not be calcing this constantly.
            //https://en.wikipedia.org/wiki/Gravitational_constant
            let gravitational_constant: f32 = 6.674f32 * (10f32).powi(-11);
            let force = gravitational_constant * ((object_mass_radius.mass * planet_mass_radius.mass) / distance_between_objects.powi(2));
            //F=MA
            //A = F/M
            let acceleration = force / object_mass_radius.mass;
            //A = distance/time
            //distance = A*time
            let distance = acceleration * time.delta_seconds();
            let normalised_vector = vec_from_angle(angle_between_two_vec(planet_translation, object_transform.translation));
            velocity.velocity += normalised_vector * distance;
        }
    }
}

fn thrust_system(
    time: Res<Time>,
    mut query: Query<(&physics::Thrust, &mut physics::Velocity)>
) {
    for (thrust, mut velocity) in query.iter_mut() {
        if thrust.thrust > 0.0 {
            velocity.velocity += time.delta_seconds() * (thrust.thrust * vec_from_angle(thrust.facing));
        }
    }
}

fn velocity_system(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &physics::Velocity)>
) {
    for (mut transform, velocity) in query.iter_mut() {
        transform.translation += time.delta_seconds() * velocity.velocity;
    }
}



//Note: this all uses Vec3s, we only want to care about x and y though

fn vec_from_angle(angle: f32) -> Vec3 {
    let vx = angle.sin();
    let vy = angle.cos();
    Vec3::new(vx, vy, 0.0)
}


fn angle_between_two_vec(sourcevec: Vec3, targetvec: Vec3) -> f32 {
    let vec = sourcevec - targetvec;
    vec.x.atan2(vec.y)
}

fn distance_between_two_vec(vecone: Vec3, vectwo: Vec3) -> f32{
    let vec = vecone - vectwo;
    ((vec.x * vec.x) + (vec.y * vec.y)).abs().sqrt()
}
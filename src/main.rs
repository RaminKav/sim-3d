//TODO: sort targets by travel time

use bevy_flycam::prelude::*;
use std::f32::consts::FRAC_PI_2;

use bevy::{
    asset::LoadState,
    gltf::{Gltf, GltfMesh},
    pbr::NotShadowCaster,
    prelude::*,
    window::PrimaryWindow,
};
use bevy_egui::{
    egui::{self},
    EguiContexts, EguiPlugin,
};
use bevy_xpbd_3d::{
    components::RigidBody,
    plugins::{collision::Collider, PhysicsDebugPlugin, PhysicsPlugins},
};
use rand::Rng;
use vleue_navigator::{NavMesh, VleueNavigatorPlugin};

const HANDLE_TRIMESH_OPTIMIZED: Handle<NavMesh> = Handle::weak_from_u128(0);

fn main() {
    App::new()
        .insert_resource(Msaa::default())
        .add_plugins((
            DefaultPlugins,
            PhysicsPlugins::default(),
            PhysicsDebugPlugin::default(),
        ))
        .add_plugins(EguiPlugin)
        .add_plugins(NoCameraPlayerPlugin)
        .add_plugins(VleueNavigatorPlugin)
        .add_event::<SimulateEvent>()
        .init_state::<AppState>()
        .add_systems(OnEnter(AppState::Setup), setup)
        .add_systems(Update, check_textures.run_if(in_state(AppState::Setup)))
        .add_systems(OnExit(AppState::Setup), setup_scene)
        .add_systems(
            Update,
            (
                build_egui,
                change_selected_target_color,
                give_target_on_click,
                trigger_navmesh_visibility,
                move_object,
            ),
        )
        .add_systems(Update, simulate_listener)
        .run();
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, States, Default)]
enum AppState {
    #[default]
    Setup,
    Playing,
}

#[derive(Clone, Debug, Default, Event)]
pub struct SimulateEvent;

#[derive(Component)]
pub struct TargetState {
    selected: bool,
}
#[derive(Component)]
struct Object(Option<Entity>);

#[derive(Resource)]
pub struct TargetMaterials {
    selected: Handle<StandardMaterial>,
    un_selected: Handle<StandardMaterial>,
}

#[derive(Resource, Default, Deref)]
struct GltfHandle(Handle<Gltf>);

#[derive(Component, Clone)]
struct NavMeshDisp(Handle<NavMesh>);

#[derive(Resource)]
struct CurrentMesh(Handle<NavMesh>);

#[derive(Component)]
struct Path {
    current: Vec3,
    next: Vec<Vec3>,
}
#[derive(Component)]
struct Target;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands.insert_resource(GltfHandle(asset_server.load("meshes/mesh5.glb")));

    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 300.0,
    });

    commands.insert_resource(TargetMaterials {
        selected: materials.add(Color::rgb(0.4, 0.8, 0.2)),
        un_selected: materials.add(Color::rgb(1., 0.7, 0.7)),
    });

    // TARGETS
    for i in 0..20 {
        let transform = if i < 10 {
            Transform::from_xyz(
                -20.5 + 4. * (i as f32 % 2.),
                0.7,
                -7.5 + f32::floor(i as f32 / 2.) * 5.,
            )
        } else {
            Transform::from_xyz(
                -8.0 + 4. * (i as f32 % 2.),
                0.7,
                -7.5 + f32::floor((i as f32 - 10.) / 2.) * 5.,
            )
        };
        commands.spawn((
            RigidBody::Static,
            TargetState { selected: false },
            // AngularVelocity(Vec3::new(2.5, 3.4, 1.6)),
            Collider::cuboid(1., 1., 2.3),
            PbrBundle {
                mesh: meshes.add(Cuboid::from_size(Vec3::new(1., 1., 2.3))),
                material: materials.add(Color::rgb(1., 0.7, 0.7)),
                transform,
                ..default()
            },
        ));
    }

    // commands.spawn(PointLightBundle {
    //     point_light: PointLight {
    //         intensity: 54500.0,
    //         shadows_enabled: true,
    //         ..Default::default()
    //     },
    //     transform: Transform::from_xyz(4.0, 8.0, 4.0),
    //     ..Default::default()
    // });
    // commands.spawn(PointLightBundle {
    //     point_light: PointLight {
    //         intensity: 54500.0,
    //         shadows_enabled: true,
    //         ..Default::default()
    //     },
    //     transform: Transform::from_xyz(2.0, 8.0, 2.0),
    //     ..Default::default()
    // });

    commands.spawn((
        FlyCam,
        Camera3dBundle {
            camera: Camera {
                #[cfg(not(target_arch = "wasm32"))]
                hdr: true,
                ..default()
            },
            transform: Transform::from_xyz(-4.0, 15., 20.0)
                .looking_at(Vec3::new(0.0, 2., 0.), Vec3::Y),
            ..Default::default()
        },
    ));
    commands.insert_resource(CurrentMesh(HANDLE_TRIMESH_OPTIMIZED));
}

fn check_textures(
    mut next_state: ResMut<NextState<AppState>>,
    gltf: ResMut<GltfHandle>,
    asset_server: Res<AssetServer>,
) {
    if let Some(LoadState::Loaded) = asset_server.get_load_state(gltf.id()) {
        next_state.set(AppState::Playing);
    }
}

fn setup_scene(
    mut commands: Commands,
    gltf: Res<GltfHandle>,
    gltfs: Res<Assets<Gltf>>,
    gltf_meshes: Res<Assets<GltfMesh>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut navmeshes: ResMut<Assets<NavMesh>>,
) {
    let mut material: StandardMaterial = Color::ALICE_BLUE.into();
    material.perceptual_roughness = 1.0;
    let ground_material = materials.add(material);
    if let Some(gltf) = gltfs.get(gltf.id()) {
        let mesh = gltf_meshes.get(&gltf.named_meshes["walls.001"]).unwrap();
        let mut material: StandardMaterial = Color::GRAY.into();
        material.perceptual_roughness = 1.0;
        commands.spawn(PbrBundle {
            mesh: mesh.primitives[0].mesh.clone(),
            transform: Transform::from_xyz(0.0, 5.5, 0.0),
            material: materials.add(material),
            ..default()
        });

        let mesh = gltf_meshes.get(&gltf.named_meshes["plane.002"]).unwrap();
        commands.spawn((
            PbrBundle {
                mesh: mesh.primitives[0].mesh.clone(),
                transform: Transform::from_xyz(0.0, 0.1, 0.0),
                material: ground_material.clone(),
                ..default()
            },
            RigidBody::Static,
            Collider::cuboid(100.5, 0.2, 100.3),
        ));

        let mesh = gltf_meshes.get(&gltf.named_meshes["agent.001"]).unwrap();
        let mut material: StandardMaterial = Color::GRAY.into();
        material.perceptual_roughness = 1.0;
        commands.spawn((
            PbrBundle {
                mesh: mesh.primitives[0].mesh.clone(),
                transform: Transform::from_xyz(0.0, 0.5, 0.0),
                material: materials.add(material),
                ..default()
            },
            Object(None),
        ));

        {
            #[cfg(target_arch = "wasm32")]
            const NB_HOVER: usize = 5;
            #[cfg(not(target_arch = "wasm32"))]
            const NB_HOVER: usize = 10;

            for _i in 0..NB_HOVER {
                commands.spawn((SpotLightBundle {
                    spot_light: SpotLight {
                        intensity: 1000000.0,
                        color: Color::SEA_GREEN,
                        shadows_enabled: true,
                        inner_angle: 0.5,
                        outer_angle: 0.8,
                        range: 250.0,
                        ..default()
                    },
                    transform: Transform::from_xyz(
                        rand::thread_rng().gen_range(-50.0..50.0),
                        20.0,
                        rand::thread_rng().gen_range(-25.0..25.0),
                    )
                    .with_rotation(Quat::from_rotation_x(-FRAC_PI_2)),
                    ..default()
                },));
            }
        }
    }

    if let Some(gltf) = gltfs.get(gltf.id()) {
        {
            let navmesh = vleue_navigator::NavMesh::from_bevy_mesh(
                meshes
                    .get(
                        &gltf_meshes
                            .get(&gltf.named_meshes["navmesh.002"])
                            .unwrap()
                            .primitives[0]
                            .mesh,
                    )
                    .unwrap(),
            );

            let mut material: StandardMaterial = Color::ANTIQUE_WHITE.into();
            material.unlit = true;

            commands.spawn((
                PbrBundle {
                    mesh: meshes.add(navmesh.to_wireframe_mesh()),
                    material: materials.add(material),
                    transform: Transform::from_xyz(0.0, 0.3, 0.0),
                    visibility: Visibility::Hidden,
                    ..Default::default()
                },
                NavMeshDisp(HANDLE_TRIMESH_OPTIMIZED),
            ));
            navmeshes.insert(HANDLE_TRIMESH_OPTIMIZED, navmesh);
        }
    }
}
fn trigger_navmesh_visibility(
    mut query: Query<(&mut Visibility, &NavMeshDisp)>,
    keyboard_input: ResMut<ButtonInput<KeyCode>>,
    current_mesh: Res<CurrentMesh>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyM) {
        for (mut visible, nav) in query.iter_mut() {
            if nav.0 == current_mesh.0 {
                match *visible {
                    Visibility::Visible => *visible = Visibility::Hidden,
                    Visibility::Hidden => *visible = Visibility::Visible,
                    Visibility::Inherited => *visible = Visibility::Inherited,
                }
            }
        }
    }
}
fn build_egui(
    mut ctx: EguiContexts,
    mut sim_event: EventWriter<SimulateEvent>,
    mut targets: Query<(Entity, &mut TargetState)>,
) {
    let mut start_sim = false;
    egui::SidePanel::left("Control Panel").show(ctx.ctx_mut(), |ui| {
        egui::CollapsingHeader::new("Simulation Targets").show(ui, |ui| {
            for (e, mut selected) in targets.iter_mut() {
                ui.add_space(10.0);
                ui.checkbox(&mut selected.selected, format!("{:?}", e));
                ui.end_row();
            }
        });

        ui.add_space(10.0);
        start_sim = ui.button("Simulate").clicked();
    });

    if start_sim {
        sim_event.send_default();
    }
}

fn change_selected_target_color(
    mut commands: Commands,
    mut toggled_targets: Query<(Entity, &TargetState), Changed<TargetState>>,
    target_mats: Res<TargetMaterials>,
) {
    for (e, target_obj) in toggled_targets.iter_mut() {
        if target_obj.selected {
            commands.entity(e).insert(target_mats.selected.clone());
        } else {
            commands.entity(e).insert(target_mats.un_selected.clone());
        }
    }
}

fn give_target_on_click(
    mut commands: Commands,
    mut object_query: Query<(Entity, &Transform, &mut Object)>,
    targets: Query<Entity, With<Target>>,
    navmeshes: Res<Assets<NavMesh>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    current_mesh: Res<CurrentMesh>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform)>,
) {
    if mouse_buttons.just_pressed(MouseButton::Left) {
        let navmesh = navmeshes.get(&current_mesh.0).unwrap();
        let Some(target) = (|| {
            let position = primary_window.single().cursor_position()?;
            let (camera, transform) = camera.get_single().ok()?;
            let ray = camera.viewport_to_world(transform, position)?;
            let denom = Vec3::Y.dot(ray.direction.into());
            let t = (Vec3::ZERO - ray.origin).dot(Vec3::Y) / denom;
            let target = ray.origin + ray.direction * t;
            navmesh.transformed_is_in_mesh(target).then_some(target)
        })() else {
            return;
        };

        for (entity, transform, mut object) in object_query.iter_mut() {
            let Some(path) = navmesh.transformed_path(transform.translation, target) else {
                break;
            };
            if let Some((first, remaining)) = path.path.split_first() {
                let mut remaining = remaining.to_vec();
                remaining.reverse();
                let target_id = commands
                    .spawn((
                        PbrBundle {
                            mesh: meshes.add(Mesh::from(Sphere {
                                radius: 0.5,
                                ..default()
                            })),
                            material: materials.add(StandardMaterial {
                                base_color: Color::RED,
                                emissive: Color::RED * 50.0,
                                ..default()
                            }),
                            transform: Transform::from_translation(target),
                            ..Default::default()
                        },
                        NotShadowCaster,
                        Target,
                    ))
                    .with_children(|target| {
                        target.spawn(PointLightBundle {
                            point_light: PointLight {
                                color: Color::RED,
                                shadows_enabled: true,
                                range: 10.0,
                                ..default()
                            },
                            transform: Transform::from_xyz(0.0, 1.5, 0.0),
                            ..default()
                        });
                    })
                    .id();
                commands.entity(entity).insert(Path {
                    current: first.clone(),
                    next: remaining,
                });
                object.0 = Some(target_id);
            }
        }
        for entity in &targets {
            commands.entity(entity).despawn_recursive();
        }
    }
}
fn simulate_listener(
    mut commands: Commands,
    mut sim_event: EventReader<SimulateEvent>,
    mut agent: Query<(Entity, &Transform, &mut Object), (Without<Path>, Without<TargetState>)>,
    targets: Query<(Entity, &TargetState, &Transform), With<TargetState>>,
    navmeshes: Res<Assets<NavMesh>>,
    current_mesh: Res<CurrentMesh>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !sim_event.is_empty() {
        sim_event.clear();
        println!("SIMULATION BEGINING");
        let Ok((agent_e, agent_tfxm, mut agent_targets)) = agent.get_single_mut() else {
            return;
        };
        for (target_e, target, target_txf) in targets.iter() {
            let navmesh = navmeshes.get(&current_mesh.0).unwrap();
            if target.selected {
                let Some(path) = navmesh.transformed_path(
                    agent_tfxm.translation,
                    Vec3::new(target_txf.translation.x, 0.0, target_txf.translation.z),
                ) else {
                    break;
                };
                if let Some((first, remaining)) = path.path.split_first() {
                    let mut remaining = remaining.to_vec();
                    remaining.reverse();
                    let target_id = commands
                        .spawn((
                            PbrBundle {
                                mesh: meshes.add(Mesh::from(Sphere {
                                    radius: 0.5,
                                    ..default()
                                })),
                                material: materials.add(StandardMaterial {
                                    base_color: Color::RED,
                                    emissive: Color::RED * 50.0,
                                    ..default()
                                }),
                                ..Default::default()
                            },
                            NotShadowCaster,
                        ))
                        .with_children(|target| {
                            target.spawn(PointLightBundle {
                                point_light: PointLight {
                                    color: Color::RED,
                                    shadows_enabled: true,
                                    range: 10.0,
                                    ..default()
                                },
                                transform: Transform::from_xyz(0.0, 1.5, 0.0),
                                ..default()
                            });
                        })
                        .id();
                    commands.entity(target_id).set_parent(target_e);
                    commands.entity(agent_e).insert(Path {
                        current: first.clone(),
                        next: remaining,
                    });
                    agent_targets.0 = Some(target_e);
                }
                return;
            }
        }
    }
}

fn move_object(
    mut commands: Commands,
    mut object_query: Query<(&mut Transform, &mut Path, Entity, &mut Object)>,
    mut sim_event: EventWriter<SimulateEvent>,
    time: Res<Time>,
) {
    for (mut transform, mut target, entity, mut object) in object_query.iter_mut() {
        let move_direction = target.current - transform.translation;
        transform.translation += move_direction.normalize() * time.delta_seconds() * 10.0;
        if transform.translation.distance(target.current) < 0.1 {
            if let Some(next) = target.next.pop() {
                target.current = next;
            } else {
                commands.entity(entity).remove::<Path>();
                let target_entity = object.0.take().unwrap();
                commands.entity(target_entity).despawn_recursive();

                // resend a sim event so we iterate all targets
                sim_event.send_default();
            }
        }
    }
}

// Simulate a factory floor:
// Object varients:
// - Conveyors: Have a direction vector, speed, collider
// - Sensors: Size, attached to robots
// - Factory Objects: Colliders
// - Robots: have sensors, do stuff when sensor fires
// robots go to fetch specific orders
// blender import of basic geometry of robots and structures

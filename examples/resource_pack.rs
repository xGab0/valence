#![allow(clippy::type_complexity)]

use valence::entity::player::PlayerEntityBundle;
use valence::entity::sheep::SheepEntityBundle;
use valence::prelude::*;
use valence_client::message::SendMessage;
use valence_client::resource_pack::{ResourcePackStatus, ResourcePackStatusEvent};

const SPAWN_Y: i32 = 64;

pub fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                init_clients,
                prompt_on_punch,
                on_resource_pack_status,
                despawn_disconnected_clients,
            ),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    server: Res<Server>,
    dimensions: Res<DimensionTypeRegistry>,
    biomes: Res<BiomeRegistry>,
) {
    let mut instance = Instance::new(ident!("overworld"), &dimensions, &biomes, &server);

    for z in -5..5 {
        for x in -5..5 {
            instance.insert_chunk([x, z], UnloadedChunk::new());
        }
    }

    for z in -25..25 {
        for x in -25..25 {
            instance.set_block([x, SPAWN_Y, z], BlockState::BEDROCK);
        }
    }

    let instance_ent = commands.spawn(instance).id();

    commands.spawn(SheepEntityBundle {
        location: Location(instance_ent),
        position: Position::new([0.0, SPAWN_Y as f64 + 1.0, 2.0]),
        look: Look::new(180.0, 0.0),
        head_yaw: HeadYaw(180.0),
        ..Default::default()
    });
}

fn init_clients(
    mut clients: Query<(Entity, &UniqueId, &mut Client, &mut GameMode), Added<Client>>,
    instances: Query<Entity, With<Instance>>,
    mut commands: Commands,
) {
    for (entity, uuid, mut client, mut game_mode) in &mut clients {
        *game_mode = GameMode::Creative;

        client.send_chat_message("Hit the sheep to prompt for the resource pack.".italic());

        commands.entity(entity).insert(PlayerEntityBundle {
            location: Location(instances.single()),
            position: Position::new([0.0, SPAWN_Y as f64 + 1.0, 0.0]),
            uuid: *uuid,
            ..Default::default()
        });
    }
}

fn prompt_on_punch(mut clients: Query<&mut Client>, mut events: EventReader<InteractEntityEvent>) {
    for event in events.iter() {
        if let Ok(mut client) = clients.get_mut(event.client) {
            if event.interact == EntityInteraction::Attack {
                client.set_resource_pack(
                    "https://github.com/valence-rs/valence/raw/main/assets/example_pack.zip",
                    "d7c6108849fb190ec2a49f2d38b7f1f897d9ce9f",
                    false,
                    None,
                );
            }
        };
    }
}

fn on_resource_pack_status(
    mut clients: Query<&mut Client>,
    mut events: EventReader<ResourcePackStatusEvent>,
) {
    for event in events.iter() {
        if let Ok(mut client) = clients.get_mut(event.client) {
            match event.status {
                ResourcePackStatus::Accepted => {
                    client.send_chat_message("Resource pack accepted.".color(Color::GREEN));
                }
                ResourcePackStatus::Declined => {
                    client.send_chat_message("Resource pack declined.".color(Color::RED));
                }
                ResourcePackStatus::FailedDownload => {
                    client.send_chat_message("Resource pack failed to download.".color(Color::RED));
                }
                ResourcePackStatus::SuccessfullyLoaded => {
                    client.send_chat_message(
                        "Resource pack successfully downloaded.".color(Color::BLUE),
                    );
                }
            }
        };
    }
}

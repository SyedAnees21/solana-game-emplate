use proto_interface::{AttributeId, EntityId, Value, updates::LocalUpdate};
use sdk::{attribute::ReservedAttributeId, session::Session};

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    // Starting two sessions
    let session_1 = Session::start("127.0.0.1:4000", &[]).await?;
    let session_2 = Session::start("127.0.0.1:4000", &[]).await?;
    
    // Entity spawned by the session 1
    let entity_1_id = session_1.derive_entity_id(1);
    // Entity spawned by the session 1
    let entity_2_id = session_2.derive_entity_id(1);

    // Helper to create a local update type to send
    let create_local_position_update = |id: EntityId| {
        LocalUpdate::Entity(
            id,
            ReservedAttributeId::Position as AttributeId,
            Value::Vec3(1., 2., 3.),
        )
    };
    
    // Session 1 sends its entity positio update
    session_1.send(create_local_position_update(entity_1_id))?;
    // Session 2 sends its entity positio update
    session_2.send(create_local_position_update(entity_2_id))?;

    loop {
        if let Some(server_update) = session_1.receive() {
            println!("Received session 2 entity at session 2 {:?}", server_update)
        }

         if let Some(server_update) = session_2.receive() {
            println!("Received session 1 entity at session 2 {:?}", server_update)
        }
    }
}

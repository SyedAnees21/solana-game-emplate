use crate::session::Session;
use prost::bytes::Bytes;
use proto_interface::outgoing::ClientOutGoing as ServerIncoming;
use proto_interface::{messages::ToClient, TickUpdate};
use sdk::sdkError;

pub async fn process_message(buffer: Bytes, session: &mut Session) -> Result<(), sdkError> {
    let tick = match ServerIncoming::from_bytes(buffer)? {
        ServerIncoming::Tick(tick_update) => tick_update,
        ServerIncoming::None => return Ok(()),
    };

    println!(
        "Received an update from client {:?} : {:?}",
        session.my_address(),
        tick
    );

    let _my_id = session.my_id();

    let global = session.global();
    let entities = global.entities();
    let snapshot = global.relevance_snap();
    let version_map = session.version_map_mut();

    for entity in tick.entities.into_iter() {
        let changes = entities.upsert(entity.id, entity.attrs)?;

        // TODO: Need to handle the changes observed after updating an entity.
        // Such as handle local version map for this session regarding the updated
        // entity.
        if !changes.is_empty() {
            version_map.submit_changes(entity.id, changes);
        }

        // TODO: This is just for managing the session snapshots manually.
        // This should later on be handled through a proper relevance scheme.
        // let my_snapshot = snapshot.get(&my_id).unwrap();
        // if !my_snapshot.contains(&entity.id) {
        //     my_snapshot.insert(entity.id);
        // }
        snapshot.insert(entity.id);
    }

    Ok(())
}

pub async fn create_updates(session: &mut Session) -> Result<Bytes, sdkError> {
    let global = session.global();
    let entities = global.entities();
    
    let version_map = session.version_map_mut();
    
    // TODO: Handle the entities properly after the relevance manage is completed
    // for now, we just are using a global entity container to create the updates.
    // let my_snapshot = relevance_snapshots.get(&session.my_id()).unwrap();
    let _relevance_snapshots = global.relevance_snap();
    let snapshot = global.relevance_snap();

    let mut tick_update = TickUpdate::default();

    for eid in snapshot.iter() {
        let Some(entity) = entities.get_entity(&eid) else {
            continue;
        };

        let versions = match version_map.get(*eid) {
            Some(version) => version,
            None => &Default::default(),
        };

        let latest_attributes = entity.get_attributes_by_version(versions)?;

        if versions.is_empty() {
            version_map.track_new(*eid, &latest_attributes);
        }

        if !latest_attributes.is_empty() {
            tick_update
                .entities
                .push(proto_interface::external::messages::Entity {
                    id: entity.id(),
                    attrs: latest_attributes,
                });
        }
    }

    if tick_update.entities.is_empty() {
        return Ok(Bytes::new());
    }

    let tick = ToClient::Tick(tick_update);
    Ok(tick.into_bytes()?)
}

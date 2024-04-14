use super::network::types as network_types;
use super::types;

impl From<types::MenuSelection> for network_types::NewRequestEntity {
    fn from(item_selection: types::MenuSelection) -> Self {
        match item_selection {
            types::MenuSelection::NewGame => network_types::NewRequestEntity::NewGame,
            types::MenuSelection::CreateRoom => network_types::NewRequestEntity::CreateRoom,
            types::MenuSelection::JoinRoom { room_id } => {
                network_types::NewRequestEntity::JoinRoom { room_id }
            }
        }
    }
}

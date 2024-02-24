use crate::app::model::AppStateUpdate;

pub mod bottom_bar;
pub mod help;
pub mod menu;
pub mod network_receptor;
pub mod room_details;

// Let's define the messages handled by our app. NOTE: it must derive `PartialEq`
#[derive(Debug, PartialEq)]
pub enum Msg {
    AppClose,
    NetworkUpdate,
    Menu(menu::MenuMessage),
    StateUpdate(AppStateUpdate),
    ReDraw,
}

// Let's define the component ids for our application
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub enum Id {
    Menu,
    BottomBar,
    RoomDetails,
    NetworkReceptor,
    Help,
}

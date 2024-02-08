pub mod bottom_bar;
// pub mod counter;
pub mod menu;

// Let's define the messages handled by our app. NOTE: it must derive `PartialEq`
#[derive(Debug, PartialEq)]
pub enum Msg {
    AppClose,
    Clock,
    StateUpdate,
    PingServer,
    SelectMenu(menu::Menus),
    JoinRoom(String),
}

// Let's define the component ids for our application
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub enum Id {
    Menu,
    BottomBar,
}

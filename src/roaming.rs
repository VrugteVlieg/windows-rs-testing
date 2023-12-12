
#[derive(Debug, Clone)]
pub enum UxiRoamEvent {
    Roam(RoamEvent),
    Reconnect(ReconnectEvent)
}
#[derive(Debug, Clone)]
pub enum RoamEvent {
    NoErrors,
    SomeErrors(Vec<String>),
    Disconnection(Vec<String>),
}
#[derive(Debug, Clone)]
pub enum ReconnectEvent {
    NoErrors,
    SomeErrors(Vec<String>),
    Failed(Vec<String>)
}
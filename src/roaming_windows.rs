use anyhow::anyhow;
use tokio::sync::broadcast::{Receiver, self};

use crate::{windows_type_wrappers::{WlanNotificationWrapper as NotificationSource, MsmNotifcationType, AcmNotifcationType, AcmNotificationDataWrapper}, roaming::{UxiRoamEvent, RoamEvent, ReconnectEvent}};


#[derive(Debug)]
enum AccessPointTransitionState {
    Init,
    Roam(ConnectionState),
    Reconnect(ConnectionState)
}
#[derive(Debug)]
enum ConnectionState {
    AttempingConnection,
    Authenticating(u8),
    UnqualifiedSuccess,
    QualifiedSuccess(Vec<String>),
    Failed(Vec<String>)
}


impl Default for AccessPointTransitionState {
    fn default() -> Self {
        AccessPointTransitionState::Init
    }
}


impl AccessPointTransitionState {
    fn is_terminal(&self) -> bool {
        match self {
            AccessPointTransitionState::Reconnect(state) | AccessPointTransitionState::Roam(state) => match state {
                ConnectionState::UnqualifiedSuccess => true,
                ConnectionState::QualifiedSuccess(_) => true,
                ConnectionState::Failed(_) => true,
                _ => false
            }
            AccessPointTransitionState::Init => false
        }
    }
}

#[rustfmt::skip]
fn compute_transition(current_state: &AccessPointTransitionState, event: NotificationSource) -> Option<AccessPointTransitionState> {
    Some(match (current_state, event) {
        //Roaming
        (AccessPointTransitionState::Init, NotificationSource::Msm(MsmNotifcationType::RoamingStart(_))) => AccessPointTransitionState::Roam(ConnectionState::AttempingConnection),
        (AccessPointTransitionState::Roam(roam_state), event) => AccessPointTransitionState::Roam({
            match (roam_state, event) {
                (ConnectionState::AttempingConnection, NotificationSource::Msm(MsmNotifcationType::Authenticating(_))) => ConnectionState::Authenticating(0),
                (ConnectionState::Authenticating(0), NotificationSource::Msm(MsmNotifcationType::RoamingEnd(_))) => ConnectionState::UnqualifiedSuccess,
                (ConnectionState::Authenticating(i), NotificationSource::Msm(MsmNotifcationType::RoamingStart(_))) => ConnectionState::Authenticating(i+1),
                (ConnectionState::Authenticating(i), NotificationSource::Msm(MsmNotifcationType::RoamingEnd(_))) => ConnectionState::QualifiedSuccess(vec![format!("{i} auth retries")]),
                (ConnectionState::Authenticating(i), NotificationSource::Msm(MsmNotifcationType::Disconnected(_))) => ConnectionState::Failed(vec![format!("Roam failed after {i} retries")]),

                //workaround for the roaming events that get emitted when you manually disconnect from a network or turn off your wifi
                (ConnectionState::AttempingConnection, NotificationSource::Msm(MsmNotifcationType::Disconnected(_))) => return Some(AccessPointTransitionState::default()),

                _ => return None
            }
        }),
        
        //Reconnect
        (AccessPointTransitionState::Init, NotificationSource::Acm(AcmNotifcationType::ConnectionStart(_))) => AccessPointTransitionState::Reconnect(ConnectionState::AttempingConnection),
        (AccessPointTransitionState::Reconnect(reconnect_state), event) => AccessPointTransitionState::Reconnect(
            match (reconnect_state, event) {
                (ConnectionState::AttempingConnection, NotificationSource::Msm(MsmNotifcationType::Authenticating(_))) => ConnectionState::Authenticating(0),
                (ConnectionState::Authenticating(0), NotificationSource::Acm(AcmNotifcationType::ConnectionComplete(AcmNotificationDataWrapper {operation_success: true, ..}))) => ConnectionState::UnqualifiedSuccess,
                (ConnectionState::Authenticating(i), NotificationSource::Msm(MsmNotifcationType::Authenticating(_))) => ConnectionState::Authenticating(i+1),
                (ConnectionState::Authenticating(i), NotificationSource::Acm(AcmNotifcationType::ConnectionComplete(AcmNotificationDataWrapper {operation_success: true, ..}))) => ConnectionState::QualifiedSuccess(vec![format!("{i} auth retries")]),
                (ConnectionState::Authenticating(i), NotificationSource::Acm(AcmNotifcationType::ConnectionComplete(AcmNotificationDataWrapper {operation_success: false, ..}))) => ConnectionState::Failed(vec![format!("Failed after {i} retries")]),
                _ => return None
        }),
        _ => return None
    })
}





pub fn create_uxi_roaming_channel(mut inlet: Receiver<NotificationSource>) -> Receiver<UxiRoamEvent> {
    let (tx, rx) = broadcast::channel::<UxiRoamEvent>(1);

    tokio::spawn(async move {
        let mut current_state = AccessPointTransitionState::default();
        loop {
            if let Ok(event) = inlet.recv().await {
                if let Some(new_state) = compute_transition(&current_state, event) {
                    if new_state.is_terminal() {
                        println!("{new_state:?} is terminal");
                        if let Ok(to_send) = UxiRoamEvent::try_from(new_state) {
                            println!("Sending {to_send:?}");
                            let _ = tx.send(to_send);
                        }
                        current_state = AccessPointTransitionState::default();
                    } else {
                        println!("Roaming state transition {:?} -> {:?}", current_state, new_state);
                        current_state = new_state;
                    }
                }
            }
        }
        
    });


    rx

}

impl TryFrom<AccessPointTransitionState> for UxiRoamEvent {
    type Error = anyhow::Error;

    fn try_from(value: AccessPointTransitionState) -> Result<Self, Self::Error> {
        Ok(match value {
            AccessPointTransitionState::Roam(roam_state) => match roam_state {
                ConnectionState::UnqualifiedSuccess => UxiRoamEvent::Roam(RoamEvent::NoErrors),
                ConnectionState::QualifiedSuccess(errors) => UxiRoamEvent::Roam(RoamEvent::SomeErrors(errors)),
                ConnectionState::Failed(errors) => UxiRoamEvent::Roam(RoamEvent::Disconnection(errors)),
                _ => return Err(anyhow!("Cannot map {roam_state:?} to UxiRoamEvent"))
            },
            AccessPointTransitionState::Reconnect(reconnect_state) => match reconnect_state {
                ConnectionState::UnqualifiedSuccess => UxiRoamEvent::Reconnect(ReconnectEvent::NoErrors),
                ConnectionState::QualifiedSuccess(errors) => UxiRoamEvent::Reconnect(ReconnectEvent::SomeErrors(errors)),
                ConnectionState::Failed(errors) => UxiRoamEvent::Reconnect(ReconnectEvent::Failed(errors)),
                _ => return Err(anyhow!("Cannot map {reconnect_state:?} to UxiRoamEvent"))
            }
            _ => return Err(anyhow!("Cannot map {value:?} to UxiRoamEvent"))
        })
    }
}
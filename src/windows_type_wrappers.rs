use anyhow::anyhow;
use windows::Win32::NetworkManagement::WiFi::{
    WlanReasonCodeToString, L2_NOTIFICATION_DATA, WLAN_MSM_NOTIFICATION_DATA, WLAN_CONNECTION_NOTIFICATION_DATA,
};

use crate::utils;


//https://learn.microsoft.com/en-us/previous-versions/windows/desktop/legacy/ms706902(v=vs.85)
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum WlanNotifcationSource {
    UNKNOWN,
    ONEX,
    ACM,
    MSM,
    SECURITY,
    IHV,
    HNWK,
    ALL,
}



impl TryFrom<u32> for WlanNotifcationSource {
    type Error = anyhow::Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        let out = match value {
            0 => Some(WlanNotifcationSource::UNKNOWN),
            0x4 => Some(WlanNotifcationSource::ONEX),
            0x8 => Some(WlanNotifcationSource::ACM),
            0x10 => Some(WlanNotifcationSource::MSM),
            0x20 => Some(WlanNotifcationSource::SECURITY),
            0x40 => Some(WlanNotifcationSource::IHV),
            0x80 => Some(WlanNotifcationSource::HNWK),
            0xFFFF => Some(WlanNotifcationSource::ALL),
            _ => None,
        };
        if let Some(value) = out {
            Ok(value)
        } else {
            Err(anyhow!("Invalid notifaction source {value}"))
        }
    }
}

//https://learn.microsoft.com/en-us/windows/win32/api/dot1x/ne-dot1x-onex_notification_type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OnexNotifcationType {
    ResultUpdate,
    AuthRestarted,
    EventInvalid,
}


impl TryFrom<L2_NOTIFICATION_DATA> for OnexNotifcationType {
    type Error = anyhow::Error;

    fn try_from(value: L2_NOTIFICATION_DATA) -> Result<Self, Self::Error> {
        match value.NotificationCode {
            1 => Ok(OnexNotifcationType::ResultUpdate),
            2 => Ok(OnexNotifcationType::AuthRestarted),
            3 => Ok(OnexNotifcationType::EventInvalid),
            _ => Err(anyhow!(
                "Invalid Onex Notifcation type code {}",
                value.NotificationCode
            )),
        }
    }
}

//https://learn.microsoft.com/en-us/windows/win32/api/wlanapi/ne-wlanapi-wlan_notification_acm-r1
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AcmNotifcationType {
    AutoconfEnabled,
    AutoconfDisabled,
    BackgroundScanEnabled,
    BackgroundScanDisabled,
    BSSTypeChange,
    PowerSettingChange,
    ScanComplete,
    ScanFail(String),
    ConnectionStart(AcmNotificationDataWrapper),
    ConnectionComplete(AcmNotificationDataWrapper),
    ConnectionAttemptFail,
    FilterListChange,
    InterfaceArrival,
    InterfaceRemoval,
    ProfileChange,
    ProfileNameChange,
    ProfilesExhausted,
    NetworkNotAvailable,
    NetworkAvailable,
    Disconnecting,
    Disconnected(AcmNotificationDataWrapper),
    AdhocNetworkStateChange,
    ProfileUnblocked,
    ScreenPowerChange,
    ProfileBlocked,
    ScanListRefresh,
    OperationalStateChange,
}

impl TryFrom<L2_NOTIFICATION_DATA> for AcmNotifcationType {
    type Error = anyhow::Error;

    fn try_from(value: L2_NOTIFICATION_DATA) -> Result<Self, Self::Error> {
        match value.NotificationCode {
            1 => Ok(AcmNotifcationType::AutoconfEnabled),
            2 => Ok(AcmNotifcationType::AutoconfDisabled),
            3 => Ok(AcmNotifcationType::BackgroundScanEnabled),
            4 => Ok(AcmNotifcationType::BackgroundScanDisabled),
            5 => Ok(AcmNotifcationType::BSSTypeChange),
            6 => Ok(AcmNotifcationType::PowerSettingChange),
            7 => Ok(AcmNotifcationType::ScanComplete),
            8 => {
                let buffer: [u16; 1024] = [0; 1024];
                let reason_string: String;
                unsafe {
                    let reason_code = value.pData as *const u32;
                    println!("Scan Fail Reason code: {}", *reason_code);
                    WlanReasonCodeToString(*reason_code, &buffer, None);
                    reason_string = String::from_utf16(&buffer).unwrap();
                }
                Ok(AcmNotifcationType::ScanFail(reason_string))
            }
            9 => Ok(AcmNotifcationType::ConnectionStart(AcmNotificationDataWrapper::from(value))),
            10 => Ok(AcmNotifcationType::ConnectionComplete(AcmNotificationDataWrapper::from(value))),
            11 => Ok(AcmNotifcationType::ConnectionAttemptFail),
            12 => Ok(AcmNotifcationType::FilterListChange),
            13 => Ok(AcmNotifcationType::InterfaceArrival),
            14 => Ok(AcmNotifcationType::InterfaceRemoval),
            15 => Ok(AcmNotifcationType::ProfileChange),
            16 => Ok(AcmNotifcationType::ProfileNameChange),
            17 => Ok(AcmNotifcationType::ProfilesExhausted),
            18 => Ok(AcmNotifcationType::NetworkNotAvailable),
            19 => Ok(AcmNotifcationType::NetworkAvailable),
            20 => Ok(AcmNotifcationType::Disconnecting),
            21 => Ok(AcmNotifcationType::Disconnected(AcmNotificationDataWrapper::from(value))),
            22 => Ok(AcmNotifcationType::AdhocNetworkStateChange),
            23 => Ok(AcmNotifcationType::ProfileUnblocked),
            24 => Ok(AcmNotifcationType::ScreenPowerChange),
            25 => Ok(AcmNotifcationType::ProfileBlocked),
            26 => Ok(AcmNotifcationType::ScanListRefresh),
            27 => Ok(AcmNotifcationType::OperationalStateChange),
            _ => Err(anyhow!(
                "Invalid Acm Notifcation type code {}",
                value.NotificationCode
            )),
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AcmNotificationDataWrapper {
    field: WLAN_CONNECTION_NOTIFICATION_DATA,
    pub operation_success: bool
}



impl From<L2_NOTIFICATION_DATA> for AcmNotificationDataWrapper {
    fn from(value: L2_NOTIFICATION_DATA) -> Self {
        unsafe {
            let data_ptr = value.pData as *const WLAN_CONNECTION_NOTIFICATION_DATA;
            let field = *data_ptr;
            AcmNotificationDataWrapper { field, operation_success: field.wlanReasonCode == 0 }
        }
    }
}


impl std::fmt::Display for AcmNotificationDataWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\nreason: {}", utils::parse_ssid(self.field.dot11Ssid), self.field.wlanReasonCode)
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WlanMsmNotifcationDataWrapper {
    field: Option<WLAN_MSM_NOTIFICATION_DATA>,
}


impl From<L2_NOTIFICATION_DATA> for WlanMsmNotifcationDataWrapper {
    fn from(value: L2_NOTIFICATION_DATA) -> Self {
        unsafe {
            let data_ptr = value.pData as *const WLAN_MSM_NOTIFICATION_DATA;
            WlanMsmNotifcationDataWrapper { field: Some(*data_ptr) }
        }
    }
}

impl std::fmt::Display for WlanMsmNotifcationDataWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(val) = self.field {
            write!(
                f,
                "{} @ {}\nprofile: {}\nreason: {}",
                utils::parse_ssid(val.dot11Ssid),
                utils::parse_bssid(val.dot11MacAddr),
                String::from_utf16(&val.strProfileName).unwrap(),
                val.wlanReasonCode
            )
        } else {
            write!(f, "WlanMsmNotifcationComparison")
        }
    }
}

//https://learn.microsoft.com/en-us/windows/win32/api/wlanapi/ne-wlanapi-wlan_notification_msm-r1
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MsmNotifcationType {
    Associating(WlanMsmNotifcationDataWrapper),
    Associated(WlanMsmNotifcationDataWrapper),
    Authenticating(WlanMsmNotifcationDataWrapper),
    Connected,
    RoamingStart(WlanMsmNotifcationDataWrapper),
    RoamingEnd(WlanMsmNotifcationDataWrapper),
    RadioStateChange,
    SignalQualityChange(u32),
    Disconnected(WlanMsmNotifcationDataWrapper),
    PeerJoin,
    PeerLeave,
    AdapterRemoval,
    AdapterOperationModeChange,
    LinkDegraded,
    LinkImproved,
    Disassociating,
}


impl TryFrom<L2_NOTIFICATION_DATA> for MsmNotifcationType {
    type Error = anyhow::Error;

    fn try_from(value: L2_NOTIFICATION_DATA) -> Result<Self, Self::Error> {
        match value.NotificationCode {
            1 => Ok(MsmNotifcationType::Associating(WlanMsmNotifcationDataWrapper::from(value))),
            2 => Ok(MsmNotifcationType::Associated(WlanMsmNotifcationDataWrapper::from(value))),
            3 => Ok(MsmNotifcationType::Authenticating(WlanMsmNotifcationDataWrapper::from(value))),
            4 => Ok(MsmNotifcationType::Connected),
            5 => Ok(MsmNotifcationType::RoamingStart(WlanMsmNotifcationDataWrapper::from(value))),
            6 => Ok(MsmNotifcationType::RoamingEnd(WlanMsmNotifcationDataWrapper::from(value))),
            7 => Ok(MsmNotifcationType::RadioStateChange),
            8 => unsafe {
                let data_ptr = value.pData as *const u32;
                Ok(MsmNotifcationType::SignalQualityChange(*data_ptr))
            },
            9 => Ok(MsmNotifcationType::Disassociating),
            10 => Ok(MsmNotifcationType::Disconnected(WlanMsmNotifcationDataWrapper::from(value))),
            11 => Ok(MsmNotifcationType::PeerJoin),
            12 => Ok(MsmNotifcationType::PeerLeave),
            13 => Ok(MsmNotifcationType::AdapterRemoval),
            14 => Ok(MsmNotifcationType::AdapterOperationModeChange),
            15 => Ok(MsmNotifcationType::LinkDegraded),
            16 => Ok(MsmNotifcationType::LinkImproved),
            _ => Err(anyhow!(
                "Invalid MSM notfication type code {}",
                value.NotificationCode
            )),
        }
    }
}

//https://learn.microsoft.com/en-gb/windows/win32/api/wlanapi/ne-wlanapi-wlan_hosted_network_notification_code?redirectedfrom=MSDN
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostedNetworkNoticationType {
    StateChange,
    PeerStateChange,
    RadioStateChange,
}


impl TryFrom<L2_NOTIFICATION_DATA> for HostedNetworkNoticationType {
    type Error = anyhow::Error;

    fn try_from(value: L2_NOTIFICATION_DATA) -> Result<Self, Self::Error> {
        match value.NotificationCode {
            4096 => Ok(HostedNetworkNoticationType::StateChange),
            4097 => Ok(HostedNetworkNoticationType::PeerStateChange),
            4098 => Ok(HostedNetworkNoticationType::RadioStateChange),
            _ => Err(anyhow!(
                "Invalid Hosted Network notfication type code {}",
                value.NotificationCode
            )),
        }
    }
}

#[derive(Debug, Clone,)]
pub enum WlanNotificationWrapper {
    Onex(OnexNotifcationType),
    Acm(AcmNotifcationType),
    Msm(MsmNotifcationType),
    Hnwk(HostedNetworkNoticationType),
    Other(WlanNotifcationSource, u32),
}

impl WlanNotificationWrapper {
    pub fn shallow_equals(&self, other: Self) -> bool {
        match (self, other) {
            (WlanNotificationWrapper::Onex(l0), WlanNotificationWrapper::Onex(r0)) => std::mem::discriminant(l0) == std::mem::discriminant(&r0),
            (WlanNotificationWrapper::Acm(l0), WlanNotificationWrapper::Acm(r0)) => std::mem::discriminant(l0) == std::mem::discriminant(&r0),
            (WlanNotificationWrapper::Msm(l0), WlanNotificationWrapper::Msm(r0)) => std::mem::discriminant(l0) == std::mem::discriminant(&r0),
            (WlanNotificationWrapper::Hnwk(l0), WlanNotificationWrapper::Hnwk(r0)) => std::mem::discriminant(l0) == std::mem::discriminant(&r0),
            (WlanNotificationWrapper::Other(_, _), WlanNotificationWrapper::Other(_, _)) => false,
            _ => false
        }
    }
}


impl std::fmt::Display for WlanNotificationWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WlanNotificationWrapper::Msm(m) => match m {
                MsmNotifcationType::Authenticating(data) => write!(f, "MSM::Authenticating:\n{data}"),
                MsmNotifcationType::RoamingStart(data) => write!(f, "MSM::Roam start:\n{data}"),
                MsmNotifcationType::RoamingEnd(data) => write!(f, "MSM::Roam end:\n{data}"),
                MsmNotifcationType::Disconnected(data) => write!(f, "MSM::Disconnected:\n{data}"),
                MsmNotifcationType::Associating(data) => write!(f, "MSM::Associating:\n{data}"),
                MsmNotifcationType::Associated(data) => write!(f, "MSM::Associated:\n{data}"),                
                _ => write!(f, "{self:?}"),
            },
            WlanNotificationWrapper::Acm(a) => match a {
                AcmNotifcationType::ConnectionStart(data) => write!(f, "ACM::ConnectionStart:\n{data}"),
                AcmNotifcationType::ConnectionComplete(data) => write!(f, "ACM::ConnectionComplete:\n{data}"),
                AcmNotifcationType::Disconnected(data) => write!(f, "ACM::Disconnected:\n{data}"),
                _ => write!(f, "{self:?}"),
            }
            _ => write!(f, "{self:?}"),
        }
    }
}

impl TryFrom<L2_NOTIFICATION_DATA> for WlanNotificationWrapper {
    type Error = anyhow::Error;

    fn try_from(notification_data: L2_NOTIFICATION_DATA) -> Result<Self, Self::Error> {
        let notification_source =
            WlanNotifcationSource::try_from(notification_data.NotificationSource)?;
        match notification_source {
            WlanNotifcationSource::ACM => Ok(WlanNotificationWrapper::Acm(
                AcmNotifcationType::try_from(notification_data)?,
            )),
            WlanNotifcationSource::ONEX => Ok(WlanNotificationWrapper::Onex(
                OnexNotifcationType::try_from(notification_data)?,
            )),
            WlanNotifcationSource::HNWK => Ok(WlanNotificationWrapper::Hnwk(
                HostedNetworkNoticationType::try_from(notification_data)?,
            )),
            // https://stackoverflow.com/questions/63916457/wlan-notification-msm-notificationcode-59
            WlanNotifcationSource::MSM => Ok(WlanNotificationWrapper::Msm(
                MsmNotifcationType::try_from(notification_data)?,
            )),
            _ => Err(anyhow!(
                "No valid Wlan Notifcation Type for ({:?}, {})",
                notification_source,
                notification_data.NotificationCode
            )),
        }
    }
}

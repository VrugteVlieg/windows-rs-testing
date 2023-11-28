use anyhow::anyhow;


//https://learn.microsoft.com/en-us/previous-versions/windows/desktop/legacy/ms706902(v=vs.85)
#[derive(Debug)]
pub enum WlanNotifcationSource {
    UNKNOWN,
    ONEX,
    ACM,
    MSM,
    SECURITY,
    IHV,
    HNWK,
    ALL
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
            _ => None
        };
        if let Some(value) = out {
            Ok(value)
        } else {
            Err(anyhow!("Invalid notifaction source {value}"))
        }
    }
}


//https://learn.microsoft.com/en-us/windows/win32/api/dot1x/ne-dot1x-onex_notification_type
#[derive(Debug)]
pub enum OnexNotifcationType {
    ResultUpdate,
    AuthRestarted,
    EventInvalid,

}

impl TryFrom<i32> for OnexNotifcationType {
    type Error = anyhow::Error;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(OnexNotifcationType::ResultUpdate),
            2 => Ok(OnexNotifcationType::AuthRestarted),
            3 => Ok(OnexNotifcationType::EventInvalid),
            _ => Err(anyhow!("Invalid Onex Notifcation type code {value}")),
        }
    }
}

//https://learn.microsoft.com/en-us/windows/win32/api/wlanapi/ne-wlanapi-wlan_notification_acm-r1
#[derive(Debug)]
pub enum AcmNotifcationType {
    AutoconfEnabled,
    AutoconfDisabled,
    BackgroundScanEnabled,
    BackgroundScanDisabled,
    BSSTypeChange,
    PowerSettingChange,
    ScanComplete,
    ScanFail,
    ConnectionStart,
    ConnectionComplete,
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
    Disconnected,
    AdhocNetworkStateChange,
    ProfileUnblocked,
    ScreenPowerChange,
    ProfileBlocked,
    ScanListRefresh,
    OperationalStateChange,
}


impl TryFrom<i32> for AcmNotifcationType {
    type Error = anyhow::Error;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(AcmNotifcationType::AutoconfEnabled),
            2 => Ok(AcmNotifcationType::AutoconfDisabled),
            3 => Ok(AcmNotifcationType::BackgroundScanEnabled),
            4 => Ok(AcmNotifcationType::BackgroundScanDisabled),
            5 => Ok(AcmNotifcationType::BSSTypeChange),
            6 => Ok(AcmNotifcationType::PowerSettingChange),
            7 => Ok(AcmNotifcationType::ScanComplete),
            8 => Ok(AcmNotifcationType::ScanFail),
            9 => Ok(AcmNotifcationType::ConnectionStart),
            10 => Ok(AcmNotifcationType::ConnectionComplete),
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
            21 => Ok(AcmNotifcationType::Disconnected),
            22 => Ok(AcmNotifcationType::AdhocNetworkStateChange),
            23 => Ok(AcmNotifcationType::ProfileUnblocked),
            24 => Ok(AcmNotifcationType::ScreenPowerChange),
            25 => Ok(AcmNotifcationType::ProfileBlocked),
            26 => Ok(AcmNotifcationType::ScanListRefresh),
            27 => Ok(AcmNotifcationType::OperationalStateChange),
            _ => Err(anyhow!("Invalid Acm Notifcation type code {value}")),
        }
    }
}

//https://learn.microsoft.com/en-us/windows/win32/api/wlanapi/ne-wlanapi-wlan_notification_msm-r1
#[derive(Debug)]
pub enum MsmNotifcationType {
    Associating,
    Associated,
    Authenticating,
    Connected,
    RoamingStart,
    RoamingEnd,
    RadioStateChange,
    SignalQualityChange,
    Disconnected,
    PeerJoin,
    PeerLeave,
    AdapterRemoval,
    AdapterOperationModeChange,
    LinkDegraded,
    LinkImproved,
    Disassociating
}

impl TryFrom<i32> for MsmNotifcationType {
    type Error = anyhow::Error;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(MsmNotifcationType::Associating),
            2 => Ok(MsmNotifcationType::Associated),
            3 => Ok(MsmNotifcationType::Authenticating),
            4 => Ok(MsmNotifcationType::Connected),
            5 => Ok(MsmNotifcationType::RoamingStart),
            6 => Ok(MsmNotifcationType::RoamingEnd),
            7 => Ok(MsmNotifcationType::RadioStateChange),
            8 => Ok(MsmNotifcationType::SignalQualityChange),
            9 => Ok(MsmNotifcationType::Disassociating),
            10 => Ok(MsmNotifcationType::Disconnected),
            11 => Ok(MsmNotifcationType::PeerJoin),
            12 => Ok(MsmNotifcationType::PeerLeave),
            13 => Ok(MsmNotifcationType::AdapterRemoval),
            14 => Ok(MsmNotifcationType::AdapterOperationModeChange),
            15 => Ok(MsmNotifcationType::LinkDegraded),
            16 => Ok(MsmNotifcationType::LinkImproved),
            _ => Err(anyhow!("Invalid MSM notfication type code {value}")),
        }
    }
}

//https://learn.microsoft.com/en-gb/windows/win32/api/wlanapi/ne-wlanapi-wlan_hosted_network_notification_code?redirectedfrom=MSDN
#[derive(Debug)]
pub enum HostedNetworkNoticationType {
    StateChange,
    PeerStateChange,
    RadioStateChange
}

impl TryFrom<i32> for HostedNetworkNoticationType {
    type Error = anyhow::Error;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            4096 => Ok(HostedNetworkNoticationType::StateChange),
            4097 => Ok(HostedNetworkNoticationType::PeerStateChange),
            4098 => Ok(HostedNetworkNoticationType::RadioStateChange),
            _ => Err(anyhow!("Invalid Hosted Network notfication type code {value}")),
        }
    }
}

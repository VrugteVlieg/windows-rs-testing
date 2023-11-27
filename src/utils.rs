use std::{any, ops::Range};

use windows::Win32::NetworkManagement::WiFi::DOT11_SSID;
use anyhow::anyhow;

pub fn map_freq_to_channel(freq: u32) -> u32 {
    let lower_bound_5_ghz = 5160_000;
    let lower_bound_2_ghz = 2412_000;
    let channel_center_separation = 5_000;

    if freq > lower_bound_5_ghz {
        32 + (freq - lower_bound_5_ghz) / channel_center_separation
    } else {
        1 + (freq - lower_bound_2_ghz) / channel_center_separation
    }
}

//convert the signal percentage to to dbm according to https://learn.microsoft.com/en-us/windows/win32/api/wlanapi/ns-wlanapi-wlan_available_network
pub fn interpolate_rssi(x: i32) -> i32 {
    let (x0, x1) = (0, 100);
    let (y0, y1) = (-100, -50);
    y1 + ((x - x1) * (y1 - y0) / (x1 - x0))
}

pub fn parse_ssid(input: DOT11_SSID) -> String {
    let ssid_len = input.uSSIDLength as usize;
    let ssid = String::from_utf8(input.ucSSID[0..ssid_len].to_vec())
        .unwrap_or("Unable to parse ssid".to_string());

    // println!("Parsed {} from {:#?}", ssid, input);

    return ssid;
}

pub fn parse_bssid(input: [u8; 6]) -> String {
    input.map(|e| format!("{e:02X}")).join(":")
}


pub fn create_dot_11_ssid_ptr(ssid: &str) -> *const DOT11_SSID {
    println!("Creating ssid struct from {ssid}");
    let mut ssid_buffer = [0_u8; 32];
    let ssid_len = ssid.len();
    ssid_buffer[0..ssid_len].copy_from_slice(ssid.as_bytes());
    let dot11_ssid_ptr: *const DOT11_SSID = &DOT11_SSID {
        uSSIDLength: ssid_len as u32,
        ucSSID: ssid_buffer
    };
    unsafe {
        println!("Created struct ptr @ {:p} {:?} ", dot11_ssid_ptr, *dot11_ssid_ptr);
    }

    dot11_ssid_ptr
}


pub fn create_dot_11_ssid(ssid: &str) -> DOT11_SSID {
    println!("Creating ssid struct from {ssid}");
    let mut ssid_buffer = [0_u8; 32];
    let ssid_len = ssid.len();
    ssid_buffer[0..ssid_len].copy_from_slice(ssid.as_bytes());
    let dot11_ssid = DOT11_SSID {
        uSSIDLength: ssid_len as u32,
        ucSSID: ssid_buffer
    };
    
    println!("Created struct {:?} ", dot11_ssid);
    

    dot11_ssid
}
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

trait NotifactionCodeResolver {}


// impl WlanNotifcationSource {
//     fn get_notifcation_type(self) -> Result<T: 
// }




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

impl NotifactionCodeResolver for OnexNotifcationType{}

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

impl NotifactionCodeResolver for AcmNotifcationType {}

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

impl NotifactionCodeResolver for MsmNotifcationType {}

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

impl NotifactionCodeResolver for HostedNetworkNoticationType {}





pub fn get_x_list_from_windows_x_list_struct<XListStruct, X: Copy>(list_ptr: *mut XListStruct, num_elements: u32) -> Vec<X> {
    unsafe {
        let base_pointer = (list_ptr.add(1) as *mut X).sub(1);
        let x_list: Vec<X> = Range {start: 0, end: num_elements}.map(|i| {
            let target_struct = *base_pointer.add(i as usize);

            target_struct
        }).collect();
        x_list
    }
}
use std::{collections::HashSet, time::Duration};

use chrono::{Utc, DateTime};
use windows::Win32::{
    Foundation::HANDLE,
    NetworkManagement::WiFi::{
        WlanCloseHandle, WlanEnumInterfaces, WlanGetAvailableNetworkList, WlanGetNetworkBssList,
        WlanOpenHandle, WlanRegisterNotification, WlanScan, DOT11_BSS_TYPE, DOT11_SSID,
        L2_NOTIFICATION_DATA, WLAN_AVAILABLE_NETWORK, WLAN_AVAILABLE_NETWORK_LIST, WLAN_BSS_ENTRY,
        WLAN_BSS_LIST, WLAN_INTERFACE_INFO, WLAN_INTERFACE_INFO_LIST,
    },
};

use crate::{
    utils::{self},
    windows_type_wrappers::{WlanNotificationWrapper, MsmNotifcationType},
    Network, roaming::UxiRoamEvent, roaming_windows,
};

use state::InitCell;

use crate::windows_type_wrappers::AcmNotifcationType;

static GLOBAL_WINDOWS_API_CLIENT: InitCell<WindowsApiClient> = InitCell::new();

use tokio::{sync::{broadcast, mpsc}, task::JoinHandle};

use anyhow::anyhow;

pub struct WindowsApiClient {
    handle: HANDLE,
    network_interface: WLAN_INTERFACE_INFO,
    _notification_logging_handle: JoinHandle<()>,
    notification_sender: broadcast::Sender<WlanNotificationWrapper>,
}

unsafe extern "system" fn notif_callback(
    param0: *mut L2_NOTIFICATION_DATA,
    _param1: *mut ::core::ffi::c_void,
) {
    let notifcation_data = *param0;

    if let Ok(parsed_notifcation) = WlanNotificationWrapper::try_from(notifcation_data) {
        let notifcation_sender = GLOBAL_WINDOWS_API_CLIENT.get().notification_sender.clone();
        match (notifcation_sender).send(parsed_notifcation) {
            Ok(_) => {},
            Err(e) => println!("Error while sending message:/n{:?}", e),
        }
    }
}

impl WindowsApiClient {
    pub fn init() {
        let mut handle: HANDLE = HANDLE::default();
        let mut client_version: u32 = 0;
        let mut interface_list_ptr: *mut WLAN_INTERFACE_INFO_LIST = std::ptr::null_mut();
        unsafe {
            WlanOpenHandle(
                2,
                None,
                &mut client_version as *mut u32,
                &mut handle as *mut HANDLE,
            );

            WlanEnumInterfaces(handle, None, &mut interface_list_ptr);
            let network_interfaces: Vec<WLAN_INTERFACE_INFO> =
                utils::get_x_list_from_windows_x_list_struct::<
                    WLAN_INTERFACE_INFO_LIST,
                    WLAN_INTERFACE_INFO,
                >(interface_list_ptr, (*interface_list_ptr).dwNumberOfItems);

            //https://learn.microsoft.com/en-us/windows/win32/api/wlanapi/nf-wlanapi-wlanregisternotification
            //Related to WlanNotifcationSource in windows_type_wrappers but the documentation treats them as separate types so I keep them separate
            let notification_source_all = 65535u32;

            // https://learn.microsoft.com/en-us/windows/win32/api/wlanapi/nc-wlanapi-wlan_notification_callback
            // We could potentially pass a pointer to some a Sender to publish notifcations when we receive them, haven't looked into this properly
            let callback_context = None;

            let (notification_sender, mut notification_receiver) =
                broadcast::channel::<WlanNotificationWrapper>(16);

            WlanRegisterNotification(
                handle,
                notification_source_all,
                false,
                Some(notif_callback),
                callback_context,
                None,
                None,
            );

            let notification_logging_handle = tokio::spawn(async move {
                loop {
                    match notification_receiver.recv().await {
                        Ok(notification) => {
                            if !matches!(notification, WlanNotificationWrapper::Msm(MsmNotifcationType::SignalQualityChange(_))) {
                                let time: DateTime<Utc> = chrono::DateTime::from(std::time::SystemTime::now());
                                println!("{} Windows notfication {notification}", time.format("%T"));
                            }

                        },
                        Err(e) => println!("Windows notification error {e:#?}")
                    }
                }
            });

            GLOBAL_WINDOWS_API_CLIENT.set(WindowsApiClient {
                handle,
                network_interface: *network_interfaces.first().unwrap(),
                _notification_logging_handle: notification_logging_handle,
                notification_sender,
            });

        }
    }

    fn retrieve_network_list() -> Vec<WLAN_AVAILABLE_NETWORK> {
        let api_client = GLOBAL_WINDOWS_API_CLIENT.get();
        unsafe {
            let mut network_list_ptr: *mut WLAN_AVAILABLE_NETWORK_LIST = std::ptr::null_mut();

            //This returns duplicates for networks that you have already connected to before, the networks that have a profile
            //https://github.com/jorgebv/windows-wifi-api/issues/7
            WlanGetAvailableNetworkList(
                api_client.handle,
                &api_client.network_interface.InterfaceGuid,
                3,
                None,
                &mut network_list_ptr,
            );

            let num_elements = (*network_list_ptr).dwNumberOfItems;
            let mut networks_ssid_set: HashSet<String> = HashSet::new();
            let mut networks = utils::get_x_list_from_windows_x_list_struct::<
                WLAN_AVAILABLE_NETWORK_LIST,
                WLAN_AVAILABLE_NETWORK,
            >(network_list_ptr, num_elements);

            //filter out the duplicate entries, as well as hidden networks which are displayed as having an empty ssid
            //Once you perform an AP scan for a given hidden network, its ssid is populated in future calls to WlanGetAvailableNetworkList
            networks.retain(|network| {
                let current_ssid = utils::parse_ssid(network.dot11Ssid);
                !current_ssid.is_empty() && networks_ssid_set.insert(current_ssid)
            });

            networks
        }
    }

    fn retrieve_bss_list(target_ssid: Option<DOT11_SSID>) -> Vec<WLAN_BSS_ENTRY> {
        let infrastructure_bss_type = 1;
        let api_client = GLOBAL_WINDOWS_API_CLIENT.get();
        unsafe {
            let mut network_bss_list_ptr: *mut WLAN_BSS_LIST = std::ptr::null_mut();

            if let Some(target_ssid) = target_ssid {
                let struct_ptr: *const DOT11_SSID = &target_ssid;
                WlanGetNetworkBssList(
                    api_client.handle,
                    &api_client.network_interface.InterfaceGuid,
                    Some(struct_ptr),
                    DOT11_BSS_TYPE(infrastructure_bss_type),
                    true,
                    None,
                    &mut network_bss_list_ptr,
                );

                let network_bss_list = *network_bss_list_ptr;
                let mut secured_bss_list =
                    utils::get_x_list_from_windows_x_list_struct::<WLAN_BSS_LIST, WLAN_BSS_ENTRY>(
                        network_bss_list_ptr,
                        network_bss_list.dwNumberOfItems,
                    );

                WlanGetNetworkBssList(
                    api_client.handle,
                    &api_client.network_interface.InterfaceGuid,
                    Some(struct_ptr),
                    DOT11_BSS_TYPE(infrastructure_bss_type),
                    false,
                    None,
                    &mut network_bss_list_ptr,
                );

                let network_bss_list = *network_bss_list_ptr;
                let mut open_bss_list =
                    utils::get_x_list_from_windows_x_list_struct::<WLAN_BSS_LIST, WLAN_BSS_ENTRY>(
                        network_bss_list_ptr,
                        network_bss_list.dwNumberOfItems,
                    );
                secured_bss_list.append(&mut open_bss_list);
                return secured_bss_list;
            } else {
                WlanGetNetworkBssList(
                    api_client.handle,
                    &api_client.network_interface.InterfaceGuid,
                    None,
                    DOT11_BSS_TYPE(infrastructure_bss_type),
                    false,
                    None,
                    &mut network_bss_list_ptr,
                );

                let network_bss_list = *network_bss_list_ptr;
                let bss_list =
                    utils::get_x_list_from_windows_x_list_struct::<WLAN_BSS_LIST, WLAN_BSS_ENTRY>(
                        network_bss_list_ptr,
                        network_bss_list.dwNumberOfItems,
                    );
                return bss_list;
            }
        }
    }


    async fn await_notification(
        target: WlanNotificationWrapper,
        timeout: Option<Duration>,
    ) -> Result<WlanNotificationWrapper, anyhow::Error> {
        let mut receiver = GLOBAL_WINDOWS_API_CLIENT
            .get()
            .notification_sender
            .subscribe();

        let operation = match timeout {
            Some(timeout) => tokio::time::sleep(timeout),
            None => tokio::time::sleep(Duration::MAX),
        };

        tokio::pin!(operation);

        loop {
            tokio::select! {
                _ = &mut operation => return Err(anyhow!("Await notifcation {:?} timed out", target.clone())),
                val = receiver.recv() => {
                    match val {
                        Ok(val) => {
                            if val.shallow_equals(target.clone()) {
                                return Ok(val)
                            }
                        },
                        Err(e) => return Err(anyhow!("Error while receiving notifcation {:?}\n{:#?}", target.clone(), e)),
                    }
                }
            }
        }
    }

    pub async fn ap_scan(target_ssid: Option<DOT11_SSID>) -> Vec<Network> {
        Self::trigger_ap_scan(target_ssid);
        let _ = Self::await_notification(
            WlanNotificationWrapper::Acm(AcmNotifcationType::ScanListRefresh),
            None,
        )
        .await;
        Self::retrieve_networks(target_ssid)
    }


    pub fn track_signal_changes() -> mpsc::Receiver<u32> {
        let (tx, rx) = mpsc::channel::<u32>(16);
        tokio::spawn(async move {
            loop {
                let val = WindowsApiClient::await_notification(WlanNotificationWrapper::Msm(MsmNotifcationType::SignalQualityChange(0)), None).await;
                if let Ok(WlanNotificationWrapper::Msm(MsmNotifcationType::SignalQualityChange(v))) = val {
                    let _ = tx.send(v);
                }
            }
        });
        rx

    }

    pub fn track_roaming_events() -> broadcast::Receiver<UxiRoamEvent> {
        roaming_windows::create_uxi_roaming_channel(GLOBAL_WINDOWS_API_CLIENT.get().notification_sender.subscribe())
        
    }
    fn retrieve_networks(target_ssid: Option<DOT11_SSID>) -> Vec<Network> {
        let bss_list = WindowsApiClient::retrieve_bss_list(target_ssid);
        let networks = WindowsApiClient::retrieve_network_list();

        let output_networks = networks
            .iter()
            .flat_map(|network| {
                bss_list
                    .iter()
                    .filter_map(|bss| {
                        if network.dot11Ssid == bss.dot11Ssid {
                            Some(Network::from((bss, network)))
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<Network>>()
            })
            .collect::<Vec<Network>>();

        output_networks
    }

    fn trigger_ap_scan(target_ssid: Option<DOT11_SSID>) {
        let api_client = GLOBAL_WINDOWS_API_CLIENT.get();
        unsafe {
            if let Some(target_ssid) = target_ssid {
                println!(
                    "Triggering targeted ap scan for {}",
                    utils::parse_ssid(target_ssid)
                );
                let struct_ptr: *const DOT11_SSID = &target_ssid;
                WlanScan(
                    api_client.handle,
                    &api_client.network_interface.InterfaceGuid,
                    Some(struct_ptr),
                    None,
                    None,
                );
            } else {
                WlanScan(
                    api_client.handle,
                    &api_client.network_interface.InterfaceGuid,
                    None,
                    None,
                    None,
                );
            }
        }
    }
}

impl Drop for WindowsApiClient {
    fn drop(&mut self) {
        unsafe {
            WlanCloseHandle(self.handle, None);
        }
    }
}

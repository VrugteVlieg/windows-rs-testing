use std::{collections::HashSet, sync::mpsc::Receiver};

use windows::Win32::{NetworkManagement::WiFi::{WlanOpenHandle, WLAN_INTERFACE_INFO_LIST, L2_NOTIFICATION_DATA, WLAN_INTERFACE_INFO, WlanEnumInterfaces, WlanRegisterNotification, WlanGetAvailableNetworkList, WLAN_AVAILABLE_NETWORK_LIST, WlanGetNetworkBssList, DOT11_BSS_TYPE, WLAN_BSS_LIST, WlanCloseHandle, WlanScan, DOT11_SSID, WLAN_BSS_ENTRY, WLAN_AVAILABLE_NETWORK}, Foundation::HANDLE};

use crate::{utils::{self}, Network};

use crate::windows_type_wrappers::{WlanNotifcationSource, AcmNotifcationType, OnexNotifcationType, HostedNetworkNoticationType, MsmNotifcationType};



struct WindowsNotifcation {
    notifcation_source: WlanNotifcationSource,
    notifcation_reason: i32,
    notifcation_string: String
}

pub struct WindowsApiClient {
    handle: HANDLE,
    network_interface: WLAN_INTERFACE_INFO,
    notification_receiver: Receiver<WindowsNotifcation>
}

unsafe extern "system" fn notif_callback(param0: *mut L2_NOTIFICATION_DATA, _param1: *mut ::core::ffi::c_void) {
    let notifcation_data = *param0;
    println!("Callback triggerd: {:?}", notifcation_data);
    let notif_reason = WlanNotifcationSource::try_from(notifcation_data.NotificationSource).unwrap();
    println!("Reason: {:?}", notif_reason);
    let notif_code = notifcation_data.NotificationCode as i32;
    let notifcation_type = match notif_reason {
        WlanNotifcationSource::ACM => Some(format!("{:?}", AcmNotifcationType::try_from(notif_code).unwrap())),
        WlanNotifcationSource::ONEX => Some(format!("{:?}", OnexNotifcationType::try_from(notif_code).unwrap())),
        WlanNotifcationSource::HNWK => Some(format!("{:?}", HostedNetworkNoticationType::try_from(notif_code).unwrap())),
        // https://stackoverflow.com/questions/63916457/wlan-notification-msm-notificationcode-59
        WlanNotifcationSource::MSM if notif_code != 59 => Some(format!("{:?}", MsmNotifcationType::try_from(notif_code).unwrap())),
        _ => None
    };
    if let Some(notification_string) = notifcation_type {
        println!("Notifcation String: {}", notification_string);
    }
}


impl WindowsApiClient {
    pub fn init() -> Self {
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
            let network_interfaces: Vec<WLAN_INTERFACE_INFO> = utils::get_x_list_from_windows_x_list_struct::<WLAN_INTERFACE_INFO_LIST, WLAN_INTERFACE_INFO>(interface_list_ptr, (*interface_list_ptr).dwNumberOfItems);

            //https://learn.microsoft.com/en-us/windows/win32/api/wlanapi/nf-wlanapi-wlanregisternotification
            //Related to WlanNotifcationSource in windows_type_wrappers but the documentation treats them as separate types so I keep them separate
            let notification_source_all = 65535u32;
            
            // https://learn.microsoft.com/en-us/windows/win32/api/wlanapi/nc-wlanapi-wlan_notification_callback
            // We could potentially pass a pointer to some a Sender to publish notifcations when we receive them, haven't looked into this properly
            let callback_context = None;

            let (notfication_sender, notification_receiver) = std::sync::mpsc::channel::<WindowsNotifcation>();

            

            WlanRegisterNotification(handle, notification_source_all, false, Some(notif_callback), callback_context, None, None);

            WindowsApiClient { handle, network_interface: *network_interfaces.first().unwrap(), notification_receiver}
        }

    }

    fn retrieve_network_list(&self) -> Vec<WLAN_AVAILABLE_NETWORK> {
        unsafe {
            let mut network_list_ptr: *mut WLAN_AVAILABLE_NETWORK_LIST = std::ptr::null_mut();
        
            //This returns duplicates for networks that you have already connected to before, the networks that have a profile
            //https://github.com/jorgebv/windows-wifi-api/issues/7
            WlanGetAvailableNetworkList(
                self.handle,
                &self.network_interface.InterfaceGuid,
                3,
                None,
                &mut network_list_ptr,
            );

            let num_elements = (*network_list_ptr).dwNumberOfItems;
            let mut networks_ssid_set: HashSet<String> = HashSet::new();
            let mut networks = utils::get_x_list_from_windows_x_list_struct::<WLAN_AVAILABLE_NETWORK_LIST, WLAN_AVAILABLE_NETWORK>(network_list_ptr, num_elements);


            //filter out the duplicate entries, as well as hidden networks which are displayed as having an empty ssid
            //Once you perform an AP scan for a given hidden network, its ssid is populated in future calls to WlanGetAvailableNetworkList
            networks.retain(|network| {
                let current_ssid = utils::parse_ssid(network.dot11Ssid);
                !current_ssid.is_empty() && networks_ssid_set.insert(current_ssid)
            });

            networks
        }
    }

    pub fn retrieve_bss_list(&self, target_ssid: Option<DOT11_SSID>) -> Vec<WLAN_BSS_ENTRY> {
        let infrastructure_bss_type = 1;
        unsafe {
            let mut network_bss_list_ptr: *mut WLAN_BSS_LIST = std::ptr::null_mut();

            if let Some(target_ssid) = target_ssid {
                let struct_ptr: *const DOT11_SSID = &target_ssid;
                WlanGetNetworkBssList(
                    self.handle,
                    &self.network_interface.InterfaceGuid,
                    Some(struct_ptr),
                    DOT11_BSS_TYPE(infrastructure_bss_type),
                    true,
                    None,
                    &mut network_bss_list_ptr,
                );

                let network_bss_list = *network_bss_list_ptr;
                let mut secured_bss_list = utils::get_x_list_from_windows_x_list_struct::<WLAN_BSS_LIST, WLAN_BSS_ENTRY>(network_bss_list_ptr, network_bss_list.dwNumberOfItems);

                WlanGetNetworkBssList(
                    self.handle,
                    &self.network_interface.InterfaceGuid,
                    Some(struct_ptr),
                    DOT11_BSS_TYPE(infrastructure_bss_type),
                    false,
                    None,
                    &mut network_bss_list_ptr,
                );

                let network_bss_list = *network_bss_list_ptr;
                let mut open_bss_list = utils::get_x_list_from_windows_x_list_struct::<WLAN_BSS_LIST, WLAN_BSS_ENTRY>(network_bss_list_ptr, network_bss_list.dwNumberOfItems);
                secured_bss_list.append(&mut open_bss_list);
                return secured_bss_list;

            } else {
                WlanGetNetworkBssList(
                    self.handle,
                    &self.network_interface.InterfaceGuid,
                    None,
                    DOT11_BSS_TYPE(infrastructure_bss_type),
                    false,
                    None,
                    &mut network_bss_list_ptr,
                );

                let network_bss_list = *network_bss_list_ptr;
                let bss_list = utils::get_x_list_from_windows_x_list_struct::<WLAN_BSS_LIST, WLAN_BSS_ENTRY>(network_bss_list_ptr, network_bss_list.dwNumberOfItems);
                return bss_list;

            }
        }
    }

    pub fn retrieve_networks(&self, target_ssid: Option<DOT11_SSID>) -> Vec<Network> {
        let bss_list= self.retrieve_bss_list(target_ssid);
        let networks = self.retrieve_network_list();

        let output_networks = networks.iter().flat_map(|network| {
            bss_list.iter().filter_map(|bss| {
                if network.dot11Ssid == bss.dot11Ssid {
                    Some(Network::from((bss, network)))
                } else {
                    None
                }
            }).collect::<Vec<Network>>()
        }).collect::<Vec<Network>>();
        

        output_networks
    }


    pub fn trigger_ap_scan(&self, target_ssid: Option<DOT11_SSID>) {
        unsafe {
            if let Some(target_ssid) = target_ssid {
                println!("Triggering targeted ap scan for {}", utils::parse_ssid(target_ssid));
                let struct_ptr: *const DOT11_SSID = &target_ssid;
                WlanScan(self.handle, &self.network_interface.InterfaceGuid, Some(struct_ptr), None, None);
            } else {
                WlanScan(self.handle, &self.network_interface.InterfaceGuid, None, None, None);
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
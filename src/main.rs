pub mod bss_entry_list;
pub mod available_network_list;
pub mod utils;
pub mod windows_manager;

use windows_manager::WindowsApiClient;

fn main() {

        let windows_client = WindowsApiClient::init();

        let target_ssid= Some("HosHos");
        let mut counter = 0;

        while counter < 5 {
            counter+=1;

            let before_scan_full_bss_list = windows_client.retrieve_bss_list(None);
            println!("Before scan:");
            let before_scan_directed_bss_list = windows_client.retrieve_bss_list(target_ssid.map(utils::create_dot_11_ssid));
            println!("Targeted list({})\n{}\n", before_scan_directed_bss_list.parsed_networks.len(), before_scan_directed_bss_list);
            windows_client.trigger_ap_scan_too(target_ssid.map(utils::create_dot_11_ssid));
            //Scans are guaranteed to be done after 4 seconds according to the MS spec 
            std::thread::sleep(std::time::Duration::from_secs(5));
    
            let post_scan_full_bss_list = windows_client.retrieve_bss_list(None);
            let post_scan_directed_bss_list = windows_client.retrieve_bss_list(target_ssid.map(utils::create_dot_11_ssid));
    
            println!("Post scan:\nTargeted list ({})\n{}", post_scan_directed_bss_list.parsed_networks.len(), post_scan_directed_bss_list);
        }
}
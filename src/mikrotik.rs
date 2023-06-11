use router_os::ApiRos;
use std::collections::HashMap;
use std::net::TcpStream;

pub struct Api<'a> {
    ros: ApiRos<'a>,
}

impl<'a> Api<'a> {
    pub fn new(stream: &'a mut TcpStream, username: String, password: String) -> Api<'a> {
        let mut ros = ApiRos::new(stream);
        ros.login(username, password);
        Api { ros }
    }

    pub fn dhcp_table(&'a mut self) -> Vec<DhcpEntry> {
        self.records("/ip/dhcp-server/lease/print")
            .map(DhcpEntry::parse)
            .collect()
    }

    pub fn external_ip(&'a mut self, interface_name: &str) -> Option<String> {
        let record = self
            .table_map("/ip/address/print")
            .into_iter()
            .find(|item| {
                item.get("interface")
                    .map_or_else(|| false, |v| v == interface_name)
            });
        match record {
            Some(r) => r
                .get("address")
                .map(|i| i.split('/').next().unwrap().to_string()),
            _ => None,
        }
    }

    pub fn dump_table(&'a mut self, path: &str) {
        println!("{:#?}", self.table_map(path))
    }

    pub fn table_map(&'a mut self, path: &str) -> Vec<HashMap<String, String>> {
        self.records(path)
            .map(|record| {
                record
                    .into_iter()
                    .filter_map(|row| {
                        let parts: Vec<&str> = row.splitn(3, '=').collect();
                        if parts.len() == 3 {
                            Some((parts[1].to_string(), parts[2].to_string()))
                        } else {
                            None
                        }
                    })
                    .collect()
            })
            .collect()
    }

    pub fn records(&'a mut self, path: &str) -> RecordIter {
        RecordIter::new(self, path)
    }
}

pub struct RecordIter<'a> {
    api: &'a mut Api<'a>,
}

impl<'a> RecordIter<'a> {
    fn new(api: &'a mut Api<'a>, path: &str) -> RecordIter<'a> {
        api.ros.write_sentence(vec![path.into()]);
        RecordIter { api }
    }
}

impl<'a> Iterator for RecordIter<'a> {
    type Item = Vec<String>;

    fn next(self: &mut RecordIter<'a>) -> Option<Self::Item> {
        let values = self.api.ros.read_sentence();
        if values
            .get(0)
            .filter(|&v| v == "!done" || v == "!trap")
            .is_some()
        {
            None
        } else {
            Some(values)
        }
    }
}

#[derive(Debug)]
pub struct DhcpEntry {
    pub address: String,
    pub mac_address: String,
    pub client_id: String,
    pub address_lists: String,
    pub server: String,
    pub dhcp_option: String,
    pub status: String,
    pub expires_after: String,
    pub last_seen: String,
    pub active_address: String,
    pub active_mac_address: String,
    pub active_client_id: String,
    pub active_server: String,
    pub host_name: String,
    pub radius: bool,
    pub dynamic: bool,
    pub blocked: bool,
    pub disabled: bool,
    pub comment: String,
}

impl DhcpEntry {
    fn parse(sentence: Vec<String>) -> DhcpEntry {
        let mut entry = DhcpEntry {
            address: String::from(""),
            mac_address: String::from(""),
            client_id: String::from(""),
            address_lists: String::from(""),
            server: String::from(""),
            dhcp_option: String::from(""),
            status: String::from(""),
            expires_after: String::from(""),
            last_seen: String::from(""),
            active_address: String::from(""),
            active_mac_address: String::from(""),
            active_client_id: String::from(""),
            active_server: String::from(""),
            host_name: String::from(""),
            radius: false,
            dynamic: false,
            blocked: false,
            disabled: false,
            comment: String::from(""),
        };

        for line in sentence {
            if !line.starts_with('=') {
                continue;
            }
            let parts: Vec<&str> = line.splitn(3, '=').collect();
            if parts.len() != 3 {
                continue;
            }
            let key = parts[1];
            let value = parts[2];
            match key {
                "address" => entry.address = value.to_string(),
                "mac-address" => entry.mac_address = value.to_string(),
                "client-id" => entry.client_id = value.to_string(),
                "address-lists" => entry.address_lists = value.to_string(),
                "server" => entry.server = value.to_string(),
                "dhcp-option" => entry.dhcp_option = value.to_string(),
                "status" => entry.status = value.to_string(),
                "expires-after" => entry.expires_after = value.to_string(),
                "last-seen" => entry.last_seen = value.to_string(),
                "active-address" => entry.active_address = value.to_string(),
                "active-mac-address" => entry.active_mac_address = value.to_string(),
                "active-client-id" => entry.active_client_id = value.to_string(),
                "active-server" => entry.active_server = value.to_string(),
                "host-name" => entry.host_name = value.to_string(),
                "radius" => entry.radius = parse_boolean(value),
                "dynamic" => entry.dynamic = parse_boolean(value),
                "blocked" => entry.blocked = parse_boolean(value),
                "disabled" => entry.disabled = parse_boolean(value),
                "comment" => entry.comment = value.to_string(),
                _ => continue,
            }
        }

        entry
    }
}

fn parse_boolean(val: &str) -> bool {
    val == "true"
}

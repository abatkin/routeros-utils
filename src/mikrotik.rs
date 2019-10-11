use router_os::ApiRos;
use std::net::TcpStream;

rental! {
    pub mod rentals {
        use std::net::TcpStream;
        use router_os::ApiRos;

        #[rental_mut]
        pub struct Wrapper {
            stream: Box<TcpStream>,
            ros: ApiRos<'stream>,
        }
    }
}

pub struct Api {
    wrapper: rentals::Wrapper,
}

impl Api {
    pub fn new(host: String, port: u16, username: String, password: String) -> Api {
        Api {
            wrapper: rentals::Wrapper::new(
                Box::new(TcpStream::connect(format!("{}:{}", host, port)).unwrap()),
                |s| {
                    let mut ros = ApiRos::new(s, false);
                    ros.login(username, password);
                    ros
                },
            ),
        }
    }

    pub fn dhcp_table(&mut self) -> Vec<DhcpEntry> {
        self.wrapper.rent_mut(|ros| {
            ros.write_sentence(vec!["/ip/dhcp-server/lease/print".into()]);
            let mut records: Vec<DhcpEntry> = Vec::new();
            loop {
                let record = ros.read_sentence();
                if record.len() <= 1 {
                    break;
                }

                records.push(DhcpEntry::parse(record));
            }
            records
        })
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

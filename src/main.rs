#[macro_use]
extern crate prettytable;
use prettytable::{format, Table};
use std::net::TcpStream;
use structopt::StructOpt;

mod mikrotik;
use mikrotik::Api;

#[derive(StructOpt)]
struct Cli {
    /// Hostname to connect to
    #[structopt(long)]
    host: String,

    /// TCP port to connect to
    #[structopt(long, default_value = "8728")]
    port: u16,

    /// API username
    #[structopt(long)]
    username: String,

    /// API password
    #[structopt(long)]
    password: String,

    #[structopt(long, default_value = "dhcp-table")]
    command: String,

    #[structopt(long, name = "interface-name", default_value = "ether1")]
    interface_name: String,
}

fn main() {
    let args = Cli::from_args();
    let mut stream = TcpStream::connect(format!("{}:{}", args.host, args.port)).unwrap();
    let mut api = Api::new(&mut stream, args.username, args.password);

    match args.command.as_ref() {
        "dhcp-table" => dump_dhcp_table(&mut api),
        "external-ip" => dump_external_ip(&mut api, &args.interface_name),
        cmd => api.dump_table(cmd),
    }
}

fn dump_external_ip<'a>(api: &'a mut Api<'a>, interface_name: &str) {
    let ip = api.external_ip(interface_name);
    match ip {
        Some(addr) => println!("{}", addr),
        None => println!("Not found"),
    }
}

fn dump_dhcp_table<'a>(api: &'a mut Api<'a>) {
    let records = api.dhcp_table();

    //TODO: this doesn't work so well (need to sort each octet separately)
    //records.sort_by(|a, b| a.address.cmp(&b.address));

    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
    table.set_titles(row![b->"IP Address", b->"Hostname", b->"Comment", b->"MAC Address", b->"Last Seen", b->"Expires", b->"Status"]);

    for record in records {
        table.add_row(row![
            record.address,
            record.host_name,
            record.comment,
            record.mac_address,
            record.last_seen,
            record.expires_after,
            record.status
        ]);
    }

    table.printstd();
}

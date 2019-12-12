#[macro_use]
extern crate prettytable;
use prettytable::{format, Table};
use structopt::StructOpt;

mod mikrotik;
use mikrotik::Api;

#[macro_use]
extern crate rental;

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
}

fn main() {
    let args = Cli::from_args();
    let mut api = Api::new(args.host, args.port, args.username, args.password);

    match args.command.as_ref() {
        "dhcp-table" => dump_dhcp_table(&mut api),
        "external-ip" => dump_external_ip(&mut api),
        cmd => api.dump_table(cmd),
//        cmd => println!("Invalid command: {}", cmd),
    }
}

fn dump_external_ip(api: &mut Api) {
    api.external_ip();
}

fn dump_dhcp_table(api: &mut Api) {
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

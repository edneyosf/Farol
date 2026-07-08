mod args;
mod ip;
mod cloudflare;

use crate::cloudflare::{find_dns_record, update_dns_record, CfDnsRecord};
use crate::{args::Args, ip::get_public_ip};

///   CF_API_TOKEN   - Cloudflare API token (requires Zone.DNS:Edit permission) [--api-token]
///   CF_ZONE_ID     - Zone ID of the target zone                               [--zone-id]
///   CF_RECORD_NAME - Fully qualified record name, e.g. home.mydomain.com      [--record-name]
///   CF_RECORD_TYPE - Record type (default: A)                                 [--record-type]
///   CF_PROXIED     - "true"/"false" (default: false)                          [--proxied]
///   CF_TTL         - TTL in seconds; 1 = auto (default: 1)                    [--ttl]

fn main() -> Result<(), String> {
    let args = Args::parse()?;

    match get_public_ip() {
        Some(ip) => {
            println!("Current public IP: {ip}"); 
        
            match find_dns_record(&args)? {
                Some(record) if record.content == ip => already_updated(&record.name, &ip),
                Some(record) => update(&args, &record, &ip)?,
                None => eprintln!("No record '{}' found", args.record_name)
            }       
        }
        None => eprintln!("No IP found")
    }

    Ok(())
}

fn already_updated(record: &str, ip: &str) {
    println!("Record '{record}' already points to {ip}");
}

fn update(args: &Args, record: &CfDnsRecord, ip: &str) -> Result<(), String> {
    let name = &record.name;
    let content = &record.content;
    
    println!("Record '{name}' points to {content} (outdated). Updating to {ip}...");
    update_dns_record(args, &record.id, ip)?;
    println!("Record successfully updated");

    Ok(())
}
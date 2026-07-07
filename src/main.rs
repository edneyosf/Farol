mod args;
mod ip;
mod cloudflare;

use crate::cloudflare::{create_dns_record, find_dns_record, update_dns_record};
use crate::{args::Args, ip::get_public_ip};

///   CF_API_TOKEN   - Cloudflare API token (requires Zone.DNS:Edit permission) [--api-token]
///   CF_ZONE_ID     - Zone ID of the target zone                               [--zone-id]
///   CF_RECORD_NAME - Fully qualified record name, e.g. home.mydomain.com      [--record-name]
///   CF_RECORD_TYPE - Record type (default: A)                                 [--record-type]
///   CF_PROXIED     - "true"/"false" (default: false)                          [--proxied]
///   CF_TTL         - TTL in seconds; 1 = auto (default: 1)                    [--ttl]

fn main() -> Result<(), String> {
    let args = Args::parse()?;
    let current_ip = get_public_ip()?;

    println!("Current public IP: {current_ip}");

    match find_dns_record(&args)? {
        Some(record) if record.content == current_ip => {
            println!(
                "Registro '{}' já aponta para {}. Nada a fazer.",
                record.name, current_ip
            );
        }
        Some(record) => {
            println!(
                "Registro '{}' aponta para {} (desatualizado). Atualizando para {}...",
                record.name, record.content, current_ip
            );
            update_dns_record(&args, &record.id, &current_ip)?;
            println!("Registro atualizado com sucesso.");
        }
        None => {
            println!(
                "Nenhum registro '{}' encontrado. Criando novo registro apontando para {}...",
                args.record_name, current_ip
            );
            create_dns_record(&args, &current_ip)?;
            println!("Registro criado com sucesso.");
        }
    }

    Ok(())
}
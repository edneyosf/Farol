use serde::{Deserialize, Serialize};
use std::env;
use std::process::ExitCode;

/// Atualiza um registro DNS (tipo A) na Cloudflare com o IP público atual desta máquina.
/// Funciona como um "DDNS" simples: só faz PATCH se o IP realmente mudou.
///
/// Configuração via variáveis de ambiente (ou flags equivalentes):
///   CF_API_TOKEN   - Token de API da Cloudflare (permissão Zone.DNS:Edit)   [--api-token]
///   CF_ZONE_ID     - Zone ID da zona alvo                                   [--zone-id]
///   CF_RECORD_NAME - Nome completo do registro, ex: casa.meudominio.com     [--record-name]
///   CF_RECORD_TYPE - Tipo do registro (padrão: A)                          [--record-type]
///   CF_PROXIED     - "true"/"false" (padrão: false)                        [--proxied]
///   CF_TTL         - TTL em segundos, 1 = automático (padrão: 1)           [--ttl]

struct Args {
    api_token: String,
    zone_id: String,
    record_name: String,
    record_type: String,
    proxied: bool,
    ttl: u32,
}

impl Args {
    fn parse() -> Result<Self, String> {
        let mut api_token = env::var("CF_API_TOKEN").ok();
        let mut zone_id = env::var("CF_ZONE_ID").ok();
        let mut record_name = env::var("CF_RECORD_NAME").ok();
        let mut record_type = env::var("CF_RECORD_TYPE").unwrap_or_else(|_| "A".to_string());
        let mut proxied = env::var("CF_PROXIED")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false);
        let mut ttl: u32 = env::var("CF_TTL")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(1);

        let mut args = env::args().skip(1);
        while let Some(flag) = args.next() {
            let mut next_val = || {
                args.next()
                    .ok_or_else(|| format!("valor faltando após {flag}"))
            };
            match flag.as_str() {
                "--api-token" => api_token = Some(next_val()?),
                "--zone-id" => zone_id = Some(next_val()?),
                "--record-name" => record_name = Some(next_val()?),
                "--record-type" => record_type = next_val()?,
                "--ttl" => ttl = next_val()?.parse().map_err(|_| "--ttl inválido".to_string())?,
                "--proxied" => proxied = true,
                "-h" | "--help" => {
                    print_help();
                    std::process::exit(0);
                }
                other => return Err(format!("flag desconhecida: {other}")),
            }
        }

        Ok(Args {
            api_token: api_token.ok_or("defina CF_API_TOKEN ou use --api-token")?,
            zone_id: zone_id.ok_or("defina CF_ZONE_ID ou use --zone-id")?,
            record_name: record_name.ok_or("defina CF_RECORD_NAME ou use --record-name")?,
            record_type,
            proxied,
            ttl,
        })
    }
}

fn print_help() {
    println!(
        r#"cf-ddns - atualiza um registro DNS na Cloudflare com o IP público atual

USO:
  cf-ddns [--api-token TOKEN] [--zone-id ID] [--record-name NOME]
          [--record-type TIPO] [--proxied] [--ttl SEGUNDOS]

Cada flag também pode ser definida por variável de ambiente:
  CF_API_TOKEN, CF_ZONE_ID, CF_RECORD_NAME, CF_RECORD_TYPE, CF_PROXIED, CF_TTL
"#
    );
}

#[derive(Deserialize, Debug)]
struct IpifyResponse {
    ip: String,
}

#[derive(Deserialize, Debug)]
struct CfListResponse {
    success: bool,
    result: Vec<CfDnsRecord>,
    #[serde(default)]
    errors: Vec<CfError>,
}

#[derive(Deserialize, Debug)]
struct CfMutateResponse {
    success: bool,
    #[serde(default)]
    errors: Vec<CfError>,
}

#[derive(Deserialize, Debug)]
struct CfError {
    code: i64,
    message: String,
}

#[derive(Deserialize, Debug, Clone)]
struct CfDnsRecord {
    id: String,
    name: String,
    content: String,
}

#[derive(Serialize, Debug)]
struct CfUpsertBody<'a> {
    #[serde(rename = "type")]
    record_type: &'a str,
    name: &'a str,
    content: &'a str,
    ttl: u32,
    proxied: bool,
}

fn get_public_ip() -> Result<String, String> {
    let body: IpifyResponse = ureq::get("https://api.ipify.org?format=json")
        .call()
        .map_err(|e| format!("falha ao consultar IP público: {e}"))?
        .body_mut()
        .read_json()
        .map_err(|e| format!("falha ao interpretar resposta do ipify: {e}"))?;
    Ok(body.ip)
}

fn find_dns_record(args: &Args) -> Result<Option<CfDnsRecord>, String> {
    let url = format!(
        "https://api.cloudflare.com/client/v4/zones/{}/dns_records?type={}&name={}",
        args.zone_id, args.record_type, args.record_name
    );

    let resp: CfListResponse = ureq::get(&url)
        .header("Authorization", &format!("Bearer {}", args.api_token)) // Mudou de .set() para .header()
        .call()
        .map_err(|e| format!("falha na requisição à Cloudflare: {e}"))?
        .body_mut()
        .read_json()
        .map_err(|e| format!("falha ao interpretar resposta da Cloudflare: {e}"))?;

    if !resp.success {
        return Err(format_cf_errors(&resp.errors));
    }

    Ok(resp.result.into_iter().next())
}

fn create_dns_record(args: &Args, ip: &str) -> Result<(), String> {
    let url = format!(
        "https://api.cloudflare.com/client/v4/zones/{}/dns_records",
        args.zone_id
    );

    let body = CfUpsertBody {
        record_type: &args.record_type,
        name: &args.record_name,
        content: ip,
        ttl: args.ttl,
        proxied: args.proxied,
    };

    let resp: CfMutateResponse = ureq::post(&url)
        .header("Authorization", &format!("Bearer {}", args.api_token)) // Mudou de .set() para .header()
        .send_json(&body)
        .map_err(|e| format!("falha na requisição de criação: {e}"))?
        .body_mut()
        .read_json()
        .map_err(|e| format!("falha ao interpretar resposta de criação: {e}"))?;

    if !resp.success {
        return Err(format_cf_errors(&resp.errors));
    }
    Ok(())
}

fn update_dns_record(args: &Args, record_id: &str, ip: &str) -> Result<(), String> {
    let url = format!(
        "https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
        args.zone_id, record_id
    );

    let body = CfUpsertBody {
        record_type: &args.record_type,
        name: &args.record_name,
        content: ip,
        ttl: args.ttl,
        proxied: args.proxied,
    };

    let resp: CfMutateResponse = ureq::patch(&url)
        .header("Authorization", &format!("Bearer {}", args.api_token)) // Mudou de .set() para .header()
        .send_json(&body)
        .map_err(|e| format!("falha na requisição de atualização: {e}"))?
        .body_mut()
        .read_json()
        .map_err(|e| format!("falha ao interpretar resposta de atualização: {e}"))?;

    if !resp.success {
        return Err(format_cf_errors(&resp.errors));
    }
    Ok(())
}

fn format_cf_errors(errors: &[CfError]) -> String {
    if errors.is_empty() {
        return "a API da Cloudflare retornou success=false sem detalhes".to_string();
    }
    errors
        .iter()
        .map(|e| format!("[{}] {}", e.code, e.message))
        .collect::<Vec<_>>()
        .join("; ")
}

fn run() -> Result<(), String> {
    let args = Args::parse()?;

    let current_ip = get_public_ip()?;
    println!("IP público atual: {current_ip}");

    match find_dns_record(&args)? {
        Some(record) => {
            if record.content == current_ip {
                println!(
                    "Registro '{}' já aponta para {}. Nada a fazer.",
                    record.name, current_ip
                );
            } else {
                println!(
                    "Registro '{}' aponta para {} (desatualizado). Atualizando para {}...",
                    record.name, record.content, current_ip
                );
                update_dns_record(&args, &record.id, &current_ip)?;
                println!("Registro atualizado com sucesso.");
            }
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

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Erro: {e}");
            ExitCode::FAILURE
        }
    }
}
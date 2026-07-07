use serde::{Deserialize, Serialize};
use crate::args::Args;

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
pub struct CfDnsRecord {
    pub id: String,
    pub name: String,
    pub content: String,
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

pub fn find_dns_record(args: &Args) -> Result<Option<CfDnsRecord>, String> {
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

pub fn create_dns_record(args: &Args, ip: &str) -> Result<(), String> {
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

pub fn update_dns_record(args: &Args, record_id: &str, ip: &str) -> Result<(), String> {
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
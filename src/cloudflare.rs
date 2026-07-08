use serde::{Deserialize, Serialize};
use crate::args::Args;

const URL: &str = "https://api.cloudflare.com/client/v4/zones";
const AUTHORIZATION_HEADER: &str = "Authorization";

#[derive(Deserialize, Debug)]
struct CfFindResponse {
    success: bool,
    result: Vec<CfDnsRecord>,
    #[serde(default)]
    errors: Vec<CfError>,
}

#[derive(Deserialize, Debug)]
struct CfUpdateResponse {
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
    let base_url = get_url(&args.zone_id);
    let record_type = &args.record_type;
    let name = &args.record_name;
    let url = format!("{base_url}?type={record_type}&name={name}");
    let resp: CfFindResponse = ureq::get(&url)
        .header(AUTHORIZATION_HEADER, &get_authorization(&args.api_token))
        .call()
        .map_err(|e| e.to_string())?
        .body_mut()
        .read_json()
        .map_err(|e| e.to_string())?;

    if !resp.success {
        return Err(to_string(&resp.errors));
    }

    Ok(resp.result.into_iter().next())
}

pub fn update_dns_record(args: &Args, record_id: &str, ip: &str) -> Result<(), String> {
    let base_url = get_url(&args.zone_id);
    let url = format!("{base_url}/{record_id}");

    let body = CfUpsertBody {
        record_type: &args.record_type,
        name: &args.record_name,
        content: ip,
        ttl: args.ttl,
        proxied: args.proxied,
    };

    let resp: CfUpdateResponse = ureq::patch(&url)
        .header(AUTHORIZATION_HEADER, &get_authorization(&args.api_token))
        .send_json(&body)
        .map_err(|e| e.to_string())?
        .body_mut()
        .read_json()
        .map_err(|e| e.to_string())?;

    if !resp.success {
        return Err(to_string(&resp.errors));
    }
    
    Ok(())
}

fn get_url(zone_id: &str) -> String {
    format!("{URL}/{zone_id}/dns_records")
}

fn get_authorization(api_token: &str) -> String {
    format!("Bearer {api_token}")
}

fn to_string(errors: &[CfError]) -> String {
    if errors.is_empty() {
        return "Unknown".to_string();
    }
    
    errors
        .iter()
        .map(|e| format!("[{}] {}", e.code, e.message))
        .collect::<Vec<_>>()
        .join("; ")
}
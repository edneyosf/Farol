use std::env;

const VERSION: &str = "1.0.0";

const CF_API_TOKEN: &str = "CF_API_TOKEN";
const CF_ZONE_ID: &str = "CF_ZONE_ID";
const CF_RECORD_NAME: &str = "CF_RECORD_NAME";
const CF_RECORD_TYPE: &str = "CF_RECORD_TYPE";
const CF_PROXIED: &str = "CF_PROXIED";
const CF_TTL: &str = "CF_TTL";

const API_TOKEN: &str = "--api-token";
const ZONE_ID: &str = "--zone-id";
const RECORD_NAME :&str = "--record-name";
const RECORD_TYPE: &str = "--record-type";
const TTL: &str = "--ttl";
const PROXIED: &str = "--proxied";

pub struct Args {
    pub api_token: String,
    pub zone_id: String,
    pub record_name: String,
    pub record_type: String,
    pub proxied: bool,
    pub ttl: u32
}

impl Args {
    pub fn parse() -> Result<Self, String> {
        let mut api_token = env::var(CF_API_TOKEN).ok();
        let mut zone_id = env::var(CF_ZONE_ID).ok();
        let mut record_name = env::var(CF_RECORD_NAME).ok();
        let mut record_type = env::var(CF_RECORD_TYPE).unwrap_or_else(|_| "A".to_string());
        let mut proxied = env::var(CF_PROXIED)
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false);
        let mut ttl: u32 = env::var(CF_TTL)
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(1);

        let mut args = env::args().skip(1);
        
        while let Some(flag) = args.next() {
            let mut next_val = || {
                args.next()
                    .ok_or_else(|| format!("Value missing after {flag}"))
            };
            
            match flag.as_str() {
                API_TOKEN => api_token = Some(next_val()?),
                ZONE_ID => zone_id = Some(next_val()?),
                RECORD_NAME => record_name = Some(next_val()?),
                RECORD_TYPE => record_type = next_val()?,
                TTL => ttl = next_val()?.parse().map_err(|_| format!("{TTL} invalid"))?,
                PROXIED => proxied = true,
                "-h" | "--help" => {
                    help();
                    std::process::exit(0);
                }
                "-v" | "--version" => {
                    version();
                    std::process::exit(0);
                }
                other => return Err(format!("Unknown flag: {other}")),
            }
        }

        Ok(Args {
            api_token: api_token.ok_or(format!("Define {CF_API_TOKEN} or use {API_TOKEN}"))?,
            zone_id: zone_id.ok_or(format!("Define {CF_ZONE_ID} or use {ZONE_ID}"))?,
            record_name: record_name.ok_or(format!("Define {CF_RECORD_NAME} or use {RECORD_NAME}"))?,
            record_type,
            proxied,
            ttl
        })
    }
}

fn help() {
    println!(
        r#"
Use:
    {API_TOKEN}
    {ZONE_ID}
    {RECORD_NAME}
    {RECORD_TYPE}
    {PROXIED}
    {TTL}

Environment variable:
    {CF_API_TOKEN}
    {CF_ZONE_ID}
    {CF_RECORD_NAME}
    {CF_RECORD_TYPE}
    {CF_PROXIED}
    {CF_TTL}
"#
    );
}

fn version() {
    println!("Farol v{VERSION}");
}
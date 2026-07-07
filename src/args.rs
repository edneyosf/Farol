use std::env;

pub struct Args {
    pub api_token: String,
    pub zone_id: String,
    pub record_name: String,
    pub record_type: String,
    pub proxied: bool,
    pub ttl: u32,
}

impl Args {
    pub fn parse() -> Result<Self, String> {
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
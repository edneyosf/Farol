use serde::Deserialize;

const URL: &str = "https://api.ipify.org?format=json";

#[derive(Deserialize, Debug)]
struct IpifyResponse {
    ip: String,
}

pub fn get_public_ip() -> Result<String, String> {
    let body: IpifyResponse = ureq::get(URL)
        .call()
        .map_err(|e| format!("falha ao consultar IP público: {e}"))?
        .body_mut()
        .read_json()
        .map_err(|e| format!("falha ao interpretar resposta do ipify: {e}"))?;
    Ok(body.ip)
}
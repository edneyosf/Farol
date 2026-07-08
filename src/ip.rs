use std::thread;
use std::time::Duration;
use anyhow::Result;

const IPIFY_URL: &str = "https://api.ipify.org";
const ICANHAZIP_URL: &str = "https://4.icanhazip.com";
const MAX_ATTEMPTS: u8 = 3;
const INTERVAL: u64 = 2;

pub fn get_public_ip() -> Option<String> {
    fetch_with_retry(IPIFY_URL)
        .or_else(|| fetch_with_retry(ICANHAZIP_URL))
}

fn fetch_with_retry(url: &str) -> Option<String> {
    for _ in 1..=MAX_ATTEMPTS {
        match fetch(url) {
            Ok(value) => return Some(value),
            Err(_) => thread::sleep(Duration::from_secs(INTERVAL))
        }
    }
    None
}

fn fetch(url: &str) -> Result<String> {
    let data = ureq::get(url)
        .call()?
        .body_mut()
        .read_to_string();
    
    Ok(data?.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::prelude::*;

    #[test]
    fn returns_ip_on_first_success() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET).path("/");
            then.status(200).body("1.2.3.4");
        });

        let result = fetch_with_retry(&server.base_url());

        assert_eq!(result, Some("1.2.3.4".to_string()));
        mock.assert_calls(1);
    }

    #[test]
    fn retries_and_recovers_after_failure() {
        let server = MockServer::start();
        let fail_mock = server.mock(|when, then| {
            when.method(GET).path("/");
            then.status(500);
        });

        let result = fetch_with_retry(&server.base_url());

        assert_eq!(result, None);
        fail_mock.assert_calls(3);
    }

    #[test]
    fn returns_trimmed_ip_on_success() {
        let server = MockServer::start();
        
        server.mock(|when, then| {
            when.method(GET).path("/");
            then.status(200).body("1.2.3.4\n");
        });

        let result = fetch(&server.base_url());

        assert_eq!(result.unwrap(), "1.2.3.4");
    }

    #[test]
    fn returns_err_on_http_error_status() {
        let server = MockServer::start();
        
        server.mock(|when, then| {
            when.method(GET).path("/");
            then.status(500).body("internal server error");
        });
    
        let result = fetch(&server.base_url());
    
        assert!(result.is_err());
    }
}
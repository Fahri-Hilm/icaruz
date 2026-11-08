pub fn parse_server(input: &str, fallback_port: u16) -> Result<(String, u16), String> {
    let s = input.trim();
    if s.is_empty() {
        return Err("Server address is empty.".to_string());
    }
    // Accept "ip:port" or bare ip/host with fallback port.
    if let Some((host, port_str)) = s.rsplit_once(':') {
        if !host.is_empty() && !port_str.is_empty() && port_str.chars().all(|c| c.is_ascii_digit()) {
            let port: u16 = port_str
                .parse()
                .map_err(|_| "Port must be 1 to 65535.".to_string())?;
            if port == 0 {
                return Err("Port must be 1 to 65535.".to_string());
            }
            validate_host(host)?;
            return Ok((host.to_string(), port));
        }
    }
    validate_host(s)?;
    if fallback_port == 0 {
        return Err("Port must be 1 to 65535.".to_string());
    }
    Ok((s.to_string(), fallback_port))
}

fn validate_host(host: &str) -> Result<(), String> {
    if host.len() > 253 {
        return Err("Server address too long.".to_string());
    }
    let ok = host
        .chars()
        .all(|c| c.is_alphanumeric() || c == '.' || c == '-' || c == '_');
    if !ok {
        return Err("Server address holds invalid characters.".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn split_ip_port() {
        assert_eq!(
            parse_server("127.0.0.1:7777", 7777).unwrap(),
            ("127.0.0.1".to_string(), 7777)
        );
        assert_eq!(
            parse_server("play.example.com", 7777).unwrap(),
            ("play.example.com".to_string(), 7777)
        );
        assert!(parse_server("", 7777).is_err());
        assert!(parse_server("127.0.0.1:0", 7777).is_err());
    }
}

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

pub const CACHE_TTL: Duration = Duration::from_secs(60);
pub const UDP_DEFAULT_TIMEOUT_MS: u64 = 1500;
pub const UDP_DEFAULT_CONCURRENCY: usize = 8;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServerEntry {
    pub address: String,
    pub port: u16,
    pub hostname: Option<String>,
    pub players: Option<u16>,
    pub max_players: Option<u16>,
}

impl ServerEntry {
    pub fn key(&self) -> String {
        format!("{}:{}", self.address.to_lowercase(), self.port)
    }
}

// Parse "ip:port" lines, skip empty and # comments.
pub fn parse_masterlist_text(input: &str, fallback_port: u16) -> Vec<ServerEntry> {
    let mut out = Vec::new();
    for line in input.lines() {
        let t = line.trim();
        if t.is_empty() || t.starts_with('#') || t.starts_with("//") {
            continue;
        }
        if let Ok((host, port)) = crate::launch::parse_server(t, fallback_port) {
            out.push(ServerEntry {
                address: host,
                port,
                hostname: None,
                players: None,
                max_players: None,
            });
        }
    }
    dedup_servers(out)
}

// Accept JSON array of strings ("ip:port") or objects with address/host/ip plus port.
pub fn parse_masterlist_json(input: &str, fallback_port: u16) -> Result<Vec<ServerEntry>, String> {
    let v: serde_json::Value =
        serde_json::from_str(input).map_err(|e| format!("Masterlist JSON invalid: {e}"))?;
    let arr = v
        .as_array()
        .ok_or("Masterlist JSON must be an array.".to_string())?;
    let mut out = Vec::new();
    for item in arr {
        match item {
            serde_json::Value::String(s) => {
                if let Ok((host, port)) = crate::launch::parse_server(s, fallback_port) {
                    out.push(ServerEntry {
                        address: host,
                        port,
                        hostname: None,
                        players: None,
                        max_players: None,
                    });
                }
            }
            serde_json::Value::Object(_) => {
                let host = item
                    .get("address")
                    .or_else(|| item.get("host"))
                    .or_else(|| item.get("ip"))
                    .and_then(|x| x.as_str());
                let port = item
                    .get("port")
                    .and_then(|x| x.as_u64())
                    .map(|x| x as u16)
                    .unwrap_or(fallback_port);
                if let Some(h) = host {
                    if let Ok((host2, port2)) = crate::launch::parse_server(h, port) {
                        out.push(ServerEntry {
                            address: host2,
                            port: port2,
                            hostname: item
                                .get("hostname")
                                .and_then(|x| x.as_str())
                                .map(|x| x.to_string()),
                            players: item
                                .get("players")
                                .and_then(|x| x.as_u64())
                                .map(|x| x as u16),
                            max_players: item
                                .get("max_players")
                                .or_else(|| item.get("maxplayers"))
                                .and_then(|x| x.as_u64())
                                .map(|x| x as u16),
                        });
                    }
                }
            }
            _ => continue,
        }
    }
    Ok(dedup_servers(out))
}

pub fn dedup_servers(servers: Vec<ServerEntry>) -> Vec<ServerEntry> {
    let mut seen = HashMap::new();
    let mut out = Vec::new();
    for s in servers {
        let k = s.key();
        if seen.insert(k, true).is_none() {
            out.push(s);
        }
    }
    out
}

pub trait ServerProvider: Send + Sync {
    fn name(&self) -> &str;
    fn fetch(&self) -> impl std::future::Future<Output = Result<Vec<ServerEntry>, String>> + Send;
}

pub struct MasterlistProvider {
    pub url: String,
    pub fallback_port: u16,
    pub timeout_ms: u64,
}

impl MasterlistProvider {
    pub fn new(url: impl Into<String>, fallback_port: u16) -> Self {
        Self {
            url: url.into(),
            fallback_port,
            timeout_ms: 8000,
        }
    }
}

impl ServerProvider for MasterlistProvider {
    fn name(&self) -> &str {
        "masterlist"
    }
    async fn fetch(&self) -> Result<Vec<ServerEntry>, String> {
        fetch_masterlist(&self.url, self.fallback_port, self.timeout_ms).await
    }
}

pub async fn fetch_masterlist(
    url: &str,
    fallback_port: u16,
    timeout_ms: u64,
) -> Result<Vec<ServerEntry>, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(timeout_ms))
        .build()
        .map_err(|e| format!("HTTP client failed: {e}"))?;
    let res = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Masterlist fetch failed: {e}"))?;
    if !res.status().is_success() {
        return Err(format!("Masterlist HTTP {}.", res.status()));
    }
    let text = res
        .text()
        .await
        .map_err(|e| format!("Masterlist read failed: {e}"))?;
    let t = text.trim_start();
    if t.starts_with('[') || t.starts_with('{') {
        parse_masterlist_json(&text, fallback_port)
    } else {
        Ok(parse_masterlist_text(&text, fallback_port))
    }
}

// 60s cache keyed by provider name plus url.
pub struct DiscoveryCache {
    ttl: Duration,
    inner: Mutex<HashMap<String, (Instant, Vec<ServerEntry>)>>,
}

impl Default for DiscoveryCache {
    fn default() -> Self {
        Self::new()
    }
}

impl DiscoveryCache {
    pub fn new() -> Self {
        Self {
            ttl: CACHE_TTL,
            inner: Mutex::new(HashMap::new()),
        }
    }
    pub fn get(&self, key: &str) -> Option<Vec<ServerEntry>> {
        let guard = self.inner.lock().ok()?;
        let (at, v) = guard.get(key)?;
        if at.elapsed() < self.ttl {
            Some(v.clone())
        } else {
            None
        }
    }
    pub fn put(&self, key: &str, servers: Vec<ServerEntry>) {
        if let Ok(mut guard) = self.inner.lock() {
            guard.insert(key.to_string(), (Instant::now(), servers));
        }
    }
    pub fn invalidate(&self, key: &str) {
        if let Ok(mut guard) = self.inner.lock() {
            guard.remove(key);
        }
    }
}

// Minimal SA-MP style info query stub over UDP with timeout.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServerQuery {
    pub address: String,
    pub port: u16,
    pub raw_len: usize,
}

pub async fn query_server_udp(
    address: &str,
    port: u16,
    timeout_ms: u64,
) -> Result<ServerQuery, String> {
    use tokio::net::UdpSocket;
    use tokio::time::timeout;
    let ip: std::net::Ipv4Addr = address
        .parse()
        .map_err(|_| format!("UDP query needs IPv4 address, got {address}."))?;
    let sock = UdpSocket::bind("0.0.0.0:0")
        .await
        .map_err(|e| format!("UDP bind failed: {e}"))?;
    let mut pkt = Vec::with_capacity(11);
    pkt.extend_from_slice(b"SAMP");
    pkt.extend_from_slice(&ip.octets());
    pkt.extend_from_slice(&port.to_le_bytes());
    pkt.push(b'i');
    sock.send_to(&pkt, format!("{address}:{port}"))
        .await
        .map_err(|e| format!("UDP send failed: {e}"))?;
    let mut buf = [0u8; 2048];
    let (n, _) = timeout(Duration::from_millis(timeout_ms), sock.recv_from(&mut buf))
        .await
        .map_err(|_| format!("UDP query timed out after {timeout_ms}ms."))?
        .map_err(|e| format!("UDP recv failed: {e}"))?;
    if n < 11 {
        return Err("UDP reply too short, server may be offline.".to_string());
    }
    Ok(ServerQuery {
        address: address.to_string(),
        port,
        raw_len: n,
    })
}

pub async fn query_many(
    servers: Vec<ServerEntry>,
    concurrency: usize,
    timeout_ms: u64,
) -> Vec<(ServerEntry, Result<ServerQuery, String>)> {
    use std::sync::Arc;
    use tokio::sync::Semaphore;
    let limit = if concurrency == 0 {
        UDP_DEFAULT_CONCURRENCY
    } else {
        concurrency
    };
    let sem = Arc::new(Semaphore::new(limit));
    let mut set = tokio::task::JoinSet::new();
    for s in servers {
        let permit_sem = sem.clone();
        set.spawn(async move {
            let _permit = permit_sem.acquire_owned().await;
            let r = query_server_udp(&s.address, s.port, timeout_ms).await;
            (s, r)
        });
    }
    let mut out = Vec::new();
    while let Some(r) = set.join_next().await {
        if let Ok(pair) = r {
            out.push(pair);
        }
    }
    out
}

// Try providers in order, return first success with its name.
pub async fn fetch_with_fallback(
    providers: &[&dyn QuerySource],
    cache: Option<&DiscoveryCache>,
) -> Result<(String, Vec<ServerEntry>), String> {
    let mut errors = Vec::new();
    for p in providers {
        let key = format!("fallback:{}", p.name());
        if let Some(c) = cache {
            if let Some(hit) = c.get(&key) {
                return Ok((p.name().to_string(), hit));
            }
        }
        match p.fetch_box().await {
            Ok(v) => {
                let v = dedup_servers(v);
                if let Some(c) = cache {
                    c.put(&key, v.clone());
                }
                return Ok((p.name().to_string(), v));
            }
            Err(e) => errors.push(format!("{}: {e}", p.name())),
        }
    }
    Err(format!("All providers failed. {}", errors.join("; ")))
}

// Object safe wrapper so fallback can hold mixed providers.
pub trait QuerySource: Send + Sync {
    fn name(&self) -> &str;
    fn fetch_box(
        &self,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Vec<ServerEntry>, String>> + Send + '_>,
    >;
}

impl QuerySource for MasterlistProvider {
    fn name(&self) -> &str {
        ServerProvider::name(self)
    }
    fn fetch_box(
        &self,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Vec<ServerEntry>, String>> + Send + '_>,
    > {
        Box::pin(async move { ServerProvider::fetch(self).await })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_text_skips_comments_and_dedups() {
        let v = parse_masterlist_text(
            "# list\n127.0.0.1:7777\n127.0.0.1:7777\nplay.example.com\n\n// x\n",
            7777,
        );
        assert_eq!(v.len(), 2);
        assert_eq!(v[0].key(), "127.0.0.1:7777");
    }

    #[test]
    fn parse_json_strings_and_objects() {
        let v = parse_masterlist_json(
            r#"["10.0.0.1:7777", {"address": "10.0.0.2", "port": 8888, "hostname": "RP"}]"#,
            7777,
        )
        .unwrap();
        assert_eq!(v.len(), 2);
        assert_eq!(v[1].hostname.as_deref(), Some("RP"));
        assert!(parse_masterlist_json("not json", 7777).is_err());
    }

    #[test]
    fn dedup_case_insensitive_host() {
        let v = dedup_servers(vec![
            ServerEntry {
                address: "Play.Example.com".into(),
                port: 7777,
                hostname: None,
                players: None,
                max_players: None,
            },
            ServerEntry {
                address: "play.example.com".into(),
                port: 7777,
                hostname: None,
                players: None,
                max_players: None,
            },
            ServerEntry {
                address: "play.example.com".into(),
                port: 7778,
                hostname: None,
                players: None,
                max_players: None,
            },
        ]);
        assert_eq!(v.len(), 2);
    }

    #[test]
    fn cache_hit_and_invalidate() {
        let c = DiscoveryCache::new();
        let v = vec![ServerEntry {
            address: "127.0.0.1".into(),
            port: 7777,
            hostname: None,
            players: None,
            max_players: None,
        }];
        assert!(c.get("k").is_none());
        c.put("k", v.clone());
        assert_eq!(c.get("k").unwrap(), v);
        c.invalidate("k");
        assert!(c.get("k").is_none());
    }

    struct Stub {
        n: &'static str,
        r: Result<Vec<ServerEntry>, String>,
    }
    impl QuerySource for Stub {
        fn name(&self) -> &str {
            self.n
        }
        fn fetch_box(
            &self,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<Vec<ServerEntry>, String>> + Send + '_>,
        > {
            let r = self.r.clone();
            Box::pin(async move { r })
        }
    }

    #[tokio::test]
    async fn fallback_picks_second_and_dedups() {
        let cache = DiscoveryCache::new();
        let a = Stub {
            n: "a",
            r: Err("down".into()),
        };
        let dup = ServerEntry {
            address: "127.0.0.1".into(),
            port: 7777,
            hostname: None,
            players: None,
            max_players: None,
        };
        let b = Stub {
            n: "b",
            r: Ok(vec![dup.clone(), dup]),
        };
        let providers: Vec<&dyn QuerySource> = vec![&a, &b];
        let (name, v) = fetch_with_fallback(&providers, Some(&cache)).await.unwrap();
        assert_eq!(name, "b");
        assert_eq!(v.len(), 1);
    }

    #[tokio::test]
    async fn fallback_all_fail() {
        let a = Stub {
            n: "a",
            r: Err("down".into()),
        };
        let providers: Vec<&dyn QuerySource> = vec![&a];
        assert!(fetch_with_fallback(&providers, None).await.is_err());
    }

    #[tokio::test]
    async fn udp_rejects_hostname_honestly() {
        let r = query_server_udp("not-an-ip", 7777, 200).await;
        assert!(r.is_err());
    }
}

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
    pub gamemode: Option<String>,
    pub language: Option<String>,
    pub passworded: bool,
    pub omp: bool,
}

impl ServerEntry {
    pub fn key(&self) -> String {
        format!("{}:{}", self.address.to_lowercase(), self.port)
    }
    pub fn bare(address: impl Into<String>, port: u16) -> Self {
        Self {
            address: address.into(),
            port,
            hostname: None,
            players: None,
            max_players: None,
            gamemode: None,
            language: None,
            passworded: false,
            omp: false,
        }
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
            out.push(ServerEntry::bare(host, port));
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
                    out.push(ServerEntry::bare(host, port));
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
                        let mut e = ServerEntry::bare(host2, port2);
                        e.hostname = item
                            .get("hostname")
                            .and_then(|x| x.as_str())
                            .map(|x| x.to_string());
                        e.players = item
                            .get("players")
                            .and_then(|x| x.as_u64())
                            .map(|x| x as u16);
                        e.max_players = item
                            .get("max_players")
                            .or_else(|| item.get("maxplayers"))
                            .and_then(|x| x.as_u64())
                            .map(|x| x as u16);
                        e.gamemode = item
                            .get("gamemode")
                            .and_then(|x| x.as_str())
                            .map(|x| x.to_string());
                        e.language = item
                            .get("language")
                            .and_then(|x| x.as_str())
                            .map(|x| x.to_string());
                        e.passworded = item
                            .get("passworded")
                            .and_then(|x| x.as_bool())
                            .unwrap_or(false);
                        out.push(e);
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

// open.mp public server API shape (verified live):
// [{ip:"host:port", hn, pc, pm, gm, la, pa, vn, omp, pr}]
pub fn parse_openmp_api(input: &str) -> Result<Vec<ServerEntry>, String> {
    let v: serde_json::Value =
        serde_json::from_str(input).map_err(|e| format!("open.mp API invalid: {e}"))?;
    let arr = v
        .as_array()
        .ok_or("open.mp API must be an array.".to_string())?;
    let mut out = Vec::new();
    for item in arr {
        let ip = item.get("ip").and_then(|x| x.as_str()).unwrap_or("");
        let (host, port) = match crate::launch::parse_server(ip, 7777) {
            Ok(x) => x,
            Err(_) => continue,
        };
        let mut e = ServerEntry::bare(host, port);
        e.hostname = item
            .get("hn")
            .and_then(|x| x.as_str())
            .map(|x| x.to_string());
        e.players = item.get("pc").and_then(|x| x.as_u64()).map(|x| x as u16);
        e.max_players = item.get("pm").and_then(|x| x.as_u64()).map(|x| x as u16);
        e.gamemode = item
            .get("gm")
            .and_then(|x| x.as_str())
            .map(|x| x.to_string());
        e.language = item
            .get("la")
            .and_then(|x| x.as_str())
            .map(|x| x.to_string());
        e.passworded = item.get("pa").and_then(|x| x.as_bool()).unwrap_or(false);
        e.omp = item.get("omp").and_then(|x| x.as_bool()).unwrap_or(true);
        out.push(e);
    }
    Ok(dedup_servers(out))
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServerInfo {
    pub address: String,
    pub port: u16,
    pub passworded: bool,
    pub players: u16,
    pub max_players: u16,
    pub hostname: String,
    pub gamemode: String,
    pub language: String,
    pub ping_ms: u64,
}

fn read_str(buf: &[u8], pos: &mut usize) -> Result<String, String> {
    if *pos + 4 > buf.len() {
        return Err("Query reply cut in string length.".to_string());
    }
    let len = u32::from_le_bytes([buf[*pos], buf[*pos + 1], buf[*pos + 2], buf[*pos + 3]]) as usize;
    *pos += 4;
    if *pos + len > buf.len() {
        return Err("Query reply cut in string body.".to_string());
    }
    let s = String::from_utf8_lossy(&buf[*pos..*pos + len]).to_string();
    *pos += len;
    Ok(s)
}

// Full SA-MP 'i' query with parse and round-trip latency.
pub async fn query_server_info(
    address: &str,
    port: u16,
    timeout_ms: u64,
) -> Result<ServerInfo, String> {
    use std::time::Instant;
    use tokio::net::UdpSocket;
    use tokio::time::timeout;
    let ip: std::net::Ipv4Addr = address
        .parse()
        .map_err(|_| format!("Server query needs IPv4 address, got {address}."))?;
    let sock = UdpSocket::bind("0.0.0.0:0")
        .await
        .map_err(|e| format!("UDP bind failed: {e}"))?;
    let mut pkt = Vec::with_capacity(11);
    pkt.extend_from_slice(b"SAMP");
    pkt.extend_from_slice(&ip.octets());
    pkt.extend_from_slice(&port.to_le_bytes());
    pkt.push(b'i');
    let start = Instant::now();
    sock.send_to(&pkt, format!("{address}:{port}"))
        .await
        .map_err(|e| format!("UDP send failed: {e}"))?;
    let mut buf = [0u8; 2048];
    let (n, _) = timeout(Duration::from_millis(timeout_ms), sock.recv_from(&mut buf))
        .await
        .map_err(|_| format!("No reply in {timeout_ms}ms. Server may be offline."))?
        .map_err(|e| format!("UDP recv failed: {e}"))?;
    let ping_ms = start.elapsed().as_millis() as u64;
    if n < 12 || &buf[0..4] != b"SAMP" {
        return Err("Reply is not a SA-MP query answer.".to_string());
    }
    let mut pos = 11;
    let passworded = buf[pos] != 0;
    pos += 1;
    if pos + 4 > n {
        return Err("Reply too short for player counts.".to_string());
    }
    let players = u16::from_le_bytes([buf[pos], buf[pos + 1]]);
    let max_players = u16::from_le_bytes([buf[pos + 2], buf[pos + 3]]);
    pos += 4;
    let hostname = read_str(&buf[..n], &mut pos)?;
    let gamemode = read_str(&buf[..n], &mut pos).unwrap_or_default();
    let language = read_str(&buf[..n], &mut pos).unwrap_or_default();
    Ok(ServerInfo {
        address: address.to_string(),
        port,
        passworded,
        players,
        max_players,
        hostname,
        gamemode,
        language,
        ping_ms,
    })
}

// Fetch live open.mp server list with timeout, then parse.
pub async fn fetch_openmp(url: &str, timeout_ms: u64) -> Result<Vec<ServerEntry>, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_millis(timeout_ms))
        .build()
        .map_err(|e| format!("HTTP client failed: {e}"))?;
    let res = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Server list fetch failed: {e}"))?;
    if !res.status().is_success() {
        return Err(format!("Server list HTTP {}.", res.status()));
    }
    let text = res
        .text()
        .await
        .map_err(|e| format!("Server list read failed: {e}"))?;
    parse_openmp_api(&text)
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
            ServerEntry::bare("Play.Example.com", 7777),
            ServerEntry::bare("play.example.com", 7777),
            ServerEntry::bare("play.example.com", 7778),
        ]);
        assert_eq!(v.len(), 2);
    }

    #[test]
    fn cache_hit_and_invalidate() {
        let c = DiscoveryCache::new();
        let v = vec![ServerEntry::bare("127.0.0.1", 7777)];
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
        let dup = ServerEntry::bare("127.0.0.1", 7777);
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

    #[test]
    fn openmp_api_parses_live_shape() {
        let v = parse_openmp_api(
            r#"[{"ip":"10.0.0.9:7777","hn":"RP Test","pc":12,"pm":100,"gm":"roleplay","la":"English","pa":false,"vn":"omp 1.5","omp":true,"pr":false},"bad-item"]"#,
        )
        .unwrap();
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].hostname.as_deref(), Some("RP Test"));
        assert_eq!(v[0].players, Some(12));
        assert_eq!(v[0].gamemode.as_deref(), Some("roleplay"));
        assert!(parse_openmp_api("nope").is_err());
    }

    #[test]
    fn info_parse_reads_fixture() {
        // Header(11) + pw(1) + players(2) + max(2) + strings.
        let mut raw = Vec::new();
        raw.extend_from_slice(b"SAMP");
        raw.extend_from_slice(&[127, 0, 0, 1]);
        raw.extend_from_slice(&7777u16.to_le_bytes());
        raw.push(b'i');
        raw.push(0);
        raw.extend_from_slice(&5u16.to_le_bytes());
        raw.extend_from_slice(&100u16.to_le_bytes());
        for s in ["My Server", "roleplay", "English"] {
            raw.extend_from_slice(&(s.len() as u32).to_le_bytes());
            raw.extend_from_slice(s.as_bytes());
        }
        let mut pos = 11;
        assert_eq!(raw[pos], 0);
        pos += 1;
        assert_eq!(u16::from_le_bytes([raw[pos], raw[pos + 1]]), 5);
        pos += 4;
        let n = raw.len();
        assert_eq!(read_str(&raw[..n], &mut pos).unwrap(), "My Server");
        assert_eq!(read_str(&raw[..n], &mut pos).unwrap(), "roleplay");
        assert_eq!(read_str(&raw[..n], &mut pos).unwrap(), "English");
        assert!(read_str(&raw[..n], &mut pos).is_err());
    }
}

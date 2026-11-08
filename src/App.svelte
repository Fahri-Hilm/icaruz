<script lang="ts">
  import { onMount } from "svelte";
  import { Home, Server, Layers3, Settings, Search, Bell, ChevronRight, Play, Star, Users, Wifi, ShieldCheck, Check, Download, FolderOpen, Wrench, RotateCcw, ArrowUpRight, SlidersHorizontal, Gamepad2, X, Menu, Info, Zap, HardDrive, Monitor, RefreshCcw, CircleDot, Copy, AlertCircle } from "lucide-svelte";

  type Page = "Home" | "Servers" | "Library" | "Settings";
  type ClientKind = "Samp037R5" | "Samp03DL" | "OpenMp1x";
  type GameInstall = { gta_path: string; exe_version: string; exe_sha256: string; valid: boolean; detail: string };
  type ClientStatus = { kind: ClientKind; installed: boolean; version: string; path: string | null };
  type LauncherStatus = { gta: GameInstall | null; clients: ClientStatus[]; nickname: string; needs_attention: string | null };
  type ServerEntry = { address: string; port: number; hostname: string | null; players: number | null; max_players: number | null; gamemode: string | null; language: string | null; passworded: boolean; omp: boolean };
  type ServerInfo = { hostname: string; gamemode: string; language: string; players: number; max_players: number; passworded: boolean; ping_ms: number };
  type Fav = { server: string; port: number; label: string };
  type Hist = { server: string; port: number; started_at: string; duration_secs: number; error: string };
  type RepairItem = { id: string; label: string; title?: string; status: string; detail: string; fix_hint: string; needs_approval: boolean; backup_before_fix?: boolean; can_rollback?: boolean };
  type Row = { key: string; address: string; port: number; omp: boolean; passworded: boolean };

  const CLIENT_LABEL: Record<ClientKind, string> = { Samp037R5: "SA-MP 0.3.7-R5", Samp03DL: "SA-MP 0.3DL", OpenMp1x: "open.mp 1.x" };
  const APP_VERSION = "1.0.7";
  const CLIENT_OPTS: ClientKind[] = ["Samp037R5", "Samp03DL", "OpenMp1x"];

  async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
    const mod = await import("@tauri-apps/api/core");
    return mod.invoke(cmd, args) as Promise<T>;
  }

  let page: Page = "Home";
  let query = "";
  let category = "Semua";
  let sortBy = "Populer";
  let sidebarOpen = false;
  let showNotifications = false;

  let statusLoading = true;
  let statusError = "";
  let gta: GameInstall | null = null;
  let clients: ClientStatus[] = [];
  let needsAttention: string | null = null;
  let webviewMsg = "";
  let webviewOk = true;

  let nickname = "Player";
  let nickMsg = "";
  let gtaPath = "";
  let gtaMsg = "";
  let gtaState: "idle" | "loading" | "success" | "error" = "idle";

  let modProfile = "Vanilla";
  let autoClose = true;
  let startup = false;
  let startupMsg = "";
  let reduceMotion = false;

  let servers: ServerEntry[] = [];
  let serversLoading = true;
  let serversError = "";
  let infoMap: Record<string, { info?: ServerInfo; loading: boolean; error: string }> = {};
  let favs: Fav[] = [];
  let favError = "";
  let hist: Hist[] = [];
  let lastPlayed: Hist | null = null;

  let directInput = "";
  let directError = "";

  let selected: Row | null = null;
  let launchClient: ClientKind = "Samp037R5";
  let launching = false;
  let launchMsg = "";
  let launchError = "";
  let planNote = "";

  let modal: "launch" | "repair" | "about" | "network" | "downloads" | null = null;

  let repairItems: RepairItem[] = [];
  let repairLoading = false;
  let repairError = "";

  let netTarget = "";
  let netLoading = false;
  let netError = "";
  let netInfo: ServerInfo | null = null;

  let dlUrl = "";
  let dlSha = "";
  let dlName = "";
  let dlSize = "";
  let dlMsg = "";
  let dlError = "";

  let notification = "";
  let toastTimer: ReturnType<typeof setTimeout>;
  function toast(message: string) {
    notification = message;
    clearTimeout(toastTimer);
    toastTimer = setTimeout(() => (notification = ""), 3700);
  }

  function rowKey(address: string, port: number) {
    return `${address.toLowerCase()}:${port}`;
  }
  function entryRow(e: ServerEntry): Row {
    return { key: rowKey(e.address, e.port), address: e.address, port: e.port, omp: e.omp, passworded: e.passworded };
  }
  function isFav(r: Row) {
    return favs.some((f) => rowKey(f.server, f.port) === r.key);
  }
  function displayName(r: Row): string {
    const st = infoMap[r.key];
    if (st?.info?.hostname) return st.info.hostname;
    const found = servers.find((s) => rowKey(s.address, s.port) === r.key);
    if (found?.hostname) return found.hostname;
    return `${r.address}:${r.port}`;
  }
  function playerText(r: Row): string {
    const st = infoMap[r.key];
    if (st?.loading) return "Memeriksa...";
    if (st?.info) return `${st.info.players}/${st.info.max_players}`;
    if (st?.error) return "offline";
    const found = servers.find((s) => rowKey(s.address, s.port) === r.key);
    if (found?.players != null && found?.max_players != null) return `${found.players}/${found.max_players}`;
    return "belum ada data";
  }
  function pingText(r: Row): string {
    const st = infoMap[r.key];
    if (st?.loading) return "Memeriksa...";
    if (st?.info) return `${st.info.ping_ms} ms`;
    if (st?.error) return "offline";
    return "belum ada data";
  }
  function modeText(r: Row): string {
    const st = infoMap[r.key];
    if (st?.info?.gamemode) return st.info.gamemode;
    const found = servers.find((s) => rowKey(s.address, s.port) === r.key);
    return found?.gamemode || "Tidak diketahui";
  }
  function defaultClientFor(r: Row): ClientKind {
    return r.omp ? "OpenMp1x" : "Samp037R5";
  }

  $: categories = ["Semua", ...Array.from(new Set(servers.map((s) => s.gamemode || infoMap[rowKey(s.address, s.port)]?.info?.gamemode).filter((x) => !!x))), "Favorit"];
  $: rows = servers.map(entryRow);
  $: visibleRows = rows
    .filter((r) => {
      if (category === "Favorit" && !isFav(r)) return false;
      if (category !== "Semua" && category !== "Favorit" && modeText(r) !== category) return false;
      const q = query.trim().toLowerCase();
      if (!q) return true;
      return `${displayName(r)} ${modeText(r)} ${r.address}`.toLowerCase().includes(q);
    })
    .sort((a, b) => {
      if (sortBy === "Ping terendah") {
        const pa = infoMap[a.key]?.info?.ping_ms ?? Number.MAX_SAFE_INTEGER;
        const pb = infoMap[b.key]?.info?.ping_ms ?? Number.MAX_SAFE_INTEGER;
        return pa - pb;
      }
      const na = infoMap[a.key]?.info?.players ?? servers.find((s) => rowKey(s.address, s.port) === a.key)?.players ?? -1;
      const nb = infoMap[b.key]?.info?.players ?? servers.find((s) => rowKey(s.address, s.port) === b.key)?.players ?? -1;
      return nb - na;
    });

  $: heroRow = selected ?? (lastPlayed ? { key: rowKey(lastPlayed.server, lastPlayed.port), address: lastPlayed.server, port: lastPlayed.port, omp: false, passworded: false } : favs.length ? { key: rowKey(favs[0].server, favs[0].port), address: favs[0].server, port: favs[0].port, omp: false, passworded: false } : rows.length ? rows[0] : null);

  $: activeClient = clients.find((c) => c.installed) ?? null;
  $: notifications = [
    ...(needsAttention ? [needsAttention] : []),
    ...(!webviewOk && webviewMsg ? [webviewMsg] : []),
    ...(launchError ? [`Peluncuran terakhir gagal: ${launchError}`] : []),
  ];

  function nav(to: Page) {
    page = to;
    sidebarOpen = false;
    showNotifications = false;
    if (to !== "Servers") query = "";
  }

  function openLaunch(r: Row) {
    selected = r;
    launchClient = defaultClientFor(r);
    launchMsg = "";
    launchError = "";
    planNote = "";
    modal = "launch";
  }

  function parseDirect(input: string): { host: string; port: number } {
    const t = input.trim();
    const m = t.match(/^(.+):(\d{1,5})$/);
    if (!m) throw new Error("Format harus IP:port, contoh 127.0.0.1:7777.");
    const port = Number(m[2]);
    if (port < 1 || port > 65535) throw new Error("Port harus 1 sampai 65535.");
    return { host: m[1], port };
  }

  async function doLaunch() {
    if (!selected) return;
    if (!gtaPath) {
      launchError = "Folder GTA belum diatur. Atur dulu di Library.";
      return;
    }
    launching = true;
    launchError = "";
    launchMsg = "Menyiapkan rencana peluncuran...";
    const started = Date.now();
    try {
      const plan = await invoke<{ exe: string; args: string[]; note: string }>("plan_launch", {
        req: { client: launchClient, gta_path: gtaPath, server: selected.address, port: selected.port, nick: nickname },
      });
      planNote = plan.note || "";
      launchMsg = `Menjalankan ${plan.exe}...`;
      const res = await invoke("launch_game", {
        req: { client: launchClient, gta_path: gtaPath, server: selected.address, port: selected.port, nick: nickname },
      });
      const secs = Math.max(0, Math.round((Date.now() - started) / 1000));
      launchMsg = `Berjalan: ${JSON.stringify(res)}`;
      try {
        await invoke("hist_record", { server: selected.address, port: selected.port, duration_secs: secs, error: "" });
        hist = await invoke<Hist[]>("hist_list", { limit: 10 });
        lastPlayed = hist[0] ?? lastPlayed;
      } catch { /* riwayat opsional, abaikan */ }
      toast("Game diluncurkan");
      if (autoClose) launchMsg += " (launcher tetap terbuka di sesi ini)";
    } catch (e) {
      const msg = String(e);
      launchError = msg;
      try {
        await invoke("hist_record", { server: selected.address, port: selected.port, duration_secs: 0, error: msg });
      } catch { /* abaikan */ }
    } finally {
      launching = false;
    }
  }

  async function quickPlay() {
    if (!heroRow) {
      toast("Belum ada server. Tambah lewat Servers.");
      return;
    }
    openLaunch(heroRow);
  }

  async function toggleFav(r: Row) {
    try {
      if (isFav(r)) {
        await invoke("fav_remove", { server: r.address, port: r.port });
        toast("Server dihapus dari favorit");
      } else {
        await invoke("fav_add", { server: r.address, port: r.port, label: displayName(r) });
        toast("Server ditambahkan ke favorit");
      }
      favs = await invoke<Fav[]>("fav_list", {});
    } catch (e) {
      favError = String(e);
      toast(`Gagal favorit: ${String(e)}`);
    }
  }

  async function enrich(rowsToCheck: Row[]) {
    const pending = rowsToCheck.filter((r) => !infoMap[r.key] || infoMap[r.key].error);
    for (const r of pending.slice(0, 12)) {
      infoMap[r.key] = { loading: true, error: "" };
      try {
        const info = await invoke<ServerInfo>("query_server_info", { address: r.address, port: r.port });
        infoMap[r.key] = { loading: false, error: "", info };
      } catch (e) {
        infoMap[r.key] = { loading: false, error: String(e) };
      }
    }
    infoMap = { ...infoMap };
  }

  async function loadServers(force: boolean) {
    serversLoading = true;
    serversError = "";
    try {
      servers = await invoke<ServerEntry[]>("discover_servers", { force });
      if (!servers.length) serversError = "Daftar server kosong dari sumber data.";
    } catch (e) {
      serversError = String(e);
    } finally {
      serversLoading = false;
    }
    if (servers.length) void enrich(servers.slice(0, 12).map(entryRow));
  }

  async function retryQuery(r: Row) {
    infoMap[r.key] = { loading: true, error: "" };
    infoMap = { ...infoMap };
    try {
      const info = await invoke<ServerInfo>("query_server_info", { address: r.address, port: r.port });
      infoMap[r.key] = { loading: false, error: "", info };
    } catch (e) {
      infoMap[r.key] = { loading: false, error: String(e) };
    }
    infoMap = { ...infoMap };
  }

  async function joinDirect() {
    directError = "";
    try {
      const { host, port } = parseDirect(directInput);
      const r: Row = { key: rowKey(host, port), address: host, port, omp: false, passworded: false };
      openLaunch(r);
      void retryQuery(r);
    } catch (e) {
      directError = String(e);
    }
  }

  async function onDetect() {
    if (!gtaPath.trim()) {
      gtaMsg = "Isi path folder GTA dulu.";
      gtaState = "error";
      return;
    }
    gtaState = "loading";
    gtaMsg = "Memeriksa gta_sa.exe...";
    try {
      const info = await invoke<GameInstall>("detect_gta", { path: gtaPath.trim() });
      gta = info;
      gtaPath = typeof info.gta_path === "string" ? info.gta_path : gtaPath;
      gtaMsg = info.valid ? `Terdeteksi: ${info.detail}` : `Versi bermasalah: ${info.detail}`;
      gtaState = info.valid ? "success" : "error";
      await invoke("save_setting", { key: "gta_path", value: gtaPath });
      const st = await invoke<LauncherStatus>("get_status", {});
      clients = st.clients;
      needsAttention = st.needs_attention;
    } catch (e) {
      gtaState = "error";
      gtaMsg = String(e);
    }
  }

  async function copyPath() {
    try {
      await navigator.clipboard.writeText(gtaPath);
      toast("Path disalin");
    } catch {
      try {
        const ta = document.createElement("textarea");
        ta.value = gtaPath;
        document.body.appendChild(ta);
        ta.select();
        document.execCommand("copy");
        ta.remove();
        toast("Path disalin");
      } catch {
        toast("Gagal menyalin path");
      }
    }
  }

  async function saveNickname() {
    nickMsg = "Menyimpan...";
    try {
      const saved = await invoke<string>("set_nickname", { nick: nickname });
      nickname = saved;
      await invoke("save_setting", { key: "nickname", value: nickname });
      nickMsg = "Nickname tersimpan.";
      toast("Nickname tersimpan");
    } catch (e) {
      nickMsg = String(e);
    }
  }

  async function persistSetting(key: string, value: string) {
    try {
      await invoke("save_setting", { key, value });
    } catch (e) {
      toast(`Gagal simpan ${key}: ${String(e)}`);
    }
  }

  async function onToggleStartup(checked: boolean) {
    startup = checked;
    await persistSetting("startup", checked ? "1" : "0");
    startupMsg = "Menyimpan...";
    try {
      const st = await invoke<{ enabled: boolean; detail?: string }>("set_autostart", { enabled: checked });
      startupMsg = st.detail || (st.enabled ? "Autostart aktif." : "Autostart nonaktif.");
    } catch (e) {
      startupMsg = String(e);
    }
  }

  async function openRepair() {
    modal = "repair";
    if (!gtaPath) {
      repairError = "Folder GTA belum diatur. Isi dulu di Library.";
      return;
    }
    repairLoading = true;
    repairError = "";
    try {
      repairItems = await invoke<RepairItem[]>("repair_check", { gta_path: gtaPath });
    } catch (e) {
      repairError = String(e);
    } finally {
      repairLoading = false;
    }
  }

  async function runNetwork() {
    netError = "";
    netInfo = null;
    if (!netTarget) {
      netError = "Pilih server dulu.";
      return;
    }
    const m = netTarget.match(/^(.+):(\d{1,5})$/);
    if (!m) {
      netError = "Target harus IP:port.";
      return;
    }
    netLoading = true;
    try {
      netInfo = await invoke<ServerInfo>("query_server_info", { address: m[1], port: Number(m[2]) });
    } catch (e) {
      netError = String(e);
    } finally {
      netLoading = false;
    }
  }

  async function stageDownload() {
    dlMsg = "";
    dlError = "";
    const size = Number(dlSize);
    if (!dlUrl || !dlSha || !dlName || !size) {
      dlError = "Lengkapi URL, SHA256, nama file, dan ukuran.";
      return;
    }
    try {
      dlMsg = await invoke<string>("download_client", { url: dlUrl, sha256: dlSha, filename: dlName, size_bytes: size });
    } catch (e) {
      dlError = String(e);
    }
  }

  onMount(async () => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") modal = null;
    };
    window.addEventListener("keydown", onKey);
    try {
      const st = await invoke<LauncherStatus>("get_status", {});
      gta = st.gta;
      clients = st.clients;
      nickname = st.nickname || "Player";
      needsAttention = st.needs_attention;
      if (st.gta) gtaPath = typeof st.gta.gta_path === "string" ? st.gta.gta_path : "";
    } catch (e) {
      statusError = String(e);
    } finally {
      statusLoading = false;
    }
    try {
      const wv = await invoke<{ installed: boolean; detail: string }>("webview_status", {});
      webviewOk = wv.installed;
      webviewMsg = wv.installed ? "" : wv.detail;
    } catch { webviewOk = true; }
    try {
      const s = await invoke<Fav[]>("fav_list", {});
      favs = s;
    } catch (e) { favError = String(e); }
    try {
      hist = await invoke<Hist[]>("hist_list", { limit: 10 });
      lastPlayed = hist[0] ?? null;
    } catch { /* riwayat boleh kosong */ }
    const get = async (k: string) => {
      try {
        const v = await invoke<string | null>("get_setting", { key: k });
        return v ?? null;
      } catch { return null; }
    };
    const [mp, ac, su, rm, gp, nn] = await Promise.all([get("mod_profile"), get("autoClose"), get("startup"), get("reduceMotion"), get("gta_path"), get("nickname")]);
    if (mp) modProfile = mp;
    if (ac) autoClose = ac === "1";
    if (su) startup = su === "1";
    if (rm) reduceMotion = rm === "1";
    if (gp && !gtaPath) gtaPath = gp;
    if (nn && !nickname) nickname = nn;
    try {
      const a = await invoke<{ enabled: boolean }>("get_autostart", {});
      startup = a.enabled;
    } catch { /* biarkan nilai settings */ }
    await loadServers(false);
    return () => window.removeEventListener("keydown", onKey);
  });

  const navigation = [{ name: "Home", icon: Home }, { name: "Servers", icon: Server }, { name: "Library", icon: Layers3 }, { name: "Settings", icon: Settings }] as const;
</script>

<svelte:head><title>ICARUZ - {page}</title></svelte:head>
<div class:reduce-motion={reduceMotion} class="app-shell">
  <aside class:open={sidebarOpen} class="sidebar">
    <div class="brand" role="button" tabindex="0" onclick={() => nav("Home")} onkeydown={(e) => e.key === "Enter" && nav("Home")}>
      <div class="brand-mark"><span>I</span><i></i></div>
      <div><div class="brand-name">ICARUZ<span>.</span></div><div class="brand-sub">MULTIPLAYER LAUNCHER</div></div>
    </div>
    <div class="nav-label">WORKSPACE</div>
    <nav aria-label="Navigasi utama">
      {#each navigation as item}
        <button class:active={page === item.name} class="nav-item" onclick={() => nav(item.name)}><item.icon size={19} strokeWidth={1.8}/><span>{item.name}</span>{#if page === item.name}<div class="nav-active-dot"></div>{/if}</button>
      {/each}
    </nav>
    <div class="sidebar-spacer"></div>
    <div class="sidebar-note"><div class="note-icon"><ShieldCheck size={17}/></div><div><strong>{gta ? (gta.valid ? "Ready to play" : "Check needed") : "Setup needed"}</strong><small>{statusLoading ? "Memeriksa..." : gta ? gta.detail : (needsAttention || "Folder GTA belum diatur")}</small></div><span class="status-dot"></span></div>
    <button class="profile" onclick={() => nav("Settings")}><div class="avatar">{(nickname || "P").slice(0, 1).toUpperCase()}</div><div class="profile-info"><strong>{nickname || "Player"}</strong><small>Local player</small></div><ChevronRight size={16}/></button>
    <div class="sidebar-credit">Developed by <strong>Fajuu</strong> <span>·</span> v{APP_VERSION}</div>
  </aside>
  {#if sidebarOpen}<button class="mobile-backdrop" aria-label="Tutup navigasi" onclick={() => (sidebarOpen = false)}></button>{/if}
  <div class="main-wrap">
    <header class="topbar">
      <button class="icon-btn mobile-menu" aria-label="Buka menu" onclick={() => (sidebarOpen = !sidebarOpen)}><Menu size={21}/></button>
      <div class="breadcrumbs"><span>ICARUZ</span><ChevronRight size={14}/><strong>{page}</strong></div>
      <div class="top-actions">
        <div class="top-search"><Search size={17}/><input aria-label="Cari server" placeholder="Cari server atau gamemode..." bind:value={query} onfocus={() => { if (page !== "Servers") page = "Servers"; }}/><kbd>⌘ K</kbd></div>
        <div class="notification-wrap"><button class="icon-btn" aria-label="Notifikasi" onclick={() => (showNotifications = !showNotifications)}><Bell size={19}/>{#if notifications.length}<span class="tiny-dot"></span>{/if}</button>{#if showNotifications}<div class="notification-popover"><strong>Notifikasi</strong>{#if notifications.length}{#each notifications as n}<p>{n}</p>{/each}{:else}<p>Tidak ada masalah. Semua siap.</p>{/if}</div>{/if}</div>
        <div class="header-avatar">{(nickname || "P").slice(0, 1).toUpperCase()}</div>
      </div>
    </header>
    <main>
      {#if !webviewOk && webviewMsg}
        <div class="webview-banner"><AlertCircle size={16}/><span>{webviewMsg}</span></div>
      {/if}
      {#if page === "Home"}
        <section class="welcome"><div><div class="eyebrow"><span class="eyebrow-line"></span> YOUR MULTIPLAYER HUB</div><h1>Play your world<span class="accent">.</span></h1><p>Semua server, satu tempat. Kembali ke permainan tanpa ribet.</p></div><div class="header-date">WINDOWS EDITION <span> / </span> 01</div></section>
        <div class="home-grid">
          <div class="primary-column">
            <section class="hero"><div class="hero-grid"></div><div class="hero-light"></div><div class="hero-content"><div class="hero-top"><span class="hero-tag"><span class="pulse-dot"></span> LAST PLAYED SERVER</span><span class="hero-counter">01 / 03</span></div><div class="hero-bottom"><div class="hero-kicker">CONTINUE YOUR JOURNEY</div>
              {#if heroRow}
                <h2>{displayName(heroRow)}</h2><div class="hero-meta"><span><Users size={15}/>{playerText(heroRow)} online</span><span><Wifi size={15}/>{pingText(heroRow)}</span><span class="meta-pill">{modeText(heroRow)}</span></div><div class="hero-buttons"><button class="primary-btn" onclick={quickPlay}><Play size={17} fill="currentColor"/> Quick Play <ArrowUpRight size={16}/></button><button class="ghost-btn" onclick={() => nav("Servers")}>Explore servers <ChevronRight size={16}/></button></div>
              {:else}
                <h2>Belum ada server</h2><p>Ambil daftar server dulu, lalu mulai bermain.</p><div class="hero-buttons"><button class="primary-btn" onclick={() => nav("Servers")}>Buka browser server <ArrowUpRight size={16}/></button></div>
              {/if}
            </div></div><div class="hero-art"><div class="sun"></div><div class="building b1"></div><div class="building b2"></div><div class="building b3"></div><div class="building b4"></div><div class="road"></div><div class="hero-art-text">LOS<br/>SANTOS</div></div></section>
            <section class="section-block"><div class="section-heading"><div><div class="eyebrow muted">DISCOVER</div><h3>Popular right now <span class="heading-count">0{Math.min(visibleRows.length, 9)}</span></h3></div><button class="text-link" onclick={() => nav("Servers")}>View all servers <ArrowUpRight size={16}/></button></div>
              {#if serversLoading}<div class="empty-state"><RefreshCcw size={28}/><h3>Memuat server...</h3><p>Mengambil daftar dari sumber data.</p></div>
              {:else if serversError}<div class="empty-state"><AlertCircle size={28}/><h3>Gagal memuat server</h3><p>{serversError}</p><button onclick={() => loadServers(true)}>Coba lagi</button></div>
              {:else if !visibleRows.length}<div class="empty-state"><Search size={28}/><h3>Belum ada server cocok</h3><p>Ubah kata kunci atau kategori.</p><button onclick={() => nav("Servers")}>Buka browser</button></div>
              {:else}<div class="server-list">{#each visibleRows.slice(0, 3) as s}<div class="server-row"><div class="server-thumb"><Gamepad2 size={20}/></div><div class="server-details"><strong>{displayName(s)}</strong><small>{modeText(s)} <span>·</span> {s.omp ? "open.mp" : "SA-MP"}</small></div><div class="server-stats"><span><Users size={14}/>{playerText(s)}</span><span class="ping"><span class="ping-bars">▂▄▆</span>{pingText(s)}</span></div><button class="row-play" aria-label="Main di {displayName(s)}" onclick={() => openLaunch(s)}><Play size={16} fill="currentColor"/></button></div>{/each}</div>{/if}
            </section>
          </div>
          <div class="side-column">
            <section class="panel game-panel"><div class="panel-title"><span>GAME STATUS</span><CircleDot size={17}/></div>
              {#if statusLoading}<p class="field-help">Memeriksa instalasi...</p>
              {:else if statusError}<p class="field-help">Gagal baca status: {statusError}</p>
              {:else}<div class="game-identity"><div class="game-symbol">SA</div><div><strong>GTA San Andreas</strong><small>{gta ? `${gta.exe_version} · ${gta.valid ? "Clean" : "Perlu cek"}` : "Belum diatur"}</small></div></div>
              {#if gta}<div class="healthy"><Check size={15}/> {gta.valid ? "Ready to launch" : "Perlu perhatian"}</div><div class="divider"></div><div class="mini-stat"><span>Installation</span><strong>Terdeteksi <Check size={13}/></strong></div><div class="mini-stat"><span>Detail</span><strong>{gta.detail}</strong></div><div class="mini-stat"><span>SHA256</span><strong>{gta.exe_sha256.slice(0, 16)}...</strong></div>
              {:else}<div class="demo-warning"><AlertCircle size={15}/> {needsAttention || "Folder GTA belum diatur."}</div>{/if}
              <div class="mini-stat"><span>Active client</span><strong>{activeClient ? CLIENT_LABEL[activeClient.kind] : "Belum ada"}</strong></div><div class="mini-stat"><span>Mod profile</span><strong>{modProfile}</strong></div><button class="panel-link" onclick={() => nav("Library")}>Manage game files <ArrowUpRight size={16}/></button>{/if}
            </section>
            <section class="panel clients-panel"><div class="panel-title"><span>YOUR CLIENTS</span><Layers3 size={17}/></div>
              {#if !clients.length}<p class="field-help">Belum ada data client.</p>
              {:else}{#each clients as c}<div class="client-row"><div class="client-icon {c.kind === 'OpenMp1x' ? 'client-purple' : c.kind === 'Samp037R5' ? 'client-orange' : 'client-blue'}">{CLIENT_LABEL[c.kind].slice(0, 1)}</div><div><strong>{CLIENT_LABEL[c.kind]}</strong><small>{c.version}{c.installed && c.path ? ` · ${c.path}` : ""}</small></div>{#if c.installed}<Check class="client-check" size={16}/>{:else}<span class="client-missing">belum terpasang</span>{/if}</div>{/each}{/if}
            </section>
            <section class="tip-panel"><div class="tip-top"><Zap size={17}/><span>THE ICARUZ WAY</span></div><h4>One click.<br/>Every server.</h4><p>Client switching, compatibility, and game profiles, made effortless.</p><button onclick={() => { modal = "about"; }}>Learn more <ArrowUpRight size={15}/></button></section>
          </div>
        </div>
      {:else if page === "Servers"}
        <section class="page-heading"><div class="eyebrow">FIND YOUR PLACE</div><h1>Server browser<span class="accent">.</span></h1><p>Temukan komunitas yang cocok dan langsung bergabung.</p></section>
        <section class="panel direct-panel"><div class="panel-title"><span>DIRECT CONNECT</span><Wifi size={17}/></div><div class="direct-row"><input aria-label="Server IP dan port" placeholder="Contoh 127.0.0.1:7777" bind:value={directInput}/><button class="primary-btn" onclick={joinDirect}>Join <ArrowUpRight size={16}/></button></div>{#if directError}<p class="field-error">{directError}</p>{/if}</section>
        <div class="filterbar"><div class="filter-chips">{#each categories as c}<button class:chosen={category === c} onclick={() => (category = c)}>{#if c === "Favorit"}<Star size={14}/>{/if}{c}</button>{/each}</div><label class="sort-control"><SlidersHorizontal size={16}/><select bind:value={sortBy}><option>Populer</option><option>Ping terendah</option></select></label><button class="secondary-btn" onclick={() => loadServers(true)}>Refresh</button></div>
        <div class="results-title"><strong>{visibleRows.length} servers found</strong><span>{serversLoading ? "memuat..." : serversError ? "gagal memuat" : "data live dari discover_servers"}</span></div>
        {#if serversLoading}<div class="empty-state"><RefreshCcw size={30}/><h3>Memuat server...</h3><p>Mengambil daftar dari sumber data.</p></div>
        {:else if serversError}<div class="empty-state"><AlertCircle size={30}/><h3>Gagal memuat server</h3><p>{serversError}</p><button onclick={() => loadServers(true)}>Coba lagi</button></div>
        {:else}<div class="browser-list">{#each visibleRows as r (r.key)}<article class="browser-card"><div class="browser-icon"><Gamepad2 size={24}/></div><div class="browser-main"><div class="browser-name"><h3>{displayName(r)}</h3>{#if r.passworded}<span class="locked-label">LOCKED</span>{/if}</div><p>{r.address}:{r.port}</p><div class="browser-tags"><span>{modeText(r)}</span><span>{r.omp ? "open.mp" : "SA-MP"}</span><span>{infoMap[r.key]?.info?.language || "?"}</span></div>{#if infoMap[r.key]?.error}<p class="field-help">Query gagal, tampil offline. <button class="text-link" onclick={() => retryQuery(r)}>Coba query ulang</button></p>{/if}</div><div class="browser-numbers"><div><Users size={15}/><strong>{playerText(r)}</strong><small>PLAYERS</small></div><div><Wifi size={15}/><strong>{pingText(r)}</strong><small>LATENCY</small></div></div><div class="browser-actions"><button class:favorite-on={isFav(r)} class="favorite-btn" aria-label="Favorit {displayName(r)}" onclick={() => toggleFav(r)}><Star size={18} fill={isFav(r) ? "currentColor" : "none"}/></button><button class="join-btn" onclick={() => openLaunch(r)}>Join <ArrowUpRight size={16}/></button></div></article>{:else}<div class="empty-state"><Search size={30}/><h3>Tidak ada server ditemukan</h3><p>Coba kata kunci atau kategori lain.</p><button onclick={() => { query = ""; category = "Semua"; }}>Reset filter</button></div>{/each}</div>{/if}
      {:else if page === "Library"}
        <section class="page-heading"><div class="eyebrow">YOUR ENVIRONMENT</div><h1>Game library<span class="accent">.</span></h1><p>Kelola instalasi game, client multiplayer, dan profil mod.</p></section>
        <div class="library-grid">
          <section class="panel library-main"><div class="panel-title"><span>BASE GAME</span><HardDrive size={18}/></div><div class="library-game"><div class="library-art">SA</div><div><h3>Grand Theft Auto: San Andreas</h3><p>{gta ? `Windows PC · ${gta.exe_version}` : "Windows PC · Belum terdeteksi"}</p>{#if gtaState === "loading"}<p class="field-help">Memeriksa...</p>{:else if gta}<div class="healthy inline"><Check size={15}/> {gta.valid ? "Installation detected" : "Ditemukan, perlu cek"}</div>{:else}<div class="demo-warning"><AlertCircle size={15}/> {needsAttention || "Folder GTA belum diatur."}</div>{/if}</div></div>
            <div class="library-path"><FolderOpen size={17}/><span>{gtaPath || "Belum diatur"}</span><button aria-label="Salin lokasi" onclick={copyPath}><Copy size={15}/></button></div>
            <div class="direct-row"><input aria-label="Path folder GTA" placeholder="Contoh C:\Games\GTA San Andreas" bind:value={gtaPath}/><button class="secondary-btn" onclick={onDetect}>Check</button></div>
            {#if gtaMsg}<p class={gtaState === "error" ? "field-error" : "field-help"}>{gtaMsg}</p>{/if}
          </section>
          <section class="panel"><div class="panel-title"><span>MOD PROFILE</span><SlidersHorizontal size={17}/></div><div class="profile-options">{#each ["Vanilla", "Modded"] as profile}<button class:profile-selected={modProfile === profile} onclick={() => { modProfile = profile; persistSetting("mod_profile", profile); toast(`Profil ${profile} dipilih`); }}><div class="profile-icon">{#if profile === "Vanilla"}<ShieldCheck size={20}/>{:else}<Wrench size={20}/>{/if}</div><div><strong>{profile}</strong><small>{profile === "Vanilla" ? "Clean, recommended" : "Your custom environment"}</small></div><span class="radio-circle">{#if modProfile === profile}<span></span>{/if}</span></button>{/each}</div></section>
          <section class="panel library-main"><div class="panel-title"><span>INSTALLED CLIENTS</span><Layers3 size={18}/></div>{#if !clients.length}<p class="field-help">Belum ada data client. Buka ulang aplikasi.</p>{:else}{#each clients as c}<div class="library-client"><div class="client-icon {c.kind === 'OpenMp1x' ? 'client-purple' : c.kind === 'Samp037R5' ? 'client-orange' : 'client-blue'}">{CLIENT_LABEL[c.kind].slice(0, 1)}</div><div><strong>{CLIENT_LABEL[c.kind]}</strong><small>{c.version}{c.installed && c.path ? ` · ${c.path}` : ""}</small></div><span>{c.installed ? "READY" : "belum terpasang"}</span></div>{/each}{/if}</section>
          <section class="panel tools-panel"><div class="panel-title"><span>QUICK TOOLS</span><Wrench size={18}/></div><button onclick={openRepair}><RotateCcw size={18}/><div><strong>Repair game files</strong><small>Validate game environment</small></div><ChevronRight size={17}/></button><button onclick={() => { netTarget = heroRow ? `${heroRow.address}:${heroRow.port}` : ""; netInfo = null; netError = ""; modal = "network"; }}><Wifi size={18}/><div><strong>Network diagnostics</strong><small>Check server latency</small></div><ChevronRight size={17}/></button><button onclick={() => { dlMsg = ""; dlError = ""; modal = "downloads"; }}><Download size={18}/><div><strong>Client downloads</strong><small>Staging terverifikasi</small></div><ChevronRight size={17}/></button></section>
        </div>
      {:else}
        <section class="page-heading"><div class="eyebrow">MAKE IT YOURS</div><h1>Settings<span class="accent">.</span></h1><p>Atur pengalaman bermain sesuai preferensimu.</p></section>
        <div class="settings-grid">
          <section class="panel settings-panel"><div class="panel-title"><span>PLAYER</span><Gamepad2 size={17}/></div><label class="field-label" for="nickname">Nickname</label><input id="nickname" class="settings-input" bind:value={nickname} maxlength={24}/><p class="field-help">Nama yang digunakan saat bergabung ke server.</p>{#if nickMsg}<p class="field-help">{nickMsg}</p>{/if}<button class="secondary-btn" onclick={saveNickname}>Save changes</button></section>
          <section class="panel settings-panel"><div class="panel-title"><span>LAUNCHER</span><Monitor size={17}/></div>
            <label class="toggle-row"><div><strong>Close launcher after launch</strong><small>Free up resources while playing</small></div><input type="checkbox" bind:checked={autoClose} onchange={() => persistSetting("autoClose", autoClose ? "1" : "0")}/><span class="switch"></span></label>
            <label class="toggle-row"><div><strong>Launch at Windows startup</strong><small>Open ICARUZ when you sign in</small></div><input type="checkbox" bind:checked={startup} onchange={(e) => onToggleStartup(e.currentTarget.checked)}/><span class="switch"></span></label>
            {#if startupMsg}<p class="field-help">{startupMsg}</p>{/if}
            <label class="toggle-row"><div><strong>Reduce motion</strong><small>Minimize interface animations</small></div><input type="checkbox" bind:checked={reduceMotion} onchange={() => persistSetting("reduceMotion", reduceMotion ? "1" : "0")}/><span class="switch"></span></label>
          </section>
          <section class="panel settings-panel"><div class="panel-title"><span>ABOUT</span><Info size={17}/></div><div class="about-brand">ICARUZ<span>.</span></div><p class="about-text">A universal GTA multiplayer experience. Built for simplicity, reliability, and performance.</p><div class="mini-stat"><span>Version</span><strong>{APP_VERSION}</strong></div><div class="mini-stat"><span>Developer</span><strong>Fajuu</strong></div><button class="secondary-btn" onclick={() => { modal = "about"; }}>About ICARUZ <ArrowUpRight size={16}/></button></section>
        </div>
      {/if}
      <footer><span>ICARUZ © 2026 <span class="footer-divider">/</span> Developed by Fajuu</span><span>v{APP_VERSION} · {gta ? "GTA terdeteksi" : "GTA belum diatur"}</span></footer>
    </main>
  </div>
  {#if notification}<div class="toast"><Check size={17}/>{notification}<button aria-label="Tutup" onclick={() => (notification = "")}><X size={16}/></button></div>{/if}
  {#if modal}
    <div class="modal-backdrop" role="presentation" onclick={() => (modal = null)}>
      <div class="modal" role="dialog" aria-modal="true" aria-label="Dialog ICARUZ" onclick={(e) => e.stopPropagation()}>
        <button class="modal-close" aria-label="Tutup" onclick={() => (modal = null)}><X size={19}/></button>
        {#if modal === "launch" && selected}
          <div class="modal-icon"><Play size={24}/></div><div class="eyebrow">QUICK PLAY</div><h2>Ready for {displayName(selected)}?</h2><p>{selected.address}:{selected.port} · Profil {modProfile}</p>
          <div class="modal-summary"><div><span>Server</span><strong>{displayName(selected)}</strong></div><div><span>Pemain</span><strong>{playerText(selected)}</strong></div><div><span>Ping</span><strong>{pingText(selected)}</strong></div></div>
          <div class="field-label">Client</div>
          <div class="profile-options">{#each CLIENT_OPTS as client}<button class:profile-selected={launchClient === client} onclick={() => (launchClient = client)}><div><strong>{CLIENT_LABEL[client]}</strong><small>{clients.find((c) => c.kind === client)?.installed ? "terpasang" : "belum terpasang"}</small></div><span class="radio-circle">{#if launchClient === client}<span></span>{/if}</span></button>{/each}</div>
          {#if planNote}<p class="field-help">{planNote}</p>{/if}
          {#if launchMsg && !launchError}<p class="field-help">{launchMsg}</p>{/if}
          {#if launchError}<p class="field-error">{launchError}</p>{/if}
          <button class="primary-btn full" disabled={launching} onclick={doLaunch}>{launching ? "Meluncurkan..." : "Play"} <ArrowUpRight size={17}/></button>
        {:else if modal === "repair"}
          <div class="modal-icon"><Wrench size={24}/></div><h2>Game repair</h2>
          {#if repairLoading}<p>Memeriksa file game...</p>
          {:else if repairError}<p class="field-error">{repairError}</p><button class="primary-btn full" onclick={() => (modal = null)}>Close <Check size={17}/></button>
          {:else if !repairItems.length}<p>Belum ada hasil. Jalankan dari Library.</p><button class="primary-btn full" onclick={() => (modal = null)}>Close <Check size={17}/></button>
          {:else}<div class="repair-list">{#each repairItems as item}<div class="repair-row"><div><strong>{item.label || item.title || item.id}</strong><small>{item.status} · {item.detail}</small>{#if item.fix_hint}<small>Saran: {item.fix_hint}</small>{/if}{#if item.needs_approval}<small>Perlu persetujuan sebelum diperbaiki.</small>{/if}</div></div>{/each}</div><button class="primary-btn full" onclick={() => (modal = null)}>Close <Check size={17}/></button>{/if}
        {:else if modal === "network"}
          <div class="modal-icon"><Wifi size={24}/></div><h2>Network diagnostics</h2><p>Hasil ping asli dari query UDP ke server.</p>
          <label class="field-label" for="net-target">Target IP:port</label>
          <input id="net-target" bind:value={netTarget} placeholder="Contoh 127.0.0.1:7777"/>
          <div class="modal-buttons"><button class="primary-btn" disabled={netLoading} onclick={runNetwork}>{netLoading ? "Memeriksa..." : "Check"}</button></div>
          {#if netError}<p class="field-error">{netError}</p>{/if}
          {#if netInfo}<div class="modal-summary"><div><span>Hostname</span><strong>{netInfo.hostname}</strong></div><div><span>Pemain</span><strong>{netInfo.players}/{netInfo.max_players}</strong></div><div><span>Ping</span><strong>{netInfo.ping_ms} ms</strong></div><div><span>Gamemode</span><strong>{netInfo.gamemode}</strong></div></div>{/if}
        {:else if modal === "downloads"}
          <div class="modal-icon"><Download size={24}/></div><h2>Client downloads</h2><p>Tahap staging: file disiapkan dan diverifikasi, belum dipasang otomatis.</p>
          <label class="field-label" for="dl-url">URL</label><input id="dl-url" bind:value={dlUrl} placeholder="https://..."/>
          <label class="field-label" for="dl-sha">SHA256</label><input id="dl-sha" bind:value={dlSha} placeholder="hex 64 karakter"/>
          <label class="field-label" for="dl-name">Nama file</label><input id="dl-name" bind:value={dlName} placeholder="client.zip"/>
          <label class="field-label" for="dl-size">Ukuran bytes</label><input id="dl-size" bind:value={dlSize} placeholder="contoh 1024"/>
          <div class="modal-buttons"><button class="primary-btn" onclick={stageDownload}>Siapkan staging</button></div>
          {#if dlError}<p class="field-error">{dlError}</p>{/if}
          {#if dlMsg}<p class="field-help">{dlMsg}</p>{/if}
        {:else}
          <div class="modal-icon"><Info size={24}/></div><h2>ICARUZ<span class="accent">.</span></h2><p>Universal GTA Multiplayer Launcher. Client switching, compatibility, and game profiles, made effortless.</p><div class="modal-summary"><div><span>Developed by</span><strong>Fajuu</strong></div><div><span>Version</span><strong>{APP_VERSION}</strong></div><div><span>Stack</span><strong>Svelte 5 + Tauri 2 + Rust</strong></div></div><button class="primary-btn full" onclick={() => (modal = null)}>Close <Check size={17}/></button>
        {/if}
      </div>
    </div>
  {/if}
</div>

<style>
  .direct-panel { margin-bottom: 14px; }
  .direct-row { display: flex; gap: 10px; }
  .direct-row input { flex: 1; }
  .field-error { color: #ff9d9d; font-size: 13px; margin: 8px 0 0; }
  .webview-banner { display: flex; gap: 10px; align-items: center; background: #2a2030; border: 1px solid #5a4a2a; border-radius: 10px; padding: 10px 14px; margin-bottom: 14px; font-size: 13px; }
  .locked-label { font-size: 10px; font-weight: 800; letter-spacing: 1px; background: #2a2f45; padding: 3px 8px; border-radius: 6px; }
  .client-missing { font-size: 11px; color: #8e99ac; }
  .modal-buttons { display: flex; gap: 10px; margin: 12px 0; }
  .repair-list { display: grid; gap: 10px; margin: 12px 0; max-height: 320px; overflow: auto; }
  .repair-row { background: #161b27; border: 1px solid #232b3d; border-radius: 10px; padding: 10px 12px; }
  .repair-row small { display: block; color: #8e99ac; margin-top: 4px; }
</style>

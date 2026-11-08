<script lang="ts">
  import { onMount } from "svelte";

  type Page = "home" | "servers" | "library" | "settings";
  type Phase = "idle" | "checking" | "preparing" | "launching" | "running" | "error";

  let page: Page = "home";
  let status = "Checking local setup...";
  let gtaPath = "";
  let gtaDetail = "";
  let nickname = "Player";
  let serverInput = "127.0.0.1:7777";
  let phase: Phase = "idle";
  let phaseMsg = "";
  let notice = "";

  async function invoke(cmd: string, args?: unknown) {
    const mod = await import("@tauri-apps/api/core");
    return mod.invoke(cmd, args as Record<string, unknown>);
  }

  onMount(async () => {
    try {
      const data = (await invoke("get_status")) as { needs_attention?: string };
      status = "Local core ready.";
      notice = data.needs_attention ?? "";
    } catch {
      status = "Tauri IPC not available in browser preview.";
      notice = "GTA San Andreas folder not set yet.";
    }
  });

  async function onDetect() {
    phase = "checking";
    phaseMsg = "Checking gta_sa.exe...";
    try {
      const info = (await invoke("detect_gta", { path: gtaPath })) as {
        valid: boolean;
        detail: string;
        exe_sha256: string;
      };
      gtaDetail = `${info.detail} sha256 ${String(info.exe_sha256).slice(0, 16)}...`;
      notice = info.valid ? "" : "Version game belum cocok. Periksa file sebelum Play.";
      phase = "idle";
      phaseMsg = "";
    } catch (e) {
      phase = "error";
      phaseMsg = String(e);
    }
  }

  async function onPlay() {
    phase = "checking";
    phaseMsg = "Checking nickname and address...";
    try {
      if (!gtaPath) throw "Set GTA folder dulu di Library atau Home.";
      phase = "preparing";
      phaseMsg = "Preparing SA-MP 0.3.7-R5 plan...";
      const plan = (await invoke("plan_launch", {
        req: { client: "Samp037R5", gta_path: gtaPath, server: serverInput, port: 7777, nick: nickname },
      })) as { exe: string; args: string[] };
      phase = "launching";
      phaseMsg = `Launching ${plan.exe}...`;
      const res = await invoke("launch_game", {
        req: { client: "Samp037R5", gta_path: gtaPath, server: serverInput, port: 7777, nick: nickname },
      });
      phase = "running";
      phaseMsg = `Running: ${JSON.stringify(res)}`;
    } catch (e) {
      phase = "error";
      phaseMsg = String(e);
    }
  }

  function goServers() {
    page = "servers";
  }
</script>

<div class="shell">
  <nav class="side" aria-label="Primary">
    <div class="brand">ICARUZ</div>
    <button class="nav-item" aria-current={page === "home" ? "page" : undefined} onclick={() => (page = "home")}>Home</button>
    <button class="nav-item" aria-current={page === "servers" ? "page" : undefined} onclick={goServers}>Servers</button>
    <button class="nav-item" aria-current={page === "library" ? "page" : undefined} onclick={() => (page = "library")}>Library</button>
    <button class="nav-item" aria-current={page === "settings" ? "page" : undefined} onclick={() => (page = "settings")}>Settings</button>
    <div class="muted" style="margin-top:auto; padding: 10px;">Developed by Fajuu</div>
  </nav>

  <main class="main">
    {#if page === "home"}
      <h1>Open ICARUZ, pick a server, press Play</h1>
      <p class="muted">Local game files stay untouched until you approve a change.</p>

      <section class="card" aria-label="Quick play">
        <h2>Quick play (SA-MP 0.3.7-R5)</h2>
        <div class="row">
          <label for="gta">GTA folder</label>
          <input id="gta" bind:value={gtaPath} placeholder="D:/Games/GTA San Andreas" />
          <button onclick={onDetect} disabled={phase === "checking"}>Check GTA</button>
        </div>
        {#if gtaDetail}<p class="muted">{gtaDetail}</p>{/if}
        <div class="row">
          <label for="nick2">Nickname</label>
          <input id="nick2" bind:value={nickname} minlength={3} maxlength={24} />
          <label for="srv">Server</label>
          <input id="srv" bind:value={serverInput} placeholder="127.0.0.1:7777" />
          <button class="primary" onclick={onPlay} disabled={phase === "launching" || phase === "preparing"}>Play</button>
        </div>
        <p role="status">Phase: {phase}{phaseMsg ? `, ${phaseMsg}` : ""}</p>
        {#if notice}<p>{notice}</p>{/if}
      </section>

      <section class="card" aria-label="Diagnostics">
        <h2>Local status</h2>
        <p class="muted">{status}</p>
      </section>
    {/if}

    {#if page === "servers"}
      <h1>Servers</h1>
      <div class="row">
        <label for="q">Direct connect</label>
        <input id="q" bind:value={serverInput} placeholder="IP:port" />
        <button class="primary" onclick={onPlay}>Join and Play</button>
      </div>
      <div class="card" role="status">
        <h2>No server list yet in P2</h2>
        <p class="muted">P2 focuses on direct connect. Discovery ships in P3 with cache and offline state.</p>
      </div>
      <p role="status">Phase: {phase}{phaseMsg ? `, ${phaseMsg}` : ""}</p>
    {/if}

    {#if page === "library"}
      <h1>Library</h1>
      <section class="card">
        <h2>GTA install</h2>
        <div class="row">
          <input bind:value={gtaPath} placeholder="D:/Games/GTA San Andreas" aria-label="GTA folder" />
          <button onclick={onDetect}>Validate</button>
        </div>
        {#if gtaDetail}<p class="muted">{gtaDetail}</p>{/if}
      </section>
      <section class="card">
        <h2>Clients</h2>
        <p class="muted">P2 wires SA-MP 0.3.7-R5. 0.3DL and open.mp show honest pending state until P3.</p>
      </section>
    {/if}

    {#if page === "settings"}
      <h1>Settings</h1>
      <section class="card">
        <h2>Profile</h2>
        <div class="row">
          <label for="nick">Nickname</label>
          <input id="nick" bind:value={nickname} minlength={3} maxlength={24} />
        </div>
      </section>
      <section class="card">
        <h2>About</h2>
        <p>ICARUZ launcher. Developed by Fajuu.</p>
      </section>
    {/if}
  </main>
</div>

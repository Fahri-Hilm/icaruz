<script lang="ts">
  import { onMount } from "svelte";

  type Page = "home" | "servers" | "library" | "settings";
  let page: Page = "home";
  let status: string = "Checking local setup...";
  let gtaPath = "";
  let nickname = "Player";
  let notice = "";
  let servers: Array<{ name: string; ip: string; port: number; players: string }> = [];
  let serversState: "empty" | "loading" | "ready" = "empty";

  onMount(async () => {
    try {
      const mod = await import("@tauri-apps/api/core");
      const data = await mod.invoke("get_status");
      status = JSON.stringify(data);
      if (typeof data === "object" && data !== null && "needs_attention" in data) {
        notice = String((data as { needs_attention: unknown }).needs_attention ?? "");
      }
    } catch {
      status = "Tauri IPC not available in browser preview.";
      notice = "GTA San Andreas folder not set yet.";
    }
  });

  function browseServers() {
    page = "servers";
    if (serversState === "empty") {
      serversState = "loading";
      setTimeout(() => {
        servers = [];
        serversState = "empty";
      }, 400);
    }
  }
</script>

<div class="shell">
  <nav class="side" aria-label="Primary">
    <div class="brand">ICARUZ</div>
    <button class="nav-item" aria-current={page === "home" ? "page" : undefined} onclick={() => (page = "home")}>Home</button>
    <button class="nav-item" aria-current={page === "servers" ? "page" : undefined} onclick={browseServers}>Servers</button>
    <button class="nav-item" aria-current={page === "library" ? "page" : undefined} onclick={() => (page = "library")}>Library</button>
    <button class="nav-item" aria-current={page === "settings" ? "page" : undefined} onclick={() => (page = "settings")}>Settings</button>
    <div class="muted" style="margin-top:auto; padding: 10px;">Developed by Fajuu</div>
  </nav>

  <main class="main">
    {#if page === "home"}
      <h1>Open ICARUZ, pick a server, press Play</h1>
      <p class="muted">Local game files stay untouched until you approve a change.</p>

      {#if notice}
        <section class="card" aria-label="Setup status">
          <h2>Needs attention</h2>
          <p>{notice}</p>
          <div class="row">
            <label for="gta">GTA folder</label>
            <input id="gta" bind:value={gtaPath} placeholder="D:/Games/GTA San Andreas" />
            <button>Use this folder</button>
          </div>
        </section>
      {:else}
        <section class="card" aria-label="Ready to play">
          <h2>Ready</h2>
          <p class="muted">No server selected yet. Browse the list to pick one.</p>
          <button class="primary" onclick={browseServers}>Browse servers</button>
        </section>
      {/if}

      <section class="card" aria-label="Diagnostics">
        <h2>Local status</h2>
        <p class="muted">{status}</p>
      </section>
    {/if}

    {#if page === "servers"}
      <h1>Servers</h1>
      <div class="row">
        <label for="q">Search</label>
        <input id="q" placeholder="Name, gamemode, or IP:port" />
        <button>Join by IP:port</button>
      </div>

      {#if serversState === "loading"}
        <div class="card" role="status"><p>Loading server list...</p></div>
      {:else if serversState === "empty"}
        <div class="card" role="status">
          <h2>No servers loaded</h2>
          <p class="muted">Discovery has not returned data yet. Cached entries will appear here with their age when offline.</p>
          <button>Retry discovery</button>
        </div>
      {:else}
        <ul>
          {#each servers as s}
            <li>{s.name} {s.ip}:{s.port} {s.players}</li>
          {/each}
        </ul>
      {/if}
    {/if}

    {#if page === "library"}
      <h1>Library</h1>
      <section class="card">
        <h2>GTA install</h2>
        <p class="muted">Set the folder containing gta_sa.exe. Validation checks version and hash before anything else runs.</p>
      </section>
      <section class="card">
        <h2>Clients</h2>
        <p class="muted">SA-MP 0.3.7-R5, SA-MP 0.3DL, open.mp 1.x appear here after detection. Each client keeps an isolated runtime.</p>
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

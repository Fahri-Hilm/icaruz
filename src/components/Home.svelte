<script lang="ts">
  export let gtaPath: string;
  export let nickname: string;
  export let serverInput: string;
  export let phase: string;
  export let phaseMsg: string;
  export let notice: string;
  export let status: string;
  export let gtaDetail: string;
  export let onDetect: () => void;
  export let onPlay: () => void;
  export let webviewMsg = "";

  $: busy = phase === "checking" || phase === "preparing" || phase === "launching";
  $: viewState = phase === "error" ? "error" : phase === "running" ? "success" : busy ? "loading" : !gtaPath ? "empty" : "ready";
</script>

<h1>Open ICARUZ, pick a server, press Play</h1>
<p class="muted">Local game files stay untouched until you approve a change.</p>

{#if webviewMsg}
  <section class="card" aria-label="WebView2 missing">
    <h2>WebView2 belum terpasang</h2>
    <p>{webviewMsg}</p>
    <p class="muted">Cari "WebView2 Runtime installer" di situs Microsoft, install, lalu buka ulang ICARUZ. Ukuran sekitar 100-180 MB, sekali saja.</p>
  </section>
{/if}

<section class="card" aria-label="Quick play">
  <h2>Quick play (SA-MP 0.3.7-R5)</h2>
  <div class="row">
    <label for="gta">GTA folder</label>
    <input id="gta" bind:value={gtaPath} placeholder="D:/Games/GTA San Andreas" autocomplete="off" />
    <button onclick={onDetect} disabled={busy}>Check GTA</button>
  </div>
  {#if gtaDetail}<p class="muted">{gtaDetail}</p>{/if}
  <div class="row">
    <label for="nick2">Nickname</label>
    <input id="nick2" bind:value={nickname} minlength={3} maxlength={24} autocomplete="off" />
    <label for="srv">Server</label>
    <input id="srv" bind:value={serverInput} placeholder="127.0.0.1:7777" autocomplete="off" />
    <button class="primary" onclick={onPlay} disabled={busy}>Play</button>
  </div>
  {#if viewState === "empty"}
    <p role="status" class="muted">Empty: set GTA folder first, then press Play.</p>
  {:else if viewState === "loading"}
    <p role="status" class="muted">Loading: {phaseMsg || phase}.</p>
  {:else if viewState === "error"}
    <p role="alert">Error: {phaseMsg || "Last action failed. Nothing was changed."}</p>
  {:else if viewState === "success"}
    <p role="status">Success: {phaseMsg || "Game process started."}</p>
  {:else}
    <p role="status" class="muted">Ready: {phaseMsg || "Press Play to join."}</p>
  {/if}
  {#if notice}<p>{notice}</p>{/if}
</section>

<section class="card" aria-label="Diagnostics">
  <h2>Local status</h2>
  <p class="muted">{status}</p>
</section>

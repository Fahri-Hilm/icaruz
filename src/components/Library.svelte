<script lang="ts">
  export let gtaPath: string;
  export let gtaDetail: string;
  export let phase: string;
  export let phaseMsg: string;
  export let onDetect: () => void;

  $: busy = phase === "checking";
  $: viewState = phase === "error" ? "error" : gtaDetail ? "success" : busy ? "loading" : !gtaPath ? "empty" : "ready";
</script>

<h1>Library</h1>
<section class="card" aria-label="GTA install">
  <h2>GTA install</h2>
  <div class="row">
    <input bind:value={gtaPath} placeholder="D:/Games/GTA San Andreas" aria-label="GTA folder" autocomplete="off" />
    <button class="primary" onclick={onDetect} disabled={busy}>Validate</button>
  </div>
  {#if viewState === "empty"}
    <p role="status" class="muted">Empty: set the folder that holds gta_sa.exe.</p>
  {:else if viewState === "loading"}
    <p role="status" class="muted">Loading: checking gta_sa.exe.</p>
  {:else if viewState === "error"}
    <p role="alert">Error: {phaseMsg || "Validation failed. Files left untouched."}</p>
  {:else if viewState === "success"}
    <p role="status">{gtaDetail}</p>
  {:else}
    <p role="status" class="muted">Ready: press Validate to confirm the folder.</p>
  {/if}
</section>

<section class="card" aria-label="Clients">
  <h2>Clients</h2>
  <ul>
    <li>SA-MP 0.3.7-R5: wired in this build.</li>
    <li>SA-MP 0.3DL: pending, honest placeholder until installer ships.</li>
    <li>open.mp 1.x: pending, honest placeholder until installer ships.</li>
  </ul>
</section>

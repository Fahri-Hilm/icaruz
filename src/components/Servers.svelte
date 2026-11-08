<script lang="ts">
  export let serverInput: string;
  export let phase: string;
  export let phaseMsg: string;
  export let onPlay: () => void;

  $: busy = phase === "checking" || phase === "preparing" || phase === "launching";
  $: viewState = phase === "error" ? "error" : phase === "running" ? "success" : busy ? "loading" : "empty";
</script>

<h1>Servers</h1>
<section class="card" aria-label="Direct connect">
  <h2>Direct connect</h2>
  <div class="row">
    <label for="q">Address</label>
    <input id="q" bind:value={serverInput} placeholder="IP:port" autocomplete="off" />
    <button class="primary" onclick={onPlay} disabled={busy}>Join and Play</button>
  </div>
  {#if viewState === "loading"}
    <p role="status" class="muted">Loading: {phaseMsg || phase}.</p>
  {:else if viewState === "error"}
    <p role="alert">Error: {phaseMsg || "Join failed. Address kept for retry."}</p>
  {:else if viewState === "success"}
    <p role="status">Success: {phaseMsg || "Game process started."}</p>
  {:else}
    <p role="status" class="muted">Empty: type an address you own or trust, then join.</p>
  {/if}
</section>

<section class="card" aria-label="Server discovery">
  <h2>Saved servers</h2>
  <p role="status" class="muted">Empty: no saved list yet. Discovery ships after P2 with cache and offline state. No sample rows are shown here on purpose.</p>
</section>

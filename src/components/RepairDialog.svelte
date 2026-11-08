<script lang="ts">
  export let gtaPath: string;
  export let repairState: string;
  export let repairMsg: string;
  export let onClose: () => void;
  export let onValidate: () => void;

  function onKey(e: KeyboardEvent) {
    if (e.key === "Escape") onClose();
  }
</script>

<svelte:window onkeydown={onKey} />

<!-- svelte-ignore a11y_click_events_have_key_events -->
<div class="backdrop" onclick={onClose} role="presentation">
  <div
    class="dialog"
    role="dialog"
    aria-modal="true"
    aria-labelledby="repair-title"
    onclick={(e) => e.stopPropagation()}
    tabindex="-1"
  >
    <h2 id="repair-title">Repair game files</h2>
    <p class="muted">Read only check first. Path: {gtaPath || "not set yet"}.</p>
    <ol>
      <li>Validate gta_sa.exe hash.</li>
      <li>List missing client files.</li>
      <li>Apply fix only after you confirm.</li>
    </ol>
    {#if repairState === "loading"}
      <p role="status" class="muted">Loading: validating files.</p>
    {:else if repairState === "error"}
      <p role="alert">Error: {repairMsg || "Check failed. Nothing was changed."}</p>
    {:else if repairState === "success"}
      <p role="status">Success: {repairMsg || "Files look valid."}</p>
    {:else}
      <p role="status" class="muted">Empty: no check has run yet in this dialog.</p>
    {/if}
    <div class="row">
      <!-- svelte-ignore a11y_autofocus -->
      <button class="primary" onclick={onValidate} autofocus>Validate now</button>
      <button onclick={onClose}>Close (Esc)</button>
    </div>
  </div>
</div>

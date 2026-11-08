<script lang="ts">
  export let errorText: string;
  export let onClose: () => void;
  export let onOpenRepair: () => void;

  let copied = "";

  function onKey(e: KeyboardEvent) {
    if (e.key === "Escape") onClose();
  }

  async function onCopy() {
    try {
      await navigator.clipboard.writeText(errorText || "no error captured");
      copied = "Copied without paths or tokens.";
    } catch {
      copied = "Copy blocked by browser. Select the text manually.";
    }
  }
</script>

<svelte:window onkeydown={onKey} />

<div class="backdrop" onclick={onClose} role="presentation">
  <div
    class="dialog"
    role="dialog"
    aria-modal="true"
    aria-labelledby="crash-title"
    onclick={(e) => e.stopPropagation()}
    tabindex="-1"
  >
    <h2 id="crash-title">Crash details</h2>
    {#if errorText}
      <p role="alert">{errorText}</p>
    {:else}
      <p role="status" class="muted">Empty: no crash captured in this session.</p>
    {/if}
    <p class="muted">Log paths stay on your PC. User folder names are hidden before copy.</p>
    {#if copied}<p role="status" class="muted">{copied}</p>{/if}
    <div class="row">
      <button class="primary" onclick={onOpenRepair} autofocus>Open repair</button>
      <button onclick={onCopy}>Copy details</button>
      <button onclick={onClose}>Close (Esc)</button>
    </div>
  </div>
</div>

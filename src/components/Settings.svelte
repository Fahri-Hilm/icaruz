<script lang="ts">
  export let nickname: string;
  export let gamingMode: boolean;
  export let saveMsg: string;
  export let onSaveNickname: () => void;
  export let onOpenRepair: () => void;
  export let onOpenCrash: () => void;
</script>

<h1>Settings</h1>
<section class="card" aria-label="Profile">
  <h2>Profile</h2>
  <div class="row">
    <label for="nick">Nickname</label>
    <input id="nick" bind:value={nickname} minlength={3} maxlength={24} autocomplete="off" />
    <button class="primary" onclick={onSaveNickname}>Save nickname</button>
  </div>
  {#if saveMsg}
    <p role="status" class="muted">{saveMsg}</p>
  {:else}
    <p role="status" class="muted">Empty: no message yet. Save once to confirm storage works.</p>
  {/if}
</section>

<section class="card" aria-label="Performance">
  <h2>Performance</h2>
  <div class="row">
    <input id="gaming" type="checkbox" bind:checked={gamingMode} />
    <label for="gaming">Gaming mode (stop animation and background refresh)</label>
  </div>
  <p class="muted">When on, the UI stops transitions and skips repeat status checks while the game runs.</p>
</section>

<section class="card" aria-label="Maintenance">
  <h2>Maintenance</h2>
  <div class="row">
    <button onclick={onOpenRepair}>Open repair</button>
    <button onclick={onOpenCrash}>Crash details</button>
  </div>
</section>

<section class="card" aria-label="About">
  <h2>About</h2>
  <p>ICARUZ launcher. Developed by Fajuu.</p>
  <p class="muted">Offline first. No tracking by default.</p>
</section>

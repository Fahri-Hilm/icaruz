<script lang="ts">
  import { onMount } from "svelte";
  import Home from "./components/Home.svelte";
  import Servers from "./components/Servers.svelte";
  import Library from "./components/Library.svelte";
  import Settings from "./components/Settings.svelte";
  import RepairDialog from "./components/RepairDialog.svelte";
  import CrashDialog from "./components/CrashDialog.svelte";

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
  let webviewMsg = "";
  let saveMsg = "";
  let showRepair = false;
  let showCrash = false;
  let repairState = "empty";
  let repairMsg = "";
  let gamingMode = false;

  async function invoke(cmd: string, args?: unknown) {
    const mod = await import("@tauri-apps/api/core");
    return mod.invoke(cmd, args as Record<string, unknown>);
  }

  function applyGamingMode(on: boolean) {
    gamingMode = on;
    try {
      localStorage.setItem("icaruz-gaming-mode", on ? "1" : "0");
    } catch {
      // storage blocked, keep in memory only
    }
    document.body.classList.toggle("gaming", on);
  }

  $: if (typeof document !== "undefined") {
    document.body.classList.toggle("gaming", gamingMode);
  }

  onMount(async () => {
    try {
      gamingMode = localStorage.getItem("icaruz-gaming-mode") === "1";
    } catch {
      gamingMode = false;
    }
    document.body.classList.toggle("gaming", gamingMode);
    try {
      const data = (await invoke("get_status")) as { needs_attention?: string; nickname?: string };
      status = "Local core ready.";
      notice = data.needs_attention ?? "";
      if (data.nickname) nickname = data.nickname;
    } catch {
      status = "Tauri IPC not available in browser preview.";
      notice = "GTA San Andreas folder not set yet.";
    }
    try {
      const wv = (await invoke("webview_status")) as { installed: boolean; detail: string };
      webviewMsg = wv.installed ? "" : wv.detail;
    } catch {
      webviewMsg = "";
    }
    try {
      const savedNick = localStorage.getItem("icaruz-nick");
      if (savedNick) nickname = savedNick;
      const savedGta = localStorage.getItem("icaruz-gta-path");
      if (savedGta) gtaPath = savedGta;
    } catch {
      // preview without storage, keep defaults
    }
  });

  async function onDetect() {
    phase = "checking";
    phaseMsg = "Checking gta_sa.exe...";
    repairState = "loading";
    repairMsg = "";
    try {
      const info = (await invoke("detect_gta", { path: gtaPath })) as {
        valid: boolean;
        detail: string;
        exe_sha256: string;
      };
      gtaDetail = `${info.detail} sha256 ${String(info.exe_sha256).slice(0, 16)}...`;
      notice = info.valid ? "" : "Game version looks off. Check files before Play.";
      repairState = info.valid ? "success" : "error";
      repairMsg = info.valid ? "Check passed. No change made." : "Check found a mismatch. Nothing was changed yet.";
      phase = "idle";
      phaseMsg = "";
      try {
        localStorage.setItem("icaruz-gta-path", gtaPath);
      } catch {
        // ignore
      }
    } catch (e) {
      phase = "error";
      phaseMsg = String(e);
      repairState = "error";
      repairMsg = String(e);
    }
  }

  async function onPlay() {
    phase = "checking";
    phaseMsg = "Checking nickname and address...";
    try {
      if (!gtaPath) throw "Set GTA folder first in Library or Home.";
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

  async function onSaveNickname() {
    saveMsg = "Saving...";
    try {
      await invoke("set_nickname", { nick: nickname });
      saveMsg = "Saved to local database.";
    } catch {
      try {
        localStorage.setItem("icaruz-nick", nickname);
        saveMsg = "Preview mode: saved to browser storage only.";
      } catch {
        saveMsg = "Save failed in preview. Tauri IPC is missing.";
      }
    }
  }

  function openRepair() {
    showCrash = false;
    showRepair = true;
  }

  function openCrash() {
    showRepair = false;
    showCrash = true;
  }
</script>

<div class="shell" data-gaming={gamingMode ? "on" : "off"}>
  <nav class="side" aria-label="Primary">
    <div class="brand">ICARUZ</div>
    <button class="nav-item" aria-current={page === "home" ? "page" : undefined} onclick={() => (page = "home")}>Home</button>
    <button class="nav-item" aria-current={page === "servers" ? "page" : undefined} onclick={() => (page = "servers")}>Servers</button>
    <button class="nav-item" aria-current={page === "library" ? "page" : undefined} onclick={() => (page = "library")}>Library</button>
    <button class="nav-item" aria-current={page === "settings" ? "page" : undefined} onclick={() => (page = "settings")}>Settings</button>
    <div class="side-foot">
      <label class="game-check"><input type="checkbox" checked={gamingMode} onchange={(e) => applyGamingMode(e.currentTarget.checked)} /> Gaming mode</label>
      <div class="muted">Developed by Fajuu</div>
    </div>
  </nav>

  <main class="main">
    {#if page === "home"}
      <Home bind:gtaPath bind:nickname bind:serverInput {phase} {phaseMsg} {notice} {status} {gtaDetail} {onDetect} {onPlay} webviewMsg={webviewMsg} />
      {#if phase === "error"}
        <div class="row"><button onclick={openCrash}>View error details</button></div>
      {/if}
    {/if}

    {#if page === "servers"}
      <Servers bind:serverInput {phase} {phaseMsg} {onPlay} />
    {/if}

    {#if page === "library"}
      <Library bind:gtaPath {gtaDetail} {phase} {phaseMsg} {onDetect} />
      <div class="row"><button onclick={openRepair}>Open repair</button></div>
    {/if}

    {#if page === "settings"}
      <Settings bind:nickname bind:gamingMode {saveMsg} {onSaveNickname} onOpenRepair={openRepair} onOpenCrash={openCrash} />
    {/if}
  </main>

  {#if showRepair}
    <RepairDialog {gtaPath} {repairState} {repairMsg} onClose={() => (showRepair = false)} onValidate={onDetect} />
  {/if}
  {#if showCrash}
    <CrashDialog errorText={phase === "error" ? phaseMsg : ""} onClose={() => (showCrash = false)} onOpenRepair={openRepair} />
  {/if}
</div>

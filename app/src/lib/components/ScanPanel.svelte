<script lang="ts">
  import { onMount } from "svelte";
  import type { ConsoleSegment } from "$lib/logSegments";
  import type { ScanProgressUi } from "$lib/scanProgress";
  import { statusDisplayLabel } from "$lib/scanProgress";
  import type { ScanStatusState } from "$lib/types";
  import type { TabBanner } from "$lib/types";
  import Banner from "$lib/components/Banner.svelte";
  import ProgressBar from "$lib/components/ProgressBar.svelte";

  type Props = {
    inputDir: string;
    running: boolean;
    statusState: ScanStatusState;
    progress: ScanProgressUi;
    elapsedSeconds: number | null;
    inputDirError: string;
    banner: TabBanner;
    logSegments: ConsoleSegment[];
    logsCount: number;
    onDismissBanner?: () => void;
    onBrowse: () => void;
    onStartScan: () => void;
    onCancelScan: () => void;
    onCopyLogs: () => void;
    onClearLogs: () => void;
    onFolderSelected: (path: string) => void;
    onInputDirChange?: () => void;
  };

  let {
    inputDir = $bindable(""),
    running,
    statusState,
    progress,
    elapsedSeconds,
    inputDirError,
    banner,
    logSegments,
    logsCount,
    onDismissBanner,
    onBrowse,
    onStartScan,
    onCancelScan,
    onCopyLogs,
    onClearLogs,
    onFolderSelected,
    onInputDirChange,
  }: Props = $props();

  let consoleEl = $state<HTMLDivElement | null>(null);
  let dragOver = $state(false);

  $effect(() => {
    logSegments;
    if (consoleEl) consoleEl.scrollTop = consoleEl.scrollHeight;
  });

  onMount(() => {
    let unlistenDrag: (() => void) | undefined;

    void (async () => {
      try {
        const { getCurrentWindow } = await import("@tauri-apps/api/window");
        unlistenDrag = await getCurrentWindow().onDragDropEvent((event) => {
          if (running) return;
          const payload = event.payload;
          if (payload.type === "over") {
            dragOver = true;
          } else if (payload.type === "leave") {
            dragOver = false;
          } else if (payload.type === "drop") {
            dragOver = false;
            const path = payload.paths[0];
            if (path) onFolderSelected(path);
          }
        });
      } catch {
        // Not running inside Tauri (e.g. vite dev in browser).
      }
    })();

    return () => {
      unlistenDrag?.();
    };
  });
</script>

<section
  class="panel scan-panel"
  class:drag-over={dragOver}
  aria-label="Scan"
>
  <Banner banner={banner} onDismiss={onDismissBanner} />

  <div class="field">
    <span class="label">Input folder</span>
    <div class="path-row">
      <input
        type="text"
        class="path-input"
        class:path-input-error={inputDirError.length > 0}
        spellcheck="false"
        autocomplete="off"
        placeholder="/absolute/path/to/textures"
        disabled={running}
        bind:value={inputDir}
        oninput={() => onInputDirChange?.()}
      />
      <button
        type="button"
        class="browse-btn"
        disabled={running}
        onclick={() => onBrowse()}
      >
        Browse…
      </button>
    </div>
    {#if inputDirError}
      <span class="field-error">{inputDirError}</span>
    {/if}
    <p class="drop-hint">Drag a folder here or use Browse…</p>
  </div>

  <div class="status-row">
    <div class="status-pill">Status: {statusDisplayLabel(statusState)}</div>
    {#if running && elapsedSeconds != null}
      <div class="status-pill muted">Elapsed: {Math.floor(elapsedSeconds / 60)}:{(elapsedSeconds % 60).toString().padStart(2, "0")}</div>
    {/if}
  </div>

  <div class="progress-block">
    <div class="progress-label">{progress.phaseLabel}</div>
    <ProgressBar
      mode={progress.barMode}
      processed={progress.processed}
      total={progress.total}
    />
  </div>

  <div class="actions">
    <button type="button" class="action-btn" disabled={running} onclick={() => onStartScan()}>
      Run Scan
    </button>
    <button type="button" class="action-btn secondary" disabled={!running} onclick={() => onCancelScan()}>
      Cancel
    </button>
    <button type="button" class="action-btn tertiary" disabled={logsCount === 0} onclick={() => onCopyLogs()}>
      Copy logs
    </button>
    <button type="button" class="action-btn tertiary" disabled={logsCount === 0} onclick={() => onClearLogs()}>
      Clear logs
    </button>
  </div>

  <div class="stub-block grow">
    <div class="stub-label">Console</div>
    <div class="console" bind:this={consoleEl}>
      {#if logSegments.length === 0}
        <pre class="console-segment">No logs yet</pre>
      {:else}
        {#each logSegments as seg, i (`${seg.kind}:${i}`)}
          {#if seg.kind === "error"}
            <pre class="console-segment console-segment-error">{seg.text}</pre>
          {:else}
            <pre class="console-segment">{seg.text}</pre>
          {/if}
        {/each}
      {/if}
    </div>
  </div>
</section>

<style>
  .panel {
    display: flex;
    flex-direction: column;
    gap: 0.65rem;
    padding: 1rem;
    border-radius: 10px;
    background: #24252c;
    border: 1px solid #34353f;
    min-height: 0;
  }

  .scan-panel.drag-over {
    border-color: #5c7cfa;
    box-shadow: 0 0 0 1px rgb(92 124 250 / 35%);
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

  .label {
    font-size: 0.72rem;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: #8b8d9a;
    font-weight: 600;
  }

  .path-row {
    display: flex;
    flex-direction: row;
    align-items: stretch;
    gap: 0.4rem;
    min-width: 0;
  }

  .path-input {
    flex: 1;
    min-width: 0;
    box-sizing: border-box;
    padding: 0.55rem 0.65rem;
    border-radius: 6px;
    border: 1px solid #3f4150;
    background: #1e1f26;
    color: #f2f3f7;
    font-size: 0.85rem;
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  }

  .path-input-error {
    border-color: #8b3a52;
  }

  .browse-btn {
    flex-shrink: 0;
    padding: 0.55rem 0.75rem;
    border-radius: 6px;
    border: 1px solid #474957;
    background: #32343f;
    color: #d7d8e0;
    font-size: 0.82rem;
    line-height: 1;
    cursor: pointer;
    white-space: nowrap;
  }

  .browse-btn:hover:not(:disabled) {
    border-color: #5c7cfa;
    color: #fff;
  }

  .browse-btn:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }

  .drop-hint {
    margin: 0;
    font-size: 0.72rem;
    color: #8b8d9a;
  }

  .field-error {
    color: #f0a8a8;
    font-size: 0.72rem;
    line-height: 1.2;
  }

  .status-row {
    display: flex;
    gap: 0.5rem;
    flex-wrap: wrap;
  }

  .status-pill {
    border: 1px solid #3f4150;
    border-radius: 999px;
    padding: 0.2rem 0.55rem;
    font-size: 0.74rem;
    color: #d8dae4;
    background: #1e1f26;
  }

  .status-pill.muted {
    color: #aeb1bf;
  }

  .progress-block {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

  .progress-label {
    font-size: 0.76rem;
    color: #aeb1bf;
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: 0.45rem;
  }

  .action-btn {
    align-self: flex-start;
    padding: 0.45rem 0.85rem;
    border-radius: 6px;
    border: 1px solid #474957;
    background: #32343f;
    color: #d7d8e0;
    font-size: 0.82rem;
    font-weight: 500;
    cursor: pointer;
  }

  .action-btn:hover:not(:disabled) {
    border-color: #5c7cfa;
    color: #fff;
  }

  .action-btn:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }

  .action-btn.secondary,
  .action-btn.tertiary {
    border-color: #3f4150;
    background: #2a2b33;
    color: #a1a3b0;
  }

  .stub-block {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
  }

  .stub-block.grow {
    flex: 1;
    min-height: 8rem;
  }

  .stub-label {
    font-size: 0.72rem;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: #8b8d9a;
    font-weight: 600;
  }

  .console {
    flex: 1;
    margin: 0;
    padding: 0.55rem 0.65rem;
    border-radius: 6px;
    border: 1px solid #3f4150;
    background: #1a1b21;
    font-size: 0.78rem;
    line-height: 1.45;
    color: #9ea1ad;
    overflow: auto;
    min-height: 6rem;
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

  .console-segment {
    margin: 0;
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    font-size: inherit;
    line-height: inherit;
    white-space: pre-wrap;
    word-break: break-word;
    color: inherit;
  }

  .console-segment-error {
    color: #de3576;
  }
</style>

<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { open, save } from "@tauri-apps/plugin-dialog";
  import { segmentConsoleLines } from "$lib/logSegments";
  import GroupDetailModal from "$lib/components/GroupDetailModal.svelte";
  import ResultsPanel from "$lib/components/ResultsPanel.svelte";
  import ScanPanel from "$lib/components/ScanPanel.svelte";
  import SettingsModal from "$lib/components/SettingsModal.svelte";
  import {
    filterVisibleGroups,
    groupedPreview,
    maxGroupsPerPage,
    reasonKindOrder,
    sortGroups,
  } from "$lib/groupUtils";
  import { idleProgress, progressFromPhase } from "$lib/scanProgress";
  import { initialSettings, validateSettings } from "$lib/settingsValidation";
  import type {
    AppSettings,
    GroupReasonKind,
    ScanResult,
    ScanStatusState,
    SettingsValidationErrors,
    SortOption,
    TabBanner,
    UiStateResponse,
  } from "$lib/types";

  type ScanStatusResponse = {
    state: ScanStatusState;
    activeScanId: number | null;
  };

  type LastResultResponse = {
    hasResult: boolean;
    result: ScanResult | null;
  };

  type StartScanResponse = {
    scanId: number;
  };

  type ScanLogPayload = {
    scanId: number;
    line: string;
    level: "info" | "warn" | "error";
  };

  type ScanProgressPayload = {
    scanId: number;
    phase: string;
    processed?: number;
    total?: number;
    vertices?: number;
    groups?: number;
  };

  type ScanFinishedPayload = {
    scanId: number;
    status: "completed" | "failed" | "cancelled";
    message?: string;
  };

  type ExportResponse = {
    path: string;
  };

  const maxLogLines = 1500;
  const bannerAutoDismissMs = 4500;

  let activeTab = $state<"scan" | "results">("scan");
  let showSettings = $state(false);
  let running = $state(false);
  let activeScanId = $state<number | null>(null);
  let statusState = $state<ScanStatusState>("idle");
  let inputDir = $state("");
  let inputDirError = $state("");
  let logs = $state<string[]>([]);
  let logSegments = $derived(segmentConsoleLines(logs));
  let scanProgress = $state(idleProgress());
  let scanStartMs = $state<number | null>(null);
  let elapsedSeconds = $state<number | null>(null);
  let result = $state<ScanResult | null>(null);
  let saveMessage = $state("");
  let loadError = $state("");
  let settingsErrors = $state<SettingsValidationErrors>({});
  let selectedReasonKinds = $state<GroupReasonKind[]>([]);
  let sortOption = $state<SortOption>("count_desc");
  let resultsPage = $state(1);
  let selectedGroupId = $state<number | null>(null);
  let scanBanner = $state<TabBanner>(null);
  let resultsBanner = $state<TabBanner>(null);

  let settings = $state<AppSettings>(structuredClone(initialSettings));
  let settingsDraft = $state<AppSettings>(structuredClone(initialSettings));

  let elapsedTimer: ReturnType<typeof setInterval> | undefined;
  let bannerTimer: ReturnType<typeof setTimeout> | undefined;

  function resetMessages() {
    saveMessage = "";
    loadError = "";
    settingsErrors = {};
  }

  function cloneSettings(next: AppSettings): AppSettings {
    return structuredClone(next);
  }

  function showTabBanner(tab: "scan" | "results", kind: "success" | "error", message: string) {
    const banner = { kind, message };
    if (tab === "scan") scanBanner = banner;
    else resultsBanner = banner;

    if (bannerTimer) clearTimeout(bannerTimer);
    if (kind === "success") {
      bannerTimer = setTimeout(() => {
        if (tab === "scan") scanBanner = null;
        else resultsBanner = null;
      }, bannerAutoDismissMs);
    }
  }

  function appendLog(line: string) {
    logs = [...logs, line].slice(-maxLogLines);
  }

  function clearLogs() {
    logs = [];
  }

  async function copyLogs() {
    try {
      await navigator.clipboard.writeText(logs.join("\n"));
      showTabBanner("scan", "success", "Logs copied to clipboard.");
    } catch {
      showTabBanner("scan", "error", "Could not copy logs to clipboard.");
    }
  }

  async function chooseInputDirectory() {
    const selected = await open({ directory: true, multiple: false });
    if (typeof selected === "string") {
      inputDir = selected;
      inputDirError = "";
      try {
        await invoke("save_last_input_dir", { inputDir: selected });
      } catch {
        // Non-fatal; folder still selected for this session.
      }
    }
  }

  function handleFolderSelected(path: string) {
    inputDir = path;
    inputDirError = "";
    void invoke("save_last_input_dir", { inputDir: path }).catch(() => undefined);
  }

  async function loadUiState() {
    try {
      const ui = await invoke<UiStateResponse>("load_ui_state");
      if (ui.lastInputDir) inputDir = ui.lastInputDir;
    } catch {
      // Non-fatal.
    }
  }

  async function loadSettings() {
    resetMessages();
    try {
      settings = await invoke<AppSettings>("load_settings");
      settingsDraft = cloneSettings(settings);
    } catch (e) {
      loadError = `Failed to load settings: ${String(e)}`;
    }
  }

  function openSettings() {
    resetMessages();
    settingsDraft = cloneSettings(settings);
    showSettings = true;
  }

  function cancelSettings() {
    resetMessages();
    settingsDraft = cloneSettings(settings);
    showSettings = false;
  }

  async function saveSettings() {
    resetMessages();
    const errors = validateSettings(settingsDraft);
    settingsErrors = errors;
    if (Object.keys(errors).length > 0) {
      loadError = "Fix validation errors before saving.";
      return;
    }
    try {
      await invoke("save_settings", { settings: settingsDraft });
      settings = cloneSettings(settingsDraft);
      showSettings = false;
      showTabBanner(activeTab, "success", "Settings saved.");
    } catch (e) {
      loadError = `Failed to save settings: ${String(e)}`;
    }
  }

  async function refreshStatus() {
    try {
      const status = await invoke<ScanStatusResponse>("scan_status");
      activeScanId = status.activeScanId;
      running = status.state === "running";
      statusState = status.state;
    } catch {
      running = false;
      statusState = "unknown";
      activeScanId = null;
    }
  }

  async function refreshResult() {
    try {
      const data = await invoke<LastResultResponse>("get_last_result");
      result = data.result;
    } catch {
      result = null;
    }
  }

  function startElapsedTimer() {
    scanStartMs = Date.now();
    elapsedSeconds = 0;
    if (elapsedTimer) clearInterval(elapsedTimer);
    elapsedTimer = setInterval(() => {
      if (scanStartMs == null) return;
      elapsedSeconds = Math.floor((Date.now() - scanStartMs) / 1000);
    }, 1000);
  }

  function stopElapsedTimer() {
    if (elapsedTimer) {
      clearInterval(elapsedTimer);
      elapsedTimer = undefined;
    }
  }

  async function startScan() {
    inputDirError = "";
    scanBanner = null;

    if (!inputDir.trim()) {
      inputDirError = "Choose an input folder before running a scan.";
      return;
    }

    clearLogs();
    scanProgress = progressFromPhase("starting");
    try {
      const response = await invoke<StartScanResponse>("start_scan", {
        request: { inputDir: inputDir.trim() },
      });
      activeScanId = response.scanId;
      running = true;
      statusState = "running";
      startElapsedTimer();
    } catch (e) {
      appendLog(`Scan start failed: ${String(e)}`);
      scanProgress = progressFromPhase("failed");
      showTabBanner("scan", "error", `Scan could not start: ${String(e)}`);
    }
  }

  async function cancelScan() {
    try {
      await invoke("cancel_scan");
      appendLog("Cancellation requested");
    } catch (e) {
      appendLog(`Cancel failed: ${String(e)}`);
    }
  }

  async function exportJson() {
    if (!result) return;
    resultsBanner = null;
    const target = await save({
      title: "Export result JSON",
      defaultPath: "result.json",
      filters: [{ name: "JSON", extensions: ["json"] }],
    });
    if (!target || Array.isArray(target)) return;
    try {
      const out = await invoke<ExportResponse>("export_last_result_json", {
        request: { targetPath: target },
      });
      showTabBanner("results", "success", `Exported JSON to ${out.path}`);
    } catch (e) {
      showTabBanner("results", "error", `Export failed: ${String(e)}`);
    }
  }

  function hasReasonFilter(kind: GroupReasonKind) {
    return selectedReasonKinds.includes(kind);
  }

  function toggleReasonFilter(kind: GroupReasonKind) {
    if (hasReasonFilter(kind)) {
      selectedReasonKinds = selectedReasonKinds.filter((item) => item !== kind);
      return;
    }
    selectedReasonKinds = [...selectedReasonKinds, kind];
  }

  function goToPage(nextPage: number) {
    resultsPage = Math.max(1, Math.min(totalPages, nextPage));
  }

  function openGroupDetails(groupId: number) {
    selectedGroupId = groupId;
  }

  function closeGroupDetails() {
    selectedGroupId = null;
  }

  let sourceGroups = $derived(result?.groups ?? []);
  let visibleGroups = $derived(filterVisibleGroups(sourceGroups, settings));
  let availableReasonKinds = $derived.by(() => {
    const seen = new Set<GroupReasonKind>();
    for (const group of visibleGroups) {
      seen.add(group.reason_kind);
    }
    return reasonKindOrder.filter((kind) => seen.has(kind));
  });
  let filteredSortedGroups = $derived.by(() => {
    const activeFilters = new Set(selectedReasonKinds);
    const filtered =
      activeFilters.size === 0
        ? visibleGroups
        : visibleGroups.filter((group) => activeFilters.has(group.reason_kind));
    return sortGroups(filtered, sortOption);
  });
  let totalPages = $derived(Math.max(1, Math.ceil(filteredSortedGroups.length / maxGroupsPerPage)));
  let pageStartIndex = $derived((resultsPage - 1) * maxGroupsPerPage);
  let pagedGroups = $derived(
    filteredSortedGroups.slice(pageStartIndex, pageStartIndex + maxGroupsPerPage),
  );
  let previewGroups = $derived(groupedPreview(pagedGroups));
  let selectedGroup = $derived.by(() => {
    if (selectedGroupId == null) return null;
    return filteredSortedGroups.find((group) => group.id === selectedGroupId) ?? null;
  });

  $effect(() => {
    if (!result) {
      resultsPage = 1;
      return;
    }
    result.groups;
    settings.hide_single_image_groups;
    sortOption;
    selectedReasonKinds.join(",");
    resultsPage = 1;
  });

  $effect(() => {
    if (resultsPage > totalPages) {
      resultsPage = totalPages;
      return;
    }
    if (resultsPage < 1) {
      resultsPage = 1;
    }
  });

  $effect(() => {
    const allowed = new Set(availableReasonKinds);
    const next = selectedReasonKinds.filter((kind) => allowed.has(kind));
    if (next.length !== selectedReasonKinds.length) {
      selectedReasonKinds = next;
    }
  });

  $effect(() => {
    if (selectedGroupId == null) return;
    const exists = filteredSortedGroups.some((group) => group.id === selectedGroupId);
    if (!exists) selectedGroupId = null;
  });

  $effect(() => {
    if (!running) stopElapsedTimer();
  });

  onMount(() => {
    let unlistenLog: UnlistenFn | undefined;
    let unlistenProgress: UnlistenFn | undefined;
    let unlistenFinished: UnlistenFn | undefined;

    void loadSettings();
    void loadUiState();
    void refreshStatus();
    void refreshResult();

    void listen<ScanLogPayload>("scan-log", (evt) => {
      const p = evt.payload;
      if (activeScanId !== null && p.scanId !== activeScanId) return;
      appendLog(p.line);
    }).then((fn) => (unlistenLog = fn));

    void listen<ScanProgressPayload>("scan-progress", (evt) => {
      const p = evt.payload;
      if (activeScanId !== null && p.scanId !== activeScanId) return;
      scanProgress = progressFromPhase(p.phase, p.processed, p.total, p.groups);
    }).then((fn) => (unlistenProgress = fn));

    void listen<ScanFinishedPayload>("scan-finished", (evt) => {
      const p = evt.payload;
      if (activeScanId !== null && p.scanId !== activeScanId) return;
      running = false;
      statusState = p.status;
      stopElapsedTimer();

      if (p.status === "completed") {
        scanProgress = progressFromPhase("completed");
        void refreshResult().then(() => {
          activeTab = "results";
        });
      } else if (p.status === "cancelled") {
        scanProgress = progressFromPhase("cancelled");
      } else {
        scanProgress = progressFromPhase("failed");
        const message = p.message ?? "The scan failed. See the console for details.";
        showTabBanner("scan", "error", message);
        if (p.message) appendLog(`Failure: ${p.message}`);
        activeTab = "scan";
      }
    }).then((fn) => (unlistenFinished = fn));

    return () => {
      unlistenLog?.();
      unlistenProgress?.();
      unlistenFinished?.();
      stopElapsedTimer();
      if (bannerTimer) clearTimeout(bannerTimer);
    };
  });
</script>

<div class="shell">
  <div class="app">
    <header class="header">
      <div class="header-text">
        <h1>Similar Textures Finder</h1>
        <div class="tabs" role="tablist" aria-label="Main sections">
          <button
            type="button"
            role="tab"
            class="tab"
            class:tab-active={activeTab === "scan"}
            aria-selected={activeTab === "scan"}
            onclick={() => (activeTab = "scan")}
          >
            Scan
          </button>
          <button
            type="button"
            role="tab"
            class="tab"
            class:tab-active={activeTab === "results"}
            aria-selected={activeTab === "results"}
            onclick={() => (activeTab = "results")}
          >
            Results
          </button>
        </div>
      </div>
      <button
        type="button"
        class="settings-btn"
        title="Settings"
        aria-label="Open settings"
        onclick={() => openSettings()}
      >
        <svg
          width="18"
          height="18"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <circle cx="12" cy="12" r="3" />
          <path
            d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"
          />
        </svg>
      </button>
    </header>

    {#if activeTab === "scan"}
      <ScanPanel
        bind:inputDir
        {running}
        statusState={statusState}
        progress={scanProgress}
        elapsedSeconds={running ? elapsedSeconds : null}
        {inputDirError}
        banner={scanBanner}
        {logSegments}
        logsCount={logs.length}
        onDismissBanner={() => (scanBanner = null)}
        onBrowse={() => void chooseInputDirectory()}
        onStartScan={() => void startScan()}
        onCancelScan={() => void cancelScan()}
        onCopyLogs={() => void copyLogs()}
        onClearLogs={clearLogs}
        onFolderSelected={handleFolderSelected}
        onInputDirChange={() => (inputDirError = "")}
      />
    {:else}
      <ResultsPanel
        hasResult={result != null}
        filteredGroupCount={filteredSortedGroups.length}
        banner={resultsBanner}
        bind:sortOption
        {availableReasonKinds}
        {selectedReasonKinds}
        {resultsPage}
        {totalPages}
        {previewGroups}
        onDismissBanner={() => (resultsBanner = null)}
        onExport={() => void exportJson()}
        onToggleReasonFilter={toggleReasonFilter}
        onGoToPage={goToPage}
        onOpenGroup={openGroupDetails}
      />
    {/if}
  </div>

  {#if showSettings}
    <SettingsModal
      bind:settingsDraft
      {settingsErrors}
      {saveMessage}
      {loadError}
      onCancel={cancelSettings}
      onSave={() => void saveSettings()}
    />
  {/if}

  {#if selectedGroup}
    <GroupDetailModal group={selectedGroup} onClose={closeGroupDetails} />
  {/if}
</div>

<style>
  :global(html),
  :global(body) {
    margin: 0;
    height: 100%;
    overflow: hidden;
    font-family:
      system-ui,
      -apple-system,
      Segoe UI,
      Roboto,
      Helvetica,
      Arial,
      sans-serif;
    background: #1a1b1e;
    color: #e9e9ef;
  }

  :global(body) {
    display: flex;
    flex-direction: column;
    min-height: 0;
  }

  .shell {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
    min-width: 0;
    overflow: hidden;
  }

  .app {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
    overflow-y: auto;
    overflow-x: hidden;
    box-sizing: border-box;
    padding: 1rem 1.25rem 0.75rem;
    gap: 1rem;
  }

  .header {
    display: flex;
    flex-direction: row;
    align-items: flex-start;
    justify-content: space-between;
    gap: 0.5rem;
  }

  .header h1 {
    margin: 0 0 0.35rem;
    font-size: 1.35rem;
    font-weight: 650;
    letter-spacing: -0.02em;
  }

  .header-text {
    flex: 1;
    min-width: 0;
  }

  .tabs {
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
    padding: 0.2rem;
    border: 1px solid #3f4150;
    border-radius: 8px;
    background: #1e1f26;
  }

  .tab {
    border: 1px solid transparent;
    border-radius: 6px;
    background: transparent;
    color: #a1a3b0;
    font-size: 0.82rem;
    padding: 0.3rem 0.65rem;
    cursor: pointer;
  }

  .tab:hover {
    color: #fff;
  }

  .tab.tab-active {
    background: #32343f;
    color: #f2f3f7;
    border-color: #474957;
  }

  .settings-btn {
    flex-shrink: 0;
    margin-top: 0.15rem;
    padding: 0.45rem;
    border-radius: 6px;
    border: 1px solid #474957;
    background: #32343f;
    color: #d7d8e0;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    line-height: 1;
  }

  .settings-btn:hover {
    border-color: #5c7cfa;
    color: #fff;
  }
</style>

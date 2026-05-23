<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { open, save } from "@tauri-apps/plugin-dialog";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import { segmentConsoleLines } from "$lib/logSegments";

  type Weights = {
    phash: number;
    ssim: number;
    histogram: number;
    orb: number;
  };

  type AppSettings = {
    enable_phash: boolean;
    enable_ssim: boolean;
    enable_histogram: boolean;
    enable_orb: boolean;
    enable_alpha_crop: boolean;
    enable_rotations: boolean;
    enable_flip: boolean;
    threshold: number;
    weights: Weights;
    hash_algorithm: string;
    phash_max_distance: number;
    ssim_threshold: number;
    resize_size: number;
    hist_bins: number;
    hist_method: string;
    alpha_threshold: number;
    orb_max_features: number | null;
    orb_match_threshold: number | null;
    max_decode_dimension_px: number | null;
    hide_single_image_groups: boolean;
  };

  type SettingsValidationErrors = {
    threshold?: string;
    phashWeight?: string;
    ssimWeight?: string;
    histogramWeight?: string;
    phashMaxDistance?: string;
    ssimThreshold?: string;
    resizeSize?: string;
    histBins?: string;
    alphaThreshold?: string;
    maxDecodeDimension?: string;
  };

  type GroupReasonKind = "singleton" | "hash" | "composite" | "mixed";

  type PairReasonType = "hash" | "composite";

  type MetricEvidence = {
    score: number;
    raw: number;
    valid: boolean;
  };

  type GroupPairReason = {
    left: string;
    right: string;
    type: PairReasonType;
    composite_score?: number;
    phash?: MetricEvidence;
    ssim?: MetricEvidence;
    histogram?: MetricEvidence;
  };

  type ScanGroup = {
    id: number;
    name?: string;
    score: number | null;
    images: string[];
    reason_kind: GroupReasonKind;
    reasons: GroupPairReason[];
  };

  type ScanResult = {
    groups: ScanGroup[];
  };

  type ScanStatusResponse = {
    state: "idle" | "running" | "completed" | "failed" | "cancelled";
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
  const maxResultThumbs = 10;
  const maxGroupsPerPage = 15;
  const reasonKindOrder: GroupReasonKind[] = ["singleton", "hash", "composite", "mixed"];

  type SortOption = "count_desc" | "count_asc" | "name_asc" | "name_desc";

  let activeTab = $state<"scan" | "results">("scan");
  let showSettings = $state(false);
  let running = $state(false);
  let activeScanId = $state<number | null>(null);
  let statusText = $state("Idle");
  let inputDir = $state("");
  let logs = $state<string[]>([]);
  let logSegments = $derived(segmentConsoleLines(logs));
  let progressText = $state("No active scan");
  let result = $state<ScanResult | null>(null);
  let saveMessage = $state("");
  let loadError = $state("");
  let settingsErrors = $state<SettingsValidationErrors>({});
  let selectedReasonKinds = $state<GroupReasonKind[]>([]);
  let sortOption = $state<SortOption>("count_desc");
  let resultsPage = $state(1);
  let selectedGroupId = $state<number | null>(null);

  const initialSettings: AppSettings = {
    enable_phash: true,
    enable_ssim: true,
    enable_histogram: true,
    enable_orb: false,
    enable_alpha_crop: false,
    enable_rotations: false,
    enable_flip: false,
    threshold: 0.85,
    weights: { phash: 0.35, ssim: 0.45, histogram: 0.2, orb: 0 },
    hash_algorithm: "sha256",
    phash_max_distance: 10,
    ssim_threshold: 0.9,
    resize_size: 256,
    hist_bins: 512,
    hist_method: "correlation",
    alpha_threshold: 0.05,
    orb_max_features: null,
    orb_match_threshold: null,
    max_decode_dimension_px: null,
    hide_single_image_groups: true,
  };
  let settings = $state<AppSettings>(structuredClone(initialSettings));
  let settingsDraft = $state<AppSettings>(structuredClone(initialSettings));

  function resetMessages() {
    saveMessage = "";
    loadError = "";
    settingsErrors = {};
  }

  function cloneSettings(next: AppSettings): AppSettings {
    return structuredClone(next);
  }

  function validateSettings(next: AppSettings): SettingsValidationErrors {
    const errors: SettingsValidationErrors = {};
    if (next.threshold < 0 || next.threshold > 1) {
      errors.threshold = "Threshold must be between 0 and 1.";
    }
    if (next.weights.phash < 0) {
      errors.phashWeight = "pHash weight cannot be negative.";
    }
    if (next.weights.ssim < 0) {
      errors.ssimWeight = "SSIM weight cannot be negative.";
    }
    if (next.weights.histogram < 0) {
      errors.histogramWeight = "Histogram weight cannot be negative.";
    }
    if (next.phash_max_distance < 0) {
      errors.phashMaxDistance = "pHash max distance cannot be negative.";
    }
    if (next.ssim_threshold < 0 || next.ssim_threshold > 1) {
      errors.ssimThreshold = "SSIM threshold must be between 0 and 1.";
    }
    if (next.resize_size < 1) {
      errors.resizeSize = "Resize size must be at least 1.";
    }
    if (next.hist_bins < 2) {
      errors.histBins = "Histogram bins must be at least 2.";
    }
    if (next.alpha_threshold < 0 || next.alpha_threshold > 1) {
      errors.alphaThreshold = "Alpha threshold must be between 0 and 1.";
    }
    if (next.max_decode_dimension_px != null && next.max_decode_dimension_px < 1) {
      errors.maxDecodeDimension = "Max decode dimension must be at least 1.";
    }
    if (!next.enable_phash && !next.enable_ssim && !next.enable_histogram) {
      errors.threshold = "Enable at least one metric (pHash, SSIM, or histogram).";
    }
    return errors;
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
    } catch {
      appendLog("Copy logs failed");
    }
  }

  async function chooseInputDirectory() {
    const selected = await open({ directory: true, multiple: false });
    if (typeof selected === "string") {
      inputDir = selected;
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
      saveMessage = "Settings saved";
      showSettings = false;
    } catch (e) {
      loadError = `Failed to save settings: ${String(e)}`;
    }
  }

  async function refreshStatus() {
    try {
      const status = await invoke<ScanStatusResponse>("scan_status");
      activeScanId = status.activeScanId;
      running = status.state === "running";
      statusText = status.state;
    } catch {
      running = false;
      statusText = "unknown";
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

  async function startScan() {
    if (!inputDir.trim()) {
      appendLog("Please choose input folder before scan");
      return;
    }
    clearLogs();
    progressText = "Starting scan...";
    try {
      const response = await invoke<StartScanResponse>("start_scan", {
        request: { inputDir: inputDir.trim() },
      });
      activeScanId = response.scanId;
      running = true;
      statusText = "running";
    } catch (e) {
      appendLog(`Scan start failed: ${String(e)}`);
      progressText = "Scan start failed";
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
      appendLog(`Exported JSON: ${out.path}`);
    } catch (e) {
      appendLog(`Export failed: ${String(e)}`);
    }
  }

  function toBaseName(path: string) {
    return path.split(/[\\/]/).at(-1) ?? path;
  }

  function groupTitle(group: Pick<ScanGroup, "id" | "name">) {
    const explicit = group.name?.trim();
    return explicit && explicit.length > 0 ? explicit : `Group #${group.id}`;
  }

  function reasonKindLabel(kind: GroupReasonKind) {
    switch (kind) {
      case "singleton":
        return "Singleton";
      case "hash":
        return "Hash";
      case "composite":
        return "Composite";
      case "mixed":
        return "Mixed";
      default:
        return kind;
    }
  }

  function pairReasonTypeLabel(reasonType: PairReasonType) {
    switch (reasonType) {
      case "hash":
        return "Hash";
      case "composite":
        return "Composite";
      default:
        return reasonType;
    }
  }

  function formatMetricValue(value: number) {
    return Number.isFinite(value) ? value.toFixed(3) : String(value);
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

  function compareGroupNames(a: ScanGroup, b: ScanGroup) {
    const nameA = a.name?.trim();
    const nameB = b.name?.trim();
    if (nameA && nameB) {
      const byName = nameA.localeCompare(nameB, undefined, { sensitivity: "base" });
      return byName !== 0 ? byName : a.id - b.id;
    }
    if (nameA && !nameB) return -1;
    if (!nameA && nameB) return 1;
    return a.id - b.id;
  }

  function groupedPreview(groups: ScanGroup[]) {
    return groups.map((g) => ({
      id: g.id,
      title: groupTitle(g),
      scoreLabel: g.score == null ? "-" : g.score.toFixed(3),
      images: g.images.slice(0, maxResultThumbs),
      hiddenCount: Math.max(0, g.images.length - maxResultThumbs),
      count: g.images.length,
      reasonKind: g.reason_kind,
    }));
  }

  let sourceGroups = $derived(result?.groups ?? []);
  let visibleGroups = $derived(
    sourceGroups.filter((group) => !settings.hide_single_image_groups || group.images.length > 1),
  );
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

    const sorted = [...filtered];
    switch (sortOption) {
      case "count_asc":
        sorted.sort((a, b) => a.images.length - b.images.length || a.id - b.id);
        break;
      case "name_asc":
        sorted.sort(compareGroupNames);
        break;
      case "name_desc":
        sorted.sort((a, b) => compareGroupNames(b, a));
        break;
      case "count_desc":
      default:
        sorted.sort((a, b) => b.images.length - a.images.length || a.id - b.id);
        break;
    }
    return sorted;
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

  onMount(() => {
    let unlistenLog: UnlistenFn | undefined;
    let unlistenProgress: UnlistenFn | undefined;
    let unlistenFinished: UnlistenFn | undefined;

    void loadSettings();
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
      if (p.phase === "ingest_progress" && p.processed != null && p.total != null) {
        progressText = `Ingesting ${p.processed}/${p.total}`;
        return;
      }
      if (p.phase === "clustering_done" && p.groups != null) {
        progressText = `Clustering done: ${p.groups} groups`;
        return;
      }
      progressText = p.phase;
    }).then((fn) => (unlistenProgress = fn));

    void listen<ScanFinishedPayload>("scan-finished", (evt) => {
      const p = evt.payload;
      if (activeScanId !== null && p.scanId !== activeScanId) return;
      running = false;
      statusText = p.status;
      if (p.status === "completed") {
        progressText = "Scan completed";
        void refreshResult().then(() => {
          activeTab = "results";
        });
      } else if (p.status === "cancelled") {
        progressText = "Scan cancelled";
      } else {
        progressText = "Scan failed";
        if (p.message) appendLog(`Failure: ${p.message}`);
      }
    }).then((fn) => (unlistenFinished = fn));

    return () => {
      unlistenLog?.();
      unlistenProgress?.();
      unlistenFinished?.();
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
      <section class="panel" aria-label="Scan">
        <div class="field">
          <span class="label">Input folder</span>
          <div class="path-row">
            <input
              type="text"
              class="path-input"
              spellcheck="false"
              autocomplete="off"
              placeholder="/absolute/path/to/textures"
              bind:value={inputDir}
            />
            <button
              type="button"
              class="browse-btn"
              title="Choose folder"
              aria-label="Choose input folder"
              disabled={running}
              onclick={() => void chooseInputDirectory()}
            >
              ...
            </button>
          </div>
        </div>

        <div class="status-row">
          <div class="status-pill">Status: {statusText}</div>
          <div class="status-pill muted">{progressText}</div>
        </div>

        <div class="actions">
          <button
            type="button"
            class="action-btn"
            disabled={running}
            onclick={() => void startScan()}
          >
            Run Scan
          </button>
          <button
            type="button"
            class="action-btn secondary"
            disabled={!running}
            onclick={() => void cancelScan()}
          >
            Cancel
          </button>
          <button
            type="button"
            class="action-btn tertiary"
            disabled={logs.length === 0}
            onclick={() => void copyLogs()}
          >
            Copy logs
          </button>
          <button
            type="button"
            class="action-btn tertiary"
            disabled={logs.length === 0}
            onclick={() => clearLogs()}
          >
            Clear logs
          </button>
        </div>

        <div class="stub-block grow">
          <div class="stub-label">Console</div>
          <div class="console">
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
    {:else}
      <section class="panel" aria-label="Results">
        <div class="results-header">
          <div>
            <h2>Scan Results</h2>
            <p class="stub">
              {#if result}
                {filteredSortedGroups.length} groups loaded
              {:else}
                No in-memory scan result yet
              {/if}
            </p>
          </div>
          <button
            type="button"
            class="action-btn"
            disabled={!result}
            onclick={() => void exportJson()}
          >
            Export JSON
          </button>
        </div>

        {#if !result}
          <div class="empty-state">Run a scan in the Scan tab to populate results.</div>
        {:else}
          <div class="results-toolbar">
            <div class="toolbar-row">
              <label class="toolbar-field">
                <span>Sort by</span>
                <select bind:value={sortOption}>
                  <option value="count_desc">Count (high to low)</option>
                  <option value="count_asc">Count (low to high)</option>
                  <option value="name_asc">Name (A to Z)</option>
                  <option value="name_desc">Name (Z to A)</option>
                </select>
              </label>
            </div>

            <div class="toolbar-row">
              <span class="toolbar-label">Reason filter</span>
              <div class="reason-chips">
                {#if availableReasonKinds.length === 0}
                  <span class="stub">No reason tags available</span>
                {:else}
                  {#each availableReasonKinds as kind (kind)}
                    <button
                      type="button"
                      class="chip"
                      class:chip-active={hasReasonFilter(kind)}
                      onclick={() => toggleReasonFilter(kind)}
                    >
                      {reasonKindLabel(kind)}
                    </button>
                  {/each}
                {/if}
              </div>
            </div>

            <div class="pagination-row">
              <div class="pagination-meta">Page {resultsPage} / {totalPages}</div>
              <div class="pagination-controls">
                <button
                  type="button"
                  class="action-btn tertiary"
                  disabled={resultsPage <= 1}
                  onclick={() => goToPage(1)}
                >
                  First
                </button>
                <button
                  type="button"
                  class="action-btn tertiary"
                  disabled={resultsPage <= 1}
                  onclick={() => goToPage(resultsPage - 1)}
                >
                  Prev
                </button>
                <button
                  type="button"
                  class="action-btn tertiary"
                  disabled={resultsPage >= totalPages}
                  onclick={() => goToPage(resultsPage + 1)}
                >
                  Next
                </button>
                <button
                  type="button"
                  class="action-btn tertiary"
                  disabled={resultsPage >= totalPages}
                  onclick={() => goToPage(totalPages)}
                >
                  Last
                </button>
              </div>
            </div>
          </div>

          <div class="results-grid">
            {#each previewGroups as group (group.id)}
              <article class="result-card result-card-clickable">
                <button
                  type="button"
                  class="result-card-button"
                  aria-label={`Open details for ${group.title}`}
                  onclick={() => openGroupDetails(group.id)}
                >
                <header class="result-card-header">
                  <div class="result-title">{group.title}</div>
                  <div class="reason-badge">{reasonKindLabel(group.reasonKind)}</div>
                  <div class="result-meta">score: {group.scoreLabel}</div>
                  <div class="result-meta">images: {group.count}</div>
                </header>
                <div class="thumb-grid">
                  {#each group.images as imagePath (imagePath)}
                    <figure class="thumb-item" title={imagePath}>
                      <img src={convertFileSrc(imagePath)} alt={imagePath} loading="lazy" />
                      <figcaption>{toBaseName(imagePath)}</figcaption>
                    </figure>
                  {/each}
                </div>
                {#if group.hiddenCount > 0}
                  <div class="stub">+ {group.hiddenCount} more image(s)</div>
                {/if}
                </button>
              </article>
            {/each}
          </div>
        {/if}
      </section>
    {/if}
  </div>

  {#if showSettings}
    <div
      class="overlay"
      role="button"
      tabindex="0"
      aria-label="Close settings"
      onclick={(e) => {
        if (e.target === e.currentTarget) showSettings = false;
      }}
      onkeydown={(e) => {
        if (e.key === "Escape" || e.key === "Enter" || e.key === " ") cancelSettings();
      }}
    >
      <div class="modal">
        <div class="modal-header">
          <h2>Settings</h2>
          <button
            type="button"
            class="modal-close-btn"
            aria-label="Close settings"
            onclick={() => cancelSettings()}
          >
            <svg
              width="16"
              height="16"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <line x1="18" y1="6" x2="6" y2="18" />
              <line x1="6" y1="6" x2="18" y2="18" />
            </svg>
          </button>
        </div>

        <div class="modal-body">
          <div class="settings-grid">
            <label>
              <input type="checkbox" bind:checked={settingsDraft.enable_phash} />
              Enable pHash
            </label>
            <label>
              <input type="checkbox" bind:checked={settingsDraft.enable_ssim} />
              Enable SSIM
            </label>
            <label>
              <input type="checkbox" bind:checked={settingsDraft.enable_histogram} />
              Enable histogram
            </label>
            <label>
              <input type="checkbox" bind:checked={settingsDraft.enable_alpha_crop} />
              Enable alpha crop
            </label>
            <label>
              <input type="checkbox" bind:checked={settingsDraft.enable_rotations} />
              Enable rotations
            </label>
            <label><input type="checkbox" bind:checked={settingsDraft.enable_flip} /> Enable flip</label>
            <label>
              <input type="checkbox" bind:checked={settingsDraft.hide_single_image_groups} />
              Hide groups with 1 image
            </label>

            <label>
              Threshold
              <input
                type="number"
                step="0.01"
                min="0"
                max="1"
                bind:value={settingsDraft.threshold}
              />
              {#if settingsErrors.threshold}<span class="field-error">{settingsErrors.threshold}</span>{/if}
            </label>
            <label>
              pHash weight
              <input type="number" step="0.01" min="0" bind:value={settingsDraft.weights.phash} />
              {#if settingsErrors.phashWeight}
                <span class="field-error">{settingsErrors.phashWeight}</span>
              {/if}
            </label>
            <label>
              SSIM weight
              <input type="number" step="0.01" min="0" bind:value={settingsDraft.weights.ssim} />
              {#if settingsErrors.ssimWeight}
                <span class="field-error">{settingsErrors.ssimWeight}</span>
              {/if}
            </label>
            <label>
              Histogram weight
              <input
                type="number"
                step="0.01"
                min="0"
                bind:value={settingsDraft.weights.histogram}
              />
              {#if settingsErrors.histogramWeight}
                <span class="field-error">{settingsErrors.histogramWeight}</span>
              {/if}
            </label>
            <label>
              Hash algorithm
              <select bind:value={settingsDraft.hash_algorithm}>
                <option value="sha256">sha256</option>
                <option value="sha1">sha1</option>
                <option value="md5">md5</option>
              </select>
            </label>
            <label>
              pHash max distance
              <input type="number" min="0" bind:value={settingsDraft.phash_max_distance} />
              {#if settingsErrors.phashMaxDistance}
                <span class="field-error">{settingsErrors.phashMaxDistance}</span>
              {/if}
            </label>
            <label>
              SSIM threshold
              <input
                type="number"
                step="0.01"
                min="0"
                max="1"
                bind:value={settingsDraft.ssim_threshold}
              />
              {#if settingsErrors.ssimThreshold}
                <span class="field-error">{settingsErrors.ssimThreshold}</span>
              {/if}
            </label>
            <label>
              Resize size
              <input type="number" min="1" bind:value={settingsDraft.resize_size} />
              {#if settingsErrors.resizeSize}<span class="field-error">{settingsErrors.resizeSize}</span>{/if}
            </label>
            <label>
              Histogram bins
              <input type="number" min="2" bind:value={settingsDraft.hist_bins} />
              {#if settingsErrors.histBins}<span class="field-error">{settingsErrors.histBins}</span>{/if}
            </label>
            <label>
              Histogram method
              <select bind:value={settingsDraft.hist_method}>
                <option value="correlation">correlation</option>
                <option value="bhattacharyya">bhattacharyya</option>
              </select>
            </label>
            <label>
              Alpha threshold
              <input
                type="number"
                step="0.01"
                min="0"
                max="1"
                bind:value={settingsDraft.alpha_threshold}
              />
              {#if settingsErrors.alphaThreshold}
                <span class="field-error">{settingsErrors.alphaThreshold}</span>
              {/if}
            </label>
            <label>
              Max decode dimension (optional)
              <input
                type="number"
                min="1"
                value={settingsDraft.max_decode_dimension_px ?? ""}
                oninput={(e) => {
                  const v = (e.currentTarget as HTMLInputElement).value.trim();
                  settingsDraft.max_decode_dimension_px = v ? Number(v) : null;
                }}
              />
              {#if settingsErrors.maxDecodeDimension}
                <span class="field-error">{settingsErrors.maxDecodeDimension}</span>
              {/if}
            </label>
          </div>

          {#if saveMessage}
            <p class="save-ok">{saveMessage}</p>
          {/if}
          {#if loadError}
            <p class="save-error">{loadError}</p>
          {/if}
        </div>

        <div class="modal-footer">
          <button type="button" class="action-btn secondary" onclick={() => cancelSettings()}>
            Cancel
          </button>
          <button type="button" class="action-btn" onclick={() => void saveSettings()}>Save</button>
        </div>
      </div>
    </div>
  {/if}

  {#if selectedGroup}
    <div
      class="overlay"
      role="button"
      tabindex="0"
      aria-label="Close group details"
      onclick={(e) => {
        if (e.target === e.currentTarget) closeGroupDetails();
      }}
      onkeydown={(e) => {
        if (e.key === "Escape" || e.key === "Enter" || e.key === " ") closeGroupDetails();
      }}
    >
      <div class="modal detail-modal">
        <div class="modal-header">
          <h2>Group Details</h2>
          <button
            type="button"
            class="modal-close-btn"
            aria-label="Close group details"
            onclick={() => closeGroupDetails()}
          >
            <svg
              width="16"
              height="16"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <line x1="18" y1="6" x2="6" y2="18" />
              <line x1="6" y1="6" x2="18" y2="18" />
            </svg>
          </button>
        </div>

        <div class="modal-body">
          <div class="detail-header">
            <div class="detail-main">
              <div class="result-title">{groupTitle(selectedGroup)}</div>
              <div class="result-meta">id: {selectedGroup.id}</div>
              <div class="result-meta">score: {selectedGroup.score == null ? "-" : selectedGroup.score.toFixed(3)}</div>
              <div class="result-meta">images: {selectedGroup.images.length}</div>
            </div>
            <div class="reason-badge">{reasonKindLabel(selectedGroup.reason_kind)}</div>
          </div>

          <section class="detail-section">
            <h3>Images</h3>
            <div class="detail-grid">
              {#each selectedGroup.images as imagePath (imagePath)}
                <figure class="thumb-item detail-thumb-item" title={imagePath}>
                  <img src={convertFileSrc(imagePath)} alt={imagePath} loading="lazy" />
                  <figcaption>{toBaseName(imagePath)}</figcaption>
                </figure>
              {/each}
            </div>
          </section>

          <section class="detail-section">
            <h3>Reasons</h3>
            {#if selectedGroup.reasons.length === 0}
              <p class="stub">No pair reasons were provided for this group.</p>
            {:else}
              <ul class="reasons-list">
                {#each selectedGroup.reasons as reason (`${reason.left}:${reason.right}:${reason.type}`)}
                  <li class="reason-item">
                    <div class="reason-item-header">
                      <div class="reason-pair">{toBaseName(reason.left)} ↔ {toBaseName(reason.right)}</div>
                      <div class="reason-type">{pairReasonTypeLabel(reason.type)}</div>
                    </div>
                    <div class="reason-metrics">
                      {#if reason.composite_score != null}
                        <span class="reason-metric">composite: {formatMetricValue(reason.composite_score)}</span>
                      {/if}
                      {#if reason.phash}
                        <span class="reason-metric">
                          pHash: score {formatMetricValue(reason.phash.score)}, raw {formatMetricValue(reason.phash.raw)}, valid {reason.phash.valid ? "yes" : "no"}
                        </span>
                      {/if}
                      {#if reason.ssim}
                        <span class="reason-metric">
                          SSIM: score {formatMetricValue(reason.ssim.score)}, raw {formatMetricValue(reason.ssim.raw)}, valid {reason.ssim.valid ? "yes" : "no"}
                        </span>
                      {/if}
                      {#if reason.histogram}
                        <span class="reason-metric">
                          Histogram: score {formatMetricValue(reason.histogram.score)}, raw {formatMetricValue(reason.histogram.raw)}, valid {reason.histogram.valid ? "yes" : "no"}
                        </span>
                      {/if}
                    </div>
                  </li>
                {/each}
              </ul>
            {/if}
          </section>
        </div>
      </div>
    </div>
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

  .browse-btn {
    flex-shrink: 0;
    width: 2.25rem;
    padding: 0;
    border-radius: 6px;
    border: 1px solid #474957;
    background: #32343f;
    color: #d7d8e0;
    font-size: 1rem;
    line-height: 1;
    cursor: pointer;
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

  .action-btn.secondary {
    border-color: #3f4150;
    background: #2a2b33;
    color: #a1a3b0;
  }

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
    display: flex;
    flex-direction: column;
  }

  .stub-label {
    font-size: 0.72rem;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: #8b8d9a;
    font-weight: 600;
  }

  .stub {
    margin: 0;
    font-size: 0.8rem;
    color: #b4b6c2;
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

  .results-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 1rem;
    flex-wrap: wrap;
  }

  .results-header h2 {
    margin: 0;
    font-size: 1.02rem;
  }

  .results-toolbar {
    display: flex;
    flex-direction: column;
    gap: 0.55rem;
    margin-top: 0.2rem;
  }

  .toolbar-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-wrap: wrap;
  }

  .toolbar-label {
    font-size: 0.76rem;
    color: #aeb1bf;
    font-weight: 600;
  }

  .toolbar-field {
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    font-size: 0.76rem;
    color: #aeb1bf;
  }

  .toolbar-field select {
    border: 1px solid #3f4150;
    border-radius: 6px;
    background: #1e1f26;
    color: #f2f3f7;
    padding: 0.34rem 0.45rem;
    font-size: 0.78rem;
  }

  .reason-chips {
    display: flex;
    flex-wrap: wrap;
    gap: 0.35rem;
  }

  .chip {
    border: 1px solid #3f4150;
    border-radius: 999px;
    background: #1e1f26;
    color: #aeb1bf;
    padding: 0.2rem 0.55rem;
    font-size: 0.74rem;
    cursor: pointer;
  }

  .chip:hover {
    border-color: #5c7cfa;
    color: #f2f3f7;
  }

  .chip.chip-active {
    border-color: #5c7cfa;
    color: #dce3ff;
    background: rgb(92 124 250 / 15%);
  }

  .pagination-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 0.55rem;
    flex-wrap: wrap;
  }

  .pagination-meta {
    font-size: 0.75rem;
    color: #aeb1bf;
  }

  .pagination-controls {
    display: flex;
    gap: 0.35rem;
    flex-wrap: wrap;
  }

  .empty-state {
    margin-top: 0.5rem;
    border: 1px dashed #3f4150;
    border-radius: 8px;
    padding: 1rem;
    color: #aeb1bf;
  }

  .results-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: 0.8rem;
  }

  .result-card {
    border: 1px solid #3f4150;
    border-radius: 8px;
    background: #1f2027;
    padding: 0.6rem;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .result-card-clickable {
    transition: border-color 0.14s ease;
  }

  .result-card-clickable:hover {
    border-color: #5c7cfa;
  }

  .result-card-button {
    border: 0;
    padding: 0;
    margin: 0;
    width: 100%;
    background: transparent;
    color: inherit;
    text-align: left;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    cursor: pointer;
  }

  .result-card-button:focus-visible {
    outline: 2px solid #5c7cfa;
    outline-offset: 4px;
    border-radius: 6px;
  }

  .result-card-header {
    display: flex;
    gap: 0.5rem;
    flex-wrap: wrap;
    align-items: center;
  }

  .result-title {
    font-weight: 600;
    color: #f0f1f6;
  }

  .reason-badge {
    border: 1px solid #4f5f9f;
    border-radius: 999px;
    background: rgb(92 124 250 / 14%);
    color: #dce3ff;
    font-size: 0.7rem;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    font-weight: 600;
    padding: 0.13rem 0.45rem;
  }

  .result-meta {
    font-size: 0.75rem;
    color: #aeb1bf;
  }

  .thumb-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(76px, 1fr));
    gap: 0.35rem;
  }

  .thumb-item {
    margin: 0;
    border: 1px solid #34353f;
    border-radius: 6px;
    background: #181920;
    overflow: hidden;
  }

  .thumb-item img {
    display: block;
    width: 100%;
    height: 76px;
    object-fit: cover;
    background: #101116;
  }

  .thumb-item figcaption {
    padding: 0.25rem;
    font-size: 0.66rem;
    color: #9ea1ad;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .overlay {
    position: fixed;
    inset: 0;
    z-index: 200;
    background: rgb(0 0 0 / 55%);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .modal {
    background: #24252c;
    border: 1px solid #3f4150;
    border-radius: 12px;
    width: min(54rem, 95vw);
    max-height: 92vh;
    display: flex;
    flex-direction: column;
    box-shadow: 0 8px 32px rgb(0 0 0 / 45%);
  }

  .detail-modal {
    width: min(66rem, 96vw);
  }

  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.85rem 1rem;
    border-bottom: 1px solid #34353f;
  }

  .modal-header h2 {
    margin: 0;
    font-size: 1rem;
    font-weight: 600;
    color: #f2f3f7;
  }

  .modal-close-btn {
    padding: 0.3rem;
    border-radius: 4px;
    border: 1px solid transparent;
    background: transparent;
    color: #8b8d9a;
    cursor: pointer;
  }

  .modal-close-btn:hover {
    color: #fff;
    border-color: #474957;
    background: #32343f;
  }

  .modal-body {
    padding: 1rem;
    overflow: auto;
    display: flex;
    flex-direction: column;
    gap: 0.8rem;
  }

  .detail-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 0.6rem;
    flex-wrap: wrap;
  }

  .detail-main {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.45rem 0.6rem;
  }

  .detail-section {
    display: flex;
    flex-direction: column;
    gap: 0.45rem;
  }

  .detail-section h3 {
    margin: 0;
    font-size: 0.9rem;
    color: #d9dbea;
  }

  .detail-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
    gap: 0.45rem;
  }

  .detail-thumb-item img {
    height: 110px;
  }

  .reasons-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.45rem;
  }

  .reason-item {
    border: 1px solid #34353f;
    border-radius: 8px;
    background: #1f2027;
    padding: 0.5rem 0.6rem;
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

  .reason-item-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 0.6rem;
    flex-wrap: wrap;
  }

  .reason-pair {
    color: #f0f1f6;
    font-size: 0.78rem;
    font-weight: 520;
  }

  .reason-type {
    font-size: 0.7rem;
    color: #aeb1bf;
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .reason-metrics {
    display: flex;
    flex-wrap: wrap;
    gap: 0.32rem;
  }

  .reason-metric {
    border: 1px solid #3f4150;
    border-radius: 999px;
    background: #1a1b21;
    color: #c9cbd8;
    font-size: 0.72rem;
    padding: 0.17rem 0.48rem;
  }

  .settings-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
    gap: 0.55rem;
  }

  .settings-grid label {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    font-size: 0.78rem;
    color: #d7d8e0;
  }

  .settings-grid label input[type="checkbox"] {
    width: auto;
    margin-right: 0.4rem;
  }

  .settings-grid label input[type="number"],
  .settings-grid label select {
    width: 100%;
    box-sizing: border-box;
    padding: 0.45rem 0.55rem;
    border-radius: 6px;
    border: 1px solid #3f4150;
    background: #1e1f26;
    color: #f2f3f7;
    font-size: 0.82rem;
  }

  .save-ok {
    margin: 0;
    color: #8fd49a;
    font-size: 0.78rem;
  }

  .save-error {
    margin: 0;
    color: #f0a8a8;
    font-size: 0.78rem;
  }

  .field-error {
    color: #f0a8a8;
    font-size: 0.72rem;
    line-height: 1.2;
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: 0.45rem;
    padding: 0.75rem 1rem 1rem;
    border-top: 1px solid #34353f;
  }
</style>


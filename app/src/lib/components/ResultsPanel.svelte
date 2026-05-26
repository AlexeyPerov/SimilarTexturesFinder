<script lang="ts">
  import Banner from "$lib/components/Banner.svelte";
  import ReasonLegend from "$lib/components/ReasonLegend.svelte";
  import ThumbnailImage from "$lib/components/ThumbnailImage.svelte";
  import { reasonKindLabel, toBaseName } from "$lib/groupUtils";
  import type { GroupPreview, GroupReasonKind, SortOption, TabBanner } from "$lib/types";

  type Props = {
    hasResult: boolean;
    filteredGroupCount: number;
    uniqueImageCount: number;
    lastScanDurationLabel: string | null;
    staleResults: boolean;
    searchQuery: string;
    hideSingleImageGroups: boolean;
    previewMaxPx: number;
    running: boolean;
    canScanAgain: boolean;
    banner: TabBanner;
    sortOption: SortOption;
    availableReasonKinds: GroupReasonKind[];
    selectedReasonKinds: GroupReasonKind[];
    resultsPage: number;
    totalPages: number;
    previewGroups: GroupPreview[];
    onDismissBanner?: () => void;
    onExport: () => void;
    onExportCsv: () => void;
    onImport: () => void;
    onScanAgain: () => void;
    onToggleHideSingletons: (next: boolean) => void;
    onToggleReasonFilter: (kind: GroupReasonKind) => void;
    onGoToPage: (page: number) => void;
    onOpenGroup: (groupId: number) => void;
    onOpenImage?: (path: string) => void;
  };

  let {
    hasResult,
    filteredGroupCount,
    uniqueImageCount,
    lastScanDurationLabel,
    staleResults,
    searchQuery = $bindable(""),
    hideSingleImageGroups,
    previewMaxPx,
    running,
    canScanAgain,
    banner,
    sortOption = $bindable("count_desc" as SortOption),
    availableReasonKinds,
    selectedReasonKinds,
    resultsPage,
    totalPages,
    previewGroups,
    onDismissBanner,
    onExport,
    onExportCsv,
    onImport,
    onScanAgain,
    onToggleHideSingletons,
    onToggleReasonFilter,
    onGoToPage,
    onOpenGroup,
    onOpenImage,
  }: Props = $props();

  function hasReasonFilter(kind: GroupReasonKind) {
    return selectedReasonKinds.includes(kind);
  }
</script>

<section class="panel" aria-label="Results">
  <Banner banner={banner} onDismiss={onDismissBanner} />

  {#if staleResults}
    <div class="stale-hint" role="status">
      Settings changed since last scan — run Scan again to apply.
    </div>
  {/if}

  <div class="results-header">
    <div>
      <h2>Scan Results</h2>
      {#if hasResult}
        <p class="summary-strip">
          {filteredGroupCount} groups · {uniqueImageCount} unique images
          {#if lastScanDurationLabel}
            · last scan {lastScanDurationLabel}
          {/if}
        </p>
      {:else}
        <p class="stub">No in-memory scan result yet</p>
      {/if}
    </div>
    <div class="header-actions">
      <button type="button" class="action-btn secondary" onclick={() => onImport()}>Import JSON</button>
      <button type="button" class="action-btn" disabled={!hasResult} onclick={() => onExport()}>Export JSON</button>
      <button type="button" class="action-btn" disabled={!hasResult} onclick={() => onExportCsv()}>Export CSV</button>
      <button type="button" class="action-btn" disabled={!canScanAgain || running} onclick={() => onScanAgain()}>
        Scan again
      </button>
    </div>
  </div>

  {#if !hasResult}
    <div class="empty-state">Run a scan in the Scan tab or import a JSON export to populate results.</div>
  {:else}
    <div class="results-toolbar">
      <div class="toolbar-row">
        <label class="toolbar-field search-field">
          <span>Search</span>
          <input
            type="search"
            placeholder="Filter by filename or path…"
            bind:value={searchQuery}
          />
        </label>
        <label class="toolbar-field checkbox-field">
          <input
            type="checkbox"
            checked={hideSingleImageGroups}
            onchange={(e) => onToggleHideSingletons((e.currentTarget as HTMLInputElement).checked)}
          />
          <span>Hide single-image groups</span>
        </label>
      </div>

      <div class="toolbar-row">
        <label class="toolbar-field">
          <span>Sort by</span>
          <select bind:value={sortOption}>
            <option value="count_desc">Count (high to low)</option>
            <option value="count_asc">Count (low to high)</option>
            <option value="score_desc">Score (high to low)</option>
            <option value="score_asc">Score (low to high)</option>
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
                onclick={() => onToggleReasonFilter(kind)}
              >
                {reasonKindLabel(kind)}
              </button>
            {/each}
          {/if}
        </div>
      </div>

      <ReasonLegend />

      <div class="pagination-row">
        <div class="pagination-meta">Page {resultsPage} / {totalPages}</div>
        <div class="pagination-controls">
          <button type="button" class="action-btn tertiary" disabled={resultsPage <= 1} onclick={() => onGoToPage(1)}>First</button>
          <button type="button" class="action-btn tertiary" disabled={resultsPage <= 1} onclick={() => onGoToPage(resultsPage - 1)}>Prev</button>
          <button type="button" class="action-btn tertiary" disabled={resultsPage >= totalPages} onclick={() => onGoToPage(resultsPage + 1)}>Next</button>
          <button type="button" class="action-btn tertiary" disabled={resultsPage >= totalPages} onclick={() => onGoToPage(totalPages)}>Last</button>
        </div>
      </div>
    </div>

    {#if previewGroups.length === 0}
      <div class="empty-state">No groups match the current filters.</div>
    {:else}
      <div class="results-grid">
        {#each previewGroups as group (group.id)}
          <article class="result-card result-card-clickable">
            <button
              type="button"
              class="result-card-button"
              aria-label={`Open details for ${group.title}`}
              onclick={() => onOpenGroup(group.id)}
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
                    <ThumbnailImage
                      imagePath={imagePath}
                      maxPx={previewMaxPx}
                      alt={imagePath}
                      class="grid-thumb"
                      {onOpenImage}
                    />
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
  {/if}
</section>

<style>
  .panel {
    display: flex;
    flex-direction: column;
    gap: 0.65rem;
    padding: 1rem;
    border-radius: 10px;
    background: var(--bg-panel);
    border: 1px solid var(--border-subtle);
    min-height: 0;
  }

  .stale-hint {
    border: 1px solid var(--border-stale);
    background: var(--bg-stale);
    color: var(--text-stale);
    border-radius: 8px;
    padding: 0.55rem 0.75rem;
    font-size: 0.82rem;
  }

  .results-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 1rem;
    flex-wrap: wrap;
  }

  .results-header h2 {
    margin: 0;
    font-size: 1.02rem;
  }

  .summary-strip,
  .stub {
    margin: 0.2rem 0 0;
    font-size: 0.8rem;
    color: var(--text-muted);
  }

  .header-actions {
    display: flex;
    flex-wrap: wrap;
    gap: 0.45rem;
  }

  .action-btn {
    align-self: flex-start;
    padding: 0.45rem 0.85rem;
    border-radius: 6px;
    border: 1px solid var(--border-strong);
    background: var(--bg-button);
    color: var(--text-secondary);
    font-size: 0.82rem;
    font-weight: 500;
    cursor: pointer;
  }

  .action-btn:hover:not(:disabled) {
    border-color: var(--accent);
    color: var(--accent-hover);
  }

  .action-btn:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }

  .action-btn.secondary,
  .action-btn.tertiary {
    border-color: var(--border);
    background: var(--bg-button-secondary);
    color: var(--text-tab);
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
    color: var(--text-muted);
    font-weight: 600;
  }

  .toolbar-field {
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    font-size: 0.76rem;
    color: var(--text-muted);
  }

  .search-field input {
    min-width: 14rem;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--bg-input);
    color: var(--text-primary);
    padding: 0.34rem 0.45rem;
    font-size: 0.78rem;
  }

  .checkbox-field input {
    margin: 0;
  }

  .toolbar-field select {
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--bg-input);
    color: var(--text-primary);
    padding: 0.34rem 0.45rem;
    font-size: 0.78rem;
  }

  .reason-chips {
    display: flex;
    flex-wrap: wrap;
    gap: 0.35rem;
  }

  .chip {
    border: 1px solid var(--border);
    border-radius: 999px;
    background: var(--bg-panel-alt);
    color: var(--text-muted);
    padding: 0.2rem 0.55rem;
    font-size: 0.74rem;
    cursor: pointer;
  }

  .chip:hover {
    border-color: var(--accent);
    color: var(--text-primary);
  }

  .chip.chip-active {
    border-color: var(--accent);
    color: var(--text-accent-label);
    background: var(--bg-tag);
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
    color: var(--text-muted);
  }

  .pagination-controls {
    display: flex;
    gap: 0.35rem;
    flex-wrap: wrap;
  }

  .empty-state {
    margin-top: 0.5rem;
    border: 1px dashed var(--border);
    border-radius: 8px;
    padding: 1rem;
    color: var(--text-muted);
  }

  .results-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: 0.8rem;
  }

  .result-card {
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--bg-modal-section);
    padding: 0.6rem;
  }

  .result-card-clickable {
    transition: border-color 0.14s ease;
  }

  .result-card-clickable:hover {
    border-color: var(--accent);
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
    outline: 2px solid var(--outline-focus);
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
    color: var(--text-secondary);
  }

  .reason-badge {
    border: 1px solid var(--border-tag);
    border-radius: 999px;
    background: var(--bg-tag);
    color: var(--text-accent-label);
    font-size: 0.7rem;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    font-weight: 600;
    padding: 0.13rem 0.45rem;
  }

  .result-meta {
    font-size: 0.75rem;
    color: var(--text-muted);
  }

  .thumb-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(76px, 1fr));
    gap: 0.35rem;
  }

  .thumb-item {
    margin: 0;
    border: 1px solid var(--border-subtle);
    border-radius: 6px;
    background: var(--bg-card-inner);
    overflow: hidden;
  }

  :global(.grid-thumb) {
    height: 76px;
  }

  .thumb-item figcaption {
    padding: 0.25rem;
    font-size: 0.66rem;
    color: var(--text-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>

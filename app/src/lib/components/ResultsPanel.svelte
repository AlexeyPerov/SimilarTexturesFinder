<script lang="ts">
  import { convertFileSrc } from "@tauri-apps/api/core";
  import { reasonKindLabel, toBaseName } from "$lib/groupUtils";
  import type { GroupPreview, GroupReasonKind, SortOption, TabBanner } from "$lib/types";
  import Banner from "$lib/components/Banner.svelte";

  type Props = {
    hasResult: boolean;
    filteredGroupCount: number;
    banner: TabBanner;
    sortOption: SortOption;
    availableReasonKinds: GroupReasonKind[];
    selectedReasonKinds: GroupReasonKind[];
    resultsPage: number;
    totalPages: number;
    previewGroups: GroupPreview[];
    onDismissBanner?: () => void;
    onExport: () => void;
    onToggleReasonFilter: (kind: GroupReasonKind) => void;
    onGoToPage: (page: number) => void;
    onOpenGroup: (groupId: number) => void;
  };

  let {
    hasResult,
    filteredGroupCount,
    banner,
    sortOption = $bindable("count_desc" as SortOption),
    availableReasonKinds,
    selectedReasonKinds,
    resultsPage,
    totalPages,
    previewGroups,
    onDismissBanner,
    onExport,
    onToggleReasonFilter,
    onGoToPage,
    onOpenGroup,
  }: Props = $props();

  function hasReasonFilter(kind: GroupReasonKind) {
    return selectedReasonKinds.includes(kind);
  }
</script>

<section class="panel" aria-label="Results">
  <Banner banner={banner} onDismiss={onDismissBanner} />

  <div class="results-header">
    <div>
      <h2>Scan Results</h2>
      <p class="stub">
        {#if hasResult}
          {filteredGroupCount} groups loaded
        {:else}
          No in-memory scan result yet
        {/if}
      </p>
    </div>
    <button type="button" class="action-btn" disabled={!hasResult} onclick={() => onExport()}>
      Export JSON
    </button>
  </div>

  {#if !hasResult}
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
                onclick={() => onToggleReasonFilter(kind)}
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
          <button type="button" class="action-btn tertiary" disabled={resultsPage <= 1} onclick={() => onGoToPage(1)}>
            First
          </button>
          <button type="button" class="action-btn tertiary" disabled={resultsPage <= 1} onclick={() => onGoToPage(resultsPage - 1)}>
            Prev
          </button>
          <button type="button" class="action-btn tertiary" disabled={resultsPage >= totalPages} onclick={() => onGoToPage(resultsPage + 1)}>
            Next
          </button>
          <button type="button" class="action-btn tertiary" disabled={resultsPage >= totalPages} onclick={() => onGoToPage(totalPages)}>
            Last
          </button>
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
  {/if}
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

  .stub {
    margin: 0;
    font-size: 0.8rem;
    color: #b4b6c2;
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

  .action-btn.tertiary {
    border-color: #3f4150;
    background: #2a2b33;
    color: #a1a3b0;
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
</style>

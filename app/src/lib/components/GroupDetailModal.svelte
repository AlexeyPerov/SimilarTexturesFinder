<script lang="ts">
  import ThumbnailImage from "$lib/components/ThumbnailImage.svelte";
  import {
    formatMetricValue,
    groupTitle,
    pairReasonTypeLabel,
    reasonKindLabel,
    toBaseName,
  } from "$lib/groupUtils";
  import type { ScanGroup } from "$lib/types";
  import { metricDetailsHelp } from "$lib/settingsHelp";
  import { trapFocus } from "$lib/modalFocus";

  type Props = {
    group: ScanGroup | null;
    previewMaxPx?: number;
    onClose: () => void;
    onCopyPath: (path: string) => void;
    onCopyAllPaths: (paths: string[]) => void;
    onRevealPath: (path: string) => void;
    onOpenImage?: (path: string) => void;
  };

  let {
    group,
    previewMaxPx = 256,
    onClose,
    onCopyPath,
    onCopyAllPaths,
    onRevealPath,
    onOpenImage,
  }: Props = $props();

  let contentReady = $state(false);
  let deferToken = 0;

  let showContentLoader = $derived(group != null && !contentReady);

  $effect(() => {
    group?.id;
    contentReady = false;
    const token = ++deferToken;
    requestAnimationFrame(() => {
      requestAnimationFrame(() => {
        if (token !== deferToken) return;
        contentReady = true;
      });
    });
  });

  function similarityPercent(score: number | null | undefined): number | null {
    if (score == null || !Number.isFinite(score)) return null;
    return Math.round(Math.max(0, Math.min(1, score)) * 100);
  }
</script>

<div
  class="overlay"
  role="presentation"
  onclick={(e) => {
    if (e.target === e.currentTarget) onClose();
  }}
>
  <div
    class="modal detail-modal"
    role="dialog"
    aria-modal="true"
    aria-labelledby="group-detail-title"
    use:trapFocus
  >
    <div class="modal-header">
      <h2 id="group-detail-title">Group Details</h2>
      <div class="header-actions">
        {#if group}
          <button type="button" class="action-btn tertiary" onclick={() => onCopyAllPaths(group.images)}>
            Copy all paths
          </button>
        {/if}
        <button type="button" class="modal-close-btn" aria-label="Close group details" onclick={() => onClose()}>
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <line x1="18" y1="6" x2="6" y2="18" />
            <line x1="6" y1="6" x2="18" y2="18" />
          </svg>
        </button>
      </div>
    </div>

    <div class="modal-body">
      {#if group == null}
        <div class="content-loading" aria-live="polite" aria-busy="true">
          <div class="detail-spinner"></div>
          <p>Opening group details…</p>
        </div>
      {:else}
        <div class="detail-header">
          <div class="detail-main">
            <div class="result-title">{groupTitle(group)}</div>
            <div class="result-meta">id: {group.id}</div>
            <div class="result-meta">score: {group.score == null ? "-" : group.score.toFixed(3)}</div>
            <div class="result-meta">images: {group.images.length}</div>
          </div>
          <div class="reason-badge">{reasonKindLabel(group.reason_kind)}</div>
        </div>

        {#if showContentLoader}
          <div class="content-loading" aria-live="polite" aria-busy="true">
            <div class="detail-spinner"></div>
            <p>Loading group details…</p>
          </div>
        {:else}
          <div class="detail-content">
            <section class="detail-section">
              <h3>Images</h3>
              <div class="detail-grid">
                {#each group.images as imagePath (imagePath)}
                  <div class="detail-thumb-wrap">
                    <figure class="thumb-item detail-thumb-item" title={imagePath}>
                      <ThumbnailImage
                        imagePath={imagePath}
                        maxPx={previewMaxPx}
                        alt={imagePath}
                        class="detail-img"
                        {onOpenImage}
                      />
                      <figcaption>{toBaseName(imagePath)}</figcaption>
                    </figure>
                    <div class="thumb-actions">
                      <button type="button" class="icon-btn" title="Copy path" onclick={() => onCopyPath(imagePath)}>Copy</button>
                      <button type="button" class="icon-btn" title="Reveal in folder" onclick={() => onRevealPath(imagePath)}>Reveal</button>
                    </div>
                  </div>
                {/each}
              </div>
            </section>

            <section class="detail-section">
              <h3>Similarity pairs</h3>
              {#if group.reasons.length === 0}
                <p class="stub">No pair reasons were provided for this group.</p>
              {:else}
                <ul class="reasons-list">
                  {#each group.reasons as reason (`${reason.left}:${reason.right}:${reason.type}`)}
                    {@const pct = similarityPercent(reason.composite_score)}
                    <li class="reason-item">
                      <div class="reason-item-header">
                        <div class="reason-pair">{toBaseName(reason.left)} ↔ {toBaseName(reason.right)}</div>
                        <div class="reason-type">{pairReasonTypeLabel(reason.type)}</div>
                      </div>

                      <div class="pair-compare">
                        <figure class="pair-thumb">
                          <ThumbnailImage
                            imagePath={reason.left}
                            maxPx={previewMaxPx}
                            alt={reason.left}
                            class="pair-img"
                            {onOpenImage}
                          />
                          <figcaption>{toBaseName(reason.left)}</figcaption>
                        </figure>
                        <div class="pair-score">
                          {#if pct != null}
                            <div class="score-value">{pct}%</div>
                            <div class="score-bar-track">
                              <div class="score-bar-fill" style:width="{pct}%"></div>
                            </div>
                            <div class="score-caption">similarity</div>
                          {:else}
                            <div class="score-caption">Exact match</div>
                          {/if}
                        </div>
                        <figure class="pair-thumb">
                          <ThumbnailImage
                            imagePath={reason.right}
                            maxPx={previewMaxPx}
                            alt={reason.right}
                            class="pair-img"
                            {onOpenImage}
                          />
                          <figcaption>{toBaseName(reason.right)}</figcaption>
                        </figure>
                      </div>

                      {#if reason.phash || reason.ssim || reason.histogram}
                        <details class="metric-details">
                          <summary title={metricDetailsHelp.metric_details}>Metric details</summary>
                          <div class="reason-metrics">
                            {#if reason.composite_score != null}
                              <span class="reason-metric">
                                <span class="metric-label" title={metricDetailsHelp.composite}>composite</span>:
                                {formatMetricValue(reason.composite_score)}
                              </span>
                            {/if}
                            {#if reason.phash}
                              <span class="reason-metric">
                                <span class="metric-label" title={metricDetailsHelp.phash}>pHash</span>:
                                <span class="metric-label" title={metricDetailsHelp.score}>score</span> {formatMetricValue(reason.phash.score)},
                                <span class="metric-label" title={metricDetailsHelp.phash_raw}>raw</span> {formatMetricValue(reason.phash.raw)},
                                <span class="metric-label" title={metricDetailsHelp.valid}>valid</span> {reason.phash.valid ? "yes" : "no"}
                              </span>
                            {/if}
                            {#if reason.ssim}
                              <span class="reason-metric">
                                <span class="metric-label" title={metricDetailsHelp.ssim}>SSIM</span>:
                                <span class="metric-label" title={metricDetailsHelp.score}>score</span> {formatMetricValue(reason.ssim.score)},
                                <span class="metric-label" title={metricDetailsHelp.ssim_raw}>raw</span> {formatMetricValue(reason.ssim.raw)},
                                <span class="metric-label" title={metricDetailsHelp.valid}>valid</span> {reason.ssim.valid ? "yes" : "no"}
                              </span>
                            {/if}
                            {#if reason.histogram}
                              <span class="reason-metric">
                                <span class="metric-label" title={metricDetailsHelp.histogram}>Histogram</span>:
                                <span class="metric-label" title={metricDetailsHelp.score}>score</span> {formatMetricValue(reason.histogram.score)},
                                <span class="metric-label" title={metricDetailsHelp.histogram_raw}>raw</span> {formatMetricValue(reason.histogram.raw)},
                                <span class="metric-label" title={metricDetailsHelp.valid}>valid</span> {reason.histogram.valid ? "yes" : "no"}
                              </span>
                            {/if}
                          </div>
                        </details>
                      {/if}
                    </li>
                  {/each}
                </ul>
              {/if}
            </section>
          </div>
        {/if}
      {/if}
    </div>
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    z-index: 200;
    background: var(--bg-overlay);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .modal {
    background: var(--bg-modal);
    border: 1px solid var(--border);
    border-radius: 12px;
    width: min(54rem, 95vw);
    max-height: 92vh;
    display: flex;
    flex-direction: column;
    box-shadow: var(--shadow-modal);
  }

  .detail-modal {
    width: min(66rem, 96vw);
  }

  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.85rem 1rem;
    border-bottom: 1px solid var(--border-subtle);
    gap: 0.5rem;
  }

  .modal-header h2 {
    margin: 0;
    font-size: 1rem;
    font-weight: 600;
    color: var(--text-primary);
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 0.4rem;
  }

  .modal-close-btn {
    padding: 0.3rem;
    border-radius: 4px;
    border: 1px solid transparent;
    background: transparent;
    color: var(--text-faint);
    cursor: pointer;
  }

  .modal-close-btn:hover {
    color: var(--accent-hover);
    border-color: var(--border-strong);
    background: var(--bg-button);
  }

  .modal-body {
    padding: 1rem;
    overflow: auto;
    display: flex;
    flex-direction: column;
    gap: 0.8rem;
    min-height: 12rem;
  }

  .content-loading {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 0.65rem;
    min-height: 10rem;
    padding: 1.5rem 1rem;
  }

  .content-loading p {
    margin: 0;
    font-size: 0.82rem;
    color: var(--text-secondary);
  }

  .detail-spinner {
    width: 1.75rem;
    height: 1.75rem;
    border: 2px solid var(--border);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: detail-spin 0.7s linear infinite;
  }

  @keyframes detail-spin {
    to {
      transform: rotate(360deg);
    }
  }

  .detail-content {
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

  .result-title {
    font-weight: 600;
    color: var(--text-secondary);
  }

  .result-meta {
    font-size: 0.75rem;
    color: var(--text-muted);
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

  .detail-section {
    display: flex;
    flex-direction: column;
    gap: 0.45rem;
  }

  .detail-section h3 {
    margin: 0;
    font-size: 0.9rem;
    color: var(--text-secondary);
  }

  .detail-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
    gap: 0.45rem;
  }

  .detail-thumb-wrap {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .thumb-item {
    margin: 0;
    border: 1px solid var(--border-subtle);
    border-radius: 6px;
    background: var(--bg-card-inner);
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  :global(.detail-img) {
    height: 110px;
  }

  .thumb-item figcaption {
    padding: 0.25rem;
    font-size: 0.66rem;
    color: var(--text-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .thumb-actions {
    display: flex;
    gap: 0.25rem;
    padding: 0.25rem;
  }

  .icon-btn,
  .action-btn.tertiary {
    border: 1px solid var(--border);
    border-radius: 4px;
    background: var(--bg-panel-alt);
    color: var(--text-muted);
    font-size: 0.66rem;
    padding: 0.15rem 0.35rem;
    cursor: pointer;
  }

  .icon-btn:hover,
  .action-btn.tertiary:hover {
    border-color: var(--accent);
    color: var(--accent-hover);
  }

  .stub {
    margin: 0;
    font-size: 0.8rem;
    color: var(--text-muted);
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
    border: 1px solid var(--border-subtle);
    border-radius: 8px;
    background: var(--bg-modal-section);
    padding: 0.5rem 0.6rem;
    display: flex;
    flex-direction: column;
    gap: 0.45rem;
  }

  .reason-item-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 0.6rem;
    flex-wrap: wrap;
  }

  .reason-pair {
    color: var(--text-secondary);
    font-size: 0.78rem;
    font-weight: 520;
  }

  .reason-type {
    font-size: 0.7rem;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .pair-compare {
    display: grid;
    grid-template-columns: 1fr auto 1fr;
    gap: 0.55rem;
    align-items: center;
  }

  .pair-thumb {
    margin: 0;
    border: 1px solid var(--border-subtle);
    border-radius: 6px;
    background: var(--bg-card-inner);
    overflow: hidden;
  }

  :global(.pair-img) {
    height: 96px;
  }

  .pair-thumb figcaption {
    padding: 0.25rem;
    font-size: 0.66rem;
    color: var(--text-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .pair-score {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.25rem;
    min-width: 4.5rem;
  }

  .score-value {
    font-size: 1rem;
    font-weight: 650;
    color: var(--text-accent-label);
  }

  .score-bar-track {
    width: 100%;
    height: 0.35rem;
    border-radius: 999px;
    background: var(--bg-console);
    border: 1px solid var(--border);
    overflow: hidden;
  }

  .score-bar-fill {
    height: 100%;
    background: var(--accent);
  }

  .score-caption {
    font-size: 0.68rem;
    color: var(--text-faint);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .metric-details summary {
    cursor: pointer;
    font-size: 0.72rem;
    color: var(--text-muted);
  }

  .reason-metrics {
    display: flex;
    flex-wrap: wrap;
    gap: 0.32rem;
    margin-top: 0.35rem;
  }

  .reason-metric {
    border: 1px solid var(--border);
    border-radius: 999px;
    background: var(--bg-console);
    color: var(--text-secondary);
    font-size: 0.72rem;
    padding: 0.17rem 0.48rem;
  }

  .metric-label {
    cursor: help;
    text-decoration: underline dotted var(--link-underline);
    text-underline-offset: 0.12em;
  }
</style>

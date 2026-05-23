<script lang="ts">
  import { convertFileSrc } from "@tauri-apps/api/core";
  import {
    formatMetricValue,
    groupTitle,
    pairReasonTypeLabel,
    reasonKindLabel,
    toBaseName,
  } from "$lib/groupUtils";
  import type { ScanGroup } from "$lib/types";

  type Props = {
    group: ScanGroup;
    onClose: () => void;
  };

  let { group, onClose }: Props = $props();
</script>

<div
  class="overlay"
  role="presentation"
  onclick={(e) => {
    if (e.target === e.currentTarget) onClose();
  }}
  onkeydown={(e) => {
    if (e.key === "Escape") onClose();
  }}
>
  <div class="modal detail-modal" role="dialog" aria-modal="true" aria-labelledby="group-detail-title">
    <div class="modal-header">
      <h2 id="group-detail-title">Group Details</h2>
      <button type="button" class="modal-close-btn" aria-label="Close group details" onclick={() => onClose()}>
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <line x1="18" y1="6" x2="6" y2="18" />
          <line x1="6" y1="6" x2="18" y2="18" />
        </svg>
      </button>
    </div>

    <div class="modal-body">
      <div class="detail-header">
        <div class="detail-main">
          <div class="result-title">{groupTitle(group)}</div>
          <div class="result-meta">id: {group.id}</div>
          <div class="result-meta">score: {group.score == null ? "-" : group.score.toFixed(3)}</div>
          <div class="result-meta">images: {group.images.length}</div>
        </div>
        <div class="reason-badge">{reasonKindLabel(group.reason_kind)}</div>
      </div>

      <section class="detail-section">
        <h3>Images</h3>
        <div class="detail-grid">
          {#each group.images as imagePath (imagePath)}
            <figure class="thumb-item detail-thumb-item" title={imagePath}>
              <img src={convertFileSrc(imagePath)} alt={imagePath} loading="lazy" />
              <figcaption>{toBaseName(imagePath)}</figcaption>
            </figure>
          {/each}
        </div>
      </section>

      <section class="detail-section">
        <h3>Reasons</h3>
        {#if group.reasons.length === 0}
          <p class="stub">No pair reasons were provided for this group.</p>
        {:else}
          <ul class="reasons-list">
            {#each group.reasons as reason (`${reason.left}:${reason.right}:${reason.type}`)}
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

<style>
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

  .result-title {
    font-weight: 600;
    color: #f0f1f6;
  }

  .result-meta {
    font-size: 0.75rem;
    color: #aeb1bf;
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

  .detail-thumb-item img {
    height: 110px;
  }

  .thumb-item figcaption {
    padding: 0.25rem;
    font-size: 0.66rem;
    color: #9ea1ad;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .stub {
    margin: 0;
    font-size: 0.8rem;
    color: #b4b6c2;
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
</style>

<script lang="ts">
  import type { AppSettings, SettingsValidationErrors } from "$lib/types";

  type Props = {
    settingsDraft: AppSettings;
    settingsErrors: SettingsValidationErrors;
    saveMessage: string;
    loadError: string;
    onCancel: () => void;
    onSave: () => void;
  };

  let {
    settingsDraft = $bindable(),
    settingsErrors,
    saveMessage,
    loadError,
    onCancel,
    onSave,
  }: Props = $props();
</script>

<div
  class="overlay"
  role="presentation"
  onclick={(e) => {
    if (e.target === e.currentTarget) onCancel();
  }}
  onkeydown={(e) => {
    if (e.key === "Escape") onCancel();
  }}
>
  <div class="modal" role="dialog" aria-modal="true" aria-labelledby="settings-title">
    <div class="modal-header">
      <h2 id="settings-title">Settings</h2>
      <button type="button" class="modal-close-btn" aria-label="Close settings" onclick={() => onCancel()}>
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
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
          <input type="number" step="0.01" min="0" max="1" bind:value={settingsDraft.threshold} />
          {#if settingsErrors.threshold}<span class="field-error">{settingsErrors.threshold}</span>{/if}
        </label>
        <label>
          pHash weight
          <input type="number" step="0.01" min="0" bind:value={settingsDraft.weights.phash} />
          {#if settingsErrors.phashWeight}<span class="field-error">{settingsErrors.phashWeight}</span>{/if}
        </label>
        <label>
          SSIM weight
          <input type="number" step="0.01" min="0" bind:value={settingsDraft.weights.ssim} />
          {#if settingsErrors.ssimWeight}<span class="field-error">{settingsErrors.ssimWeight}</span>{/if}
        </label>
        <label>
          Histogram weight
          <input type="number" step="0.01" min="0" bind:value={settingsDraft.weights.histogram} />
          {#if settingsErrors.histogramWeight}<span class="field-error">{settingsErrors.histogramWeight}</span>{/if}
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
          {#if settingsErrors.phashMaxDistance}<span class="field-error">{settingsErrors.phashMaxDistance}</span>{/if}
        </label>
        <label>
          SSIM threshold
          <input type="number" step="0.01" min="0" max="1" bind:value={settingsDraft.ssim_threshold} />
          {#if settingsErrors.ssimThreshold}<span class="field-error">{settingsErrors.ssimThreshold}</span>{/if}
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
          <input type="number" step="0.01" min="0" max="1" bind:value={settingsDraft.alpha_threshold} />
          {#if settingsErrors.alphaThreshold}<span class="field-error">{settingsErrors.alphaThreshold}</span>{/if}
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
          {#if settingsErrors.maxDecodeDimension}<span class="field-error">{settingsErrors.maxDecodeDimension}</span>{/if}
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
      <button type="button" class="action-btn secondary" onclick={() => onCancel()}>Cancel</button>
      <button type="button" class="action-btn" onclick={() => onSave()}>Save</button>
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

  .field-error {
    color: #f0a8a8;
    font-size: 0.72rem;
    line-height: 1.2;
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

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: 0.45rem;
    padding: 0.75rem 1rem 1rem;
    border-top: 1px solid #34353f;
  }

  .action-btn {
    padding: 0.45rem 0.85rem;
    border-radius: 6px;
    border: 1px solid #474957;
    background: #32343f;
    color: #d7d8e0;
    font-size: 0.82rem;
    font-weight: 500;
    cursor: pointer;
  }

  .action-btn:hover {
    border-color: #5c7cfa;
    color: #fff;
  }

  .action-btn.secondary {
    border-color: #3f4150;
    background: #2a2b33;
    color: #a1a3b0;
  }
</style>

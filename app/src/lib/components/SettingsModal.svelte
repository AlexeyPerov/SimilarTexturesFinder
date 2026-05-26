<script lang="ts">
  import type { AppSettings, SettingsValidationErrors } from "$lib/types";
  import { applySettingsPreset, presetDescriptions, type SettingsPresetId } from "$lib/settingsPresets";
  import { settingsHelp } from "$lib/settingsHelp";
  import { settingsEqual } from "$lib/settingsUtils";
  import { trapFocus } from "$lib/modalFocus";

  type Props = {
    settingsDraft: AppSettings;
    savedSettings: AppSettings;
    settingsErrors: SettingsValidationErrors;
    saveMessage: string;
    loadError: string;
    onCancel: () => void;
    onSave: () => void;
  };

  let {
    settingsDraft = $bindable(),
    savedSettings,
    settingsErrors,
    saveMessage,
    loadError,
    onCancel,
    onSave,
  }: Props = $props();

  function requestClose() {
    if (!settingsEqual(savedSettings, settingsDraft)) {
      if (!confirm("Discard unsaved settings changes?")) return;
    }
    onCancel();
  }

  function applyPreset(id: SettingsPresetId) {
    settingsDraft = applySettingsPreset(id);
  }
</script>

<div
  class="overlay"
  role="presentation"
  onclick={(e) => {
    if (e.target === e.currentTarget) requestClose();
  }}
>
  <div
    class="modal"
    role="dialog"
    aria-modal="true"
    aria-labelledby="analysis-settings-title"
    use:trapFocus
  >
    <div class="modal-header">
      <h2 id="analysis-settings-title">Analysis settings</h2>
      <button type="button" class="modal-close-btn" aria-label="Close analysis settings" onclick={() => requestClose()}>
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <line x1="18" y1="6" x2="6" y2="18" />
          <line x1="6" y1="6" x2="18" y2="18" />
        </svg>
      </button>
    </div>

    <div class="modal-body">
      <div class="presets-row">
        <span class="presets-label">Presets</span>
        <button type="button" class="preset-btn" onclick={() => applyPreset("fast")}>Fast</button>
        <button type="button" class="preset-btn" onclick={() => applyPreset("balanced")}>Balanced</button>
        <button type="button" class="preset-btn" onclick={() => applyPreset("strict")}>Strict</button>
        <span class="presets-hint">Apply a preset, then click Save.</span>
      </div>
      <ul class="preset-descriptions">
        <li><strong>Fast</strong> — {presetDescriptions.fast}</li>
        <li><strong>Balanced</strong> — {presetDescriptions.balanced}</li>
        <li><strong>Strict</strong> — {presetDescriptions.strict}</li>
      </ul>

      <details class="settings-section" open>
        <summary>Matching</summary>
        <div class="settings-grid">
          <label title={settingsHelp.enable_phash}>
            <span class="field-label">Enable pHash</span>
            <input type="checkbox" bind:checked={settingsDraft.enable_phash} />
          </label>
          <label title={settingsHelp.enable_ssim}>
            <span class="field-label">Enable SSIM</span>
            <input type="checkbox" bind:checked={settingsDraft.enable_ssim} />
          </label>
          <label title={settingsHelp.enable_histogram}>
            <span class="field-label">Enable histogram</span>
            <input type="checkbox" bind:checked={settingsDraft.enable_histogram} />
          </label>
          <label title={settingsHelp.threshold}>
            <span class="field-label">Threshold</span>
            <input type="number" step="0.01" min="0" max="1" bind:value={settingsDraft.threshold} />
            {#if settingsErrors.threshold}<span class="field-error">{settingsErrors.threshold}</span>{/if}
          </label>
          <label title={settingsHelp.hide_single_image_groups}>
            <span class="field-label">Hide groups with 1 image</span>
            <input type="checkbox" bind:checked={settingsDraft.hide_single_image_groups} />
          </label>
        </div>
      </details>

      <details class="settings-section" open>
        <summary>Transforms</summary>
        <div class="settings-grid">
          <label title={settingsHelp.enable_alpha_crop}>
            <span class="field-label">Enable alpha crop</span>
            <input type="checkbox" bind:checked={settingsDraft.enable_alpha_crop} />
          </label>
          <label title={settingsHelp.enable_rotations}>
            <span class="field-label">Enable rotations</span>
            <input type="checkbox" bind:checked={settingsDraft.enable_rotations} />
          </label>
          <label title={settingsHelp.enable_flip}>
            <span class="field-label">Enable flip</span>
            <input type="checkbox" bind:checked={settingsDraft.enable_flip} />
          </label>
        </div>
      </details>

      <details class="settings-section" open>
        <summary>Weights &amp; thresholds</summary>
        <div class="settings-grid">
          <label title={settingsHelp.phash_weight}>
            <span class="field-label">pHash weight</span>
            <input type="number" step="0.01" min="0" bind:value={settingsDraft.weights.phash} />
            {#if settingsErrors.phashWeight}<span class="field-error">{settingsErrors.phashWeight}</span>{/if}
          </label>
          <label title={settingsHelp.ssim_weight}>
            <span class="field-label">SSIM weight</span>
            <input type="number" step="0.01" min="0" bind:value={settingsDraft.weights.ssim} />
            {#if settingsErrors.ssimWeight}<span class="field-error">{settingsErrors.ssimWeight}</span>{/if}
          </label>
          <label title={settingsHelp.histogram_weight}>
            <span class="field-label">Histogram weight</span>
            <input type="number" step="0.01" min="0" bind:value={settingsDraft.weights.histogram} />
            {#if settingsErrors.histogramWeight}<span class="field-error">{settingsErrors.histogramWeight}</span>{/if}
          </label>
          <label title={settingsHelp.phash_max_distance}>
            <span class="field-label">pHash max distance</span>
            <input type="number" min="0" bind:value={settingsDraft.phash_max_distance} />
            {#if settingsErrors.phashMaxDistance}<span class="field-error">{settingsErrors.phashMaxDistance}</span>{/if}
          </label>
          <label title={settingsHelp.ssim_threshold}>
            <span class="field-label">SSIM threshold</span>
            <input type="number" step="0.01" min="0" max="1" bind:value={settingsDraft.ssim_threshold} />
            {#if settingsErrors.ssimThreshold}<span class="field-error">{settingsErrors.ssimThreshold}</span>{/if}
          </label>
        </div>
      </details>

      <details class="settings-section" open>
        <summary>Performance</summary>
        <div class="settings-grid">
          <label title={settingsHelp.resize_size}>
            <span class="field-label">Resize size</span>
            <input type="number" min="1" bind:value={settingsDraft.resize_size} />
            {#if settingsErrors.resizeSize}<span class="field-error">{settingsErrors.resizeSize}</span>{/if}
          </label>
          <label title={settingsHelp.hist_bins}>
            <span class="field-label">Histogram bins</span>
            <input type="number" min="2" bind:value={settingsDraft.hist_bins} />
            {#if settingsErrors.histBins}<span class="field-error">{settingsErrors.histBins}</span>{/if}
          </label>
          <label title={settingsHelp.hist_method}>
            <span class="field-label">Histogram method</span>
            <select bind:value={settingsDraft.hist_method}>
              <option value="correlation">correlation</option>
              <option value="bhattacharyya">bhattacharyya</option>
            </select>
          </label>
          <label title={settingsHelp.alpha_threshold}>
            <span class="field-label">Alpha threshold</span>
            <input type="number" step="0.01" min="0" max="1" bind:value={settingsDraft.alpha_threshold} />
            {#if settingsErrors.alphaThreshold}<span class="field-error">{settingsErrors.alphaThreshold}</span>{/if}
          </label>
          <label title={settingsHelp.max_decode_dimension}>
            <span class="field-label">Max decode dimension (optional)</span>
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
      </details>

      <details class="settings-section" open>
        <summary>Advanced</summary>
        <div class="settings-grid">
          <label title={settingsHelp.hash_algorithm}>
            <span class="field-label">Hash algorithm</span>
            <select bind:value={settingsDraft.hash_algorithm}>
              <option value="sha256">sha256</option>
              <option value="sha1">sha1</option>
              <option value="md5">md5</option>
            </select>
          </label>
        </div>
        <p class="orb-note">{settingsHelp.orb_note}</p>
      </details>

      {#if saveMessage}
        <p class="save-ok">{saveMessage}</p>
      {/if}
      {#if loadError}
        <p class="save-error">{loadError}</p>
      {/if}
    </div>

    <div class="modal-footer">
      <button type="button" class="action-btn secondary" onclick={() => requestClose()}>Cancel</button>
      <button type="button" class="action-btn" onclick={() => onSave()}>Save</button>
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

  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.85rem 1rem;
    border-bottom: 1px solid var(--border-subtle);
  }

  .modal-header h2 {
    margin: 0;
    font-size: 1rem;
    font-weight: 600;
    color: var(--text-primary);
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
  }

  .presets-row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.4rem;
  }

  .presets-label {
    font-size: 0.76rem;
    color: var(--text-muted);
    font-weight: 600;
  }

  .preset-btn {
    border: 1px solid var(--border);
    border-radius: 999px;
    background: var(--bg-panel-alt);
    color: var(--text-secondary);
    font-size: 0.74rem;
    padding: 0.22rem 0.6rem;
    cursor: pointer;
  }

  .preset-btn:hover {
    border-color: var(--accent);
    color: var(--accent-hover);
  }

  .presets-hint {
    font-size: 0.72rem;
    color: var(--text-faint);
  }

  .preset-descriptions {
    margin: 0;
    padding: 0 0 0 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.28rem;
    font-size: 0.72rem;
    color: var(--text-muted);
    line-height: 1.35;
  }

  .preset-descriptions strong {
    color: var(--text-secondary);
    font-weight: 600;
  }

  .settings-section {
    border: 1px solid var(--border-subtle);
    border-radius: 8px;
    background: var(--bg-modal-section);
    padding: 0.55rem 0.65rem;
  }

  .settings-section summary {
    cursor: pointer;
    font-size: 0.84rem;
    font-weight: 600;
    color: var(--text-secondary);
    margin-bottom: 0.35rem;
  }

  .settings-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
    gap: 0.55rem;
    margin-top: 0.45rem;
  }

  .settings-grid label {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    font-size: 0.78rem;
    color: var(--text-secondary);
  }

  .field-label {
    font-weight: 520;
  }

  .settings-grid label input[type="checkbox"] {
    width: auto;
    align-self: flex-start;
  }

  .settings-grid label input[type="number"],
  .settings-grid label select {
    width: 100%;
    box-sizing: border-box;
    padding: 0.45rem 0.55rem;
    border-radius: 6px;
    border: 1px solid var(--border);
    background: var(--bg-input);
    color: var(--text-primary);
    font-size: 0.82rem;
  }

  .orb-note {
    margin: 0.45rem 0 0;
    font-size: 0.72rem;
    color: var(--text-faint);
  }

  .field-error {
    color: var(--error);
    font-size: 0.72rem;
    line-height: 1.2;
  }

  .save-ok {
    margin: 0;
    color: var(--success);
    font-size: 0.78rem;
  }

  .save-error {
    margin: 0;
    color: var(--error);
    font-size: 0.78rem;
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: 0.45rem;
    padding: 0.75rem 1rem 1rem;
    border-top: 1px solid var(--border-subtle);
  }

  .action-btn {
    padding: 0.45rem 0.85rem;
    border-radius: 6px;
    border: 1px solid var(--border-strong);
    background: var(--bg-button);
    color: var(--text-secondary);
    font-size: 0.82rem;
    font-weight: 500;
    cursor: pointer;
  }

  .action-btn:hover {
    border-color: var(--accent);
    color: var(--accent-hover);
  }

  .action-btn.secondary {
    border-color: var(--border);
    background: var(--bg-button-secondary);
    color: var(--text-tab);
  }
</style>

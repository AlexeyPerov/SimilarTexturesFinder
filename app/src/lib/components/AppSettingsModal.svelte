<script lang="ts">
  import type { AppTheme } from "$lib/types";
  import { trapFocus } from "$lib/modalFocus";

  type Props = {
    theme: AppTheme;
    onClose: () => void;
    onThemeChange: (theme: AppTheme) => void;
  };

  let { theme, onClose, onThemeChange }: Props = $props();
</script>

<div
  class="overlay"
  role="presentation"
  onclick={(e) => {
    if (e.target === e.currentTarget) onClose();
  }}
>
  <div
    class="modal"
    role="dialog"
    aria-modal="true"
    aria-labelledby="app-settings-title"
    use:trapFocus
  >
    <div class="modal-header">
      <h2 id="app-settings-title">App settings</h2>
      <button type="button" class="modal-close-btn" aria-label="Close app settings" onclick={() => onClose()}>
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <line x1="18" y1="6" x2="6" y2="18" />
          <line x1="6" y1="6" x2="18" y2="18" />
        </svg>
      </button>
    </div>

    <div class="modal-body">
      <div class="settings-section">
        <span class="section-label">Theme</span>
        <div class="theme-toggle" role="radiogroup" aria-label="Theme">
          <button
            type="button"
            class="theme-option"
            class:theme-option-active={theme === "dark"}
            role="radio"
            aria-checked={theme === "dark"}
            onclick={() => onThemeChange("dark")}
          >
            Dark
          </button>
          <button
            type="button"
            class="theme-option"
            class:theme-option-active={theme === "light"}
            role="radio"
            aria-checked={theme === "light"}
            onclick={() => onThemeChange("light")}
          >
            Light
          </button>
        </div>
      </div>
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
    width: min(28rem, 95vw);
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
  }

  .settings-section {
    display: flex;
    flex-direction: column;
    gap: 0.55rem;
  }

  .section-label {
    font-size: 0.76rem;
    color: var(--text-muted);
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }

  .theme-toggle {
    display: inline-flex;
    gap: 0.25rem;
    padding: 0.2rem;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--bg-panel-alt);
    align-self: flex-start;
  }

  .theme-option {
    border: 1px solid transparent;
    border-radius: 6px;
    background: transparent;
    color: var(--text-tab);
    font-size: 0.82rem;
    padding: 0.35rem 0.75rem;
    cursor: pointer;
  }

  .theme-option:hover {
    color: var(--text-primary);
  }

  .theme-option-active {
    background: var(--bg-tab-active);
    color: var(--text-primary);
    border-color: var(--border-strong);
  }
</style>

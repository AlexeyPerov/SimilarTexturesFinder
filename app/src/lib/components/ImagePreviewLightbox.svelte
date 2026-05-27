<script lang="ts">
  import ThumbnailImage from "$lib/components/ThumbnailImage.svelte";
  import { formatImageSize, fetchImagePreview, type ImageDimensions } from "$lib/imagePreview";
  import { trapFocus } from "$lib/modalFocus";
  import { toBaseName } from "$lib/groupUtils";

  type Props = {
    imagePath: string;
    maxPx?: number;
    dimensions?: ImageDimensions;
    onClose: () => void;
    onCopyPath: (path: string) => void;
  };

  let {
    imagePath,
    maxPx = 1024,
    dimensions,
    onClose,
    onCopyPath,
  }: Props = $props();

  let pathInput: HTMLInputElement | undefined = $state();
  let resolvedDimensions = $state<ImageDimensions | undefined>(undefined);

  $effect(() => {
    imagePath;
    resolvedDimensions = dimensions;
    if (dimensions != null) return;

    let active = true;
    void fetchImagePreview(imagePath, maxPx)
      .then((out) => {
        if (!active) return;
        resolvedDimensions = { width: out.width, height: out.height };
      })
      .catch(() => {
        // Size stays unavailable.
      });

    return () => {
      active = false;
    };
  });

  function onBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) onClose();
  }

  function onKeyDown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      e.stopPropagation();
      onClose();
    }
  }

  function selectPath() {
    pathInput?.select();
  }
</script>

<svelte:window onkeydown={onKeyDown} />

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="lightbox-overlay" role="presentation" onclick={onBackdropClick}>
  <div
    class="lightbox"
    role="dialog"
    aria-modal="true"
    aria-label={`Preview ${toBaseName(imagePath)}`}
    use:trapFocus
  >
    <header class="lightbox-header">
      <div class="lightbox-title">
        <span class="basename">{toBaseName(imagePath)}</span>
        <span class="size-label">{formatImageSize(resolvedDimensions)}</span>
      </div>
      <button type="button" class="lightbox-close-btn" aria-label="Close preview" onclick={() => onClose()}>
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <line x1="18" y1="6" x2="6" y2="18" />
          <line x1="6" y1="6" x2="18" y2="18" />
        </svg>
      </button>
    </header>

    <div class="lightbox-preview">
      <ThumbnailImage
        imagePath={imagePath}
        {maxPx}
        fit="contain"
        shellClass="lightbox-thumb"
        alt={imagePath}
      />
    </div>

    <div class="lightbox-path-row">
      <input
        bind:this={pathInput}
        readonly
        class="path-input"
        value={imagePath}
        aria-label="Image path"
        onclick={selectPath}
      />
      <button type="button" class="action-btn" onclick={() => onCopyPath(imagePath)}>Copy path</button>
    </div>
  </div>
</div>

<style>
  .lightbox-overlay {
    position: fixed;
    inset: 0;
    z-index: 300;
    background: var(--bg-overlay);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 1rem;
  }

  .lightbox {
    background: var(--bg-modal);
    border: 1px solid var(--border);
    border-radius: 12px;
    width: min(52rem, 96vw);
    max-height: 92vh;
    display: flex;
    flex-direction: column;
    box-shadow: var(--shadow-modal);
    overflow: hidden;
  }

  .lightbox-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 0.75rem;
    padding: 0.75rem 1rem;
    border-bottom: 1px solid var(--border-subtle);
  }

  .lightbox-title {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    gap: 0.45rem 0.65rem;
    min-width: 0;
  }

  .basename {
    font-size: 0.9rem;
    font-weight: 600;
    color: var(--text-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .size-label {
    font-size: 0.78rem;
    color: var(--text-muted);
    white-space: nowrap;
  }

  .lightbox-close-btn {
    padding: 0.3rem;
    border-radius: 4px;
    border: 1px solid transparent;
    background: transparent;
    color: var(--text-faint);
    cursor: pointer;
    flex-shrink: 0;
  }

  .lightbox-close-btn:hover {
    color: var(--accent-hover);
    border-color: var(--border-strong);
    background: var(--bg-button);
  }

  .lightbox-preview {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 12rem;
    max-height: min(70vh, 720px);
    overflow: hidden;
    padding: 0.75rem 1rem;
    background: var(--bg-card-inner);
  }

  .lightbox-preview :global(.lightbox-thumb) {
    width: 100%;
    height: 100%;
    max-height: min(68vh, 700px);
  }

  .lightbox-path-row {
    display: flex;
    gap: 0.5rem;
    padding: 0.75rem 1rem;
    border-top: 1px solid var(--border-subtle);
    align-items: center;
  }

  .path-input {
    flex: 1;
    min-width: 0;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--bg-console);
    color: var(--text-secondary);
    font-size: 0.78rem;
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    padding: 0.45rem 0.55rem;
    cursor: text;
  }

  .path-input:focus {
    outline: none;
    border-color: var(--accent);
  }

  .action-btn {
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--bg-panel-alt);
    color: var(--text-muted);
    font-size: 0.78rem;
    padding: 0.45rem 0.65rem;
    cursor: pointer;
    white-space: nowrap;
  }

  .action-btn:hover {
    border-color: var(--accent);
    color: var(--accent-hover);
  }
</style>

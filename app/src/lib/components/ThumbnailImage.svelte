<script lang="ts">
  import { convertFileSrc, invoke } from "@tauri-apps/api/core";

  type Props = {
    imagePath: string;
    alt?: string;
    maxPx?: number;
    class?: string;
    onOpenImage?: (path: string) => void;
  };

  let {
    imagePath,
    alt = imagePath,
    maxPx = 256,
    class: className = "",
    onOpenImage,
  }: Props = $props();

  let src = $state("");
  let failed = $state(false);
  let loading = $state(true);

  $effect(() => {
    const path = imagePath;
    const px = maxPx;
    let active = true;

    loading = true;
    failed = false;
    src = "";

    void (async () => {
      try {
        const out = await invoke<{ path: string }>("get_image_preview", {
          request: { path, maxPx: px },
        });
        if (!active) return;
        src = convertFileSrc(out.path);
      } catch {
        if (!active) return;
        failed = true;
        src = convertFileSrc(path);
      } finally {
        if (active) loading = false;
      }
    })();

    return () => {
      active = false;
    };
  });
</script>

<div class="thumb-shell" class:loading>
  {#if loading}
    <div class="thumb-placeholder" aria-hidden="true">
      <div class="thumb-spinner"></div>
    </div>
  {/if}
  <img
    class={className}
    {src}
    {alt}
    loading="lazy"
    class:fallback={failed}
    class:openable={onOpenImage != null}
    class:loaded={!loading && src !== ""}
    ondblclick={(e) => {
      if (!onOpenImage) return;
      e.stopPropagation();
      e.preventDefault();
      onOpenImage(imagePath);
    }}
  />
</div>

<style>
  .thumb-shell {
    position: relative;
    width: 100%;
    height: 100%;
  }

  .thumb-placeholder {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--bg-thumb);
    z-index: 1;
  }

  .thumb-spinner {
    width: 1.1rem;
    height: 1.1rem;
    border: 2px solid var(--border);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: thumb-spin 0.7s linear infinite;
  }

  @keyframes thumb-spin {
    to {
      transform: rotate(360deg);
    }
  }

  img {
    display: block;
    width: 100%;
    object-fit: cover;
    background: var(--bg-thumb);
    opacity: 0;
    transition: opacity 0.15s ease;
  }

  img.loaded {
    opacity: 1;
  }

  img.fallback {
    opacity: 0.85;
  }

  img.openable {
    cursor: pointer;
  }
</style>

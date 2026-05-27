<script lang="ts">
  import { convertFileSrc } from "@tauri-apps/api/core";
  import { fetchImagePreview, type ImageDimensions } from "$lib/imagePreview";

  type Props = {
    imagePath: string;
    alt?: string;
    maxPx?: number;
    fit?: "cover" | "contain";
    class?: string;
    shellClass?: string;
    onOpenImage?: (path: string) => void;
    onDimensions?: (dims: ImageDimensions) => void;
  };

  let {
    imagePath,
    alt = imagePath,
    maxPx = 256,
    fit = "cover",
    class: className = "",
    shellClass = "",
    onOpenImage,
    onDimensions,
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
        const out = await fetchImagePreview(path, px);
        if (!active) return;
        src = convertFileSrc(out.path);
        onDimensions?.({ width: out.width, height: out.height });
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

<div class="thumb-shell {shellClass}" class:loading>
  {#if loading}
    <div class="thumb-placeholder" aria-hidden="true">
      <div class="thumb-spinner"></div>
    </div>
  {/if}
  <img
    class={className}
    class:contain={fit === "contain"}
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
    height: var(--thumb-height, auto);
  }

  .thumb-shell.lightbox-thumb {
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
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
    height: 100%;
    object-fit: cover;
    background: var(--bg-thumb);
    opacity: 0;
    transition: opacity 0.15s ease;
  }

  img.contain {
    object-fit: contain;
  }

  .thumb-shell.lightbox-thumb img {
    width: auto;
    height: auto;
    max-width: 100%;
    max-height: 100%;
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

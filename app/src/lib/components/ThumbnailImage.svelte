<script lang="ts">
  import { convertFileSrc, invoke } from "@tauri-apps/api/core";

  type Props = {
    imagePath: string;
    alt?: string;
    maxPx?: number;
    class?: string;
  };

  let { imagePath, alt = imagePath, maxPx = 256, class: className = "" }: Props = $props();

  let src = $state("");
  let failed = $state(false);

  async function loadPreview() {
    failed = false;
    src = "";
    try {
      const out = await invoke<{ path: string }>("get_image_preview", {
        request: { path: imagePath, maxPx },
      });
      src = convertFileSrc(out.path);
    } catch {
      failed = true;
      src = convertFileSrc(imagePath);
    }
  }

  $effect(() => {
    imagePath;
    maxPx;
    void loadPreview();
  });
</script>

<img class={className} {src} {alt} loading="lazy" class:fallback={failed} />

<style>
  img {
    display: block;
    width: 100%;
    object-fit: cover;
    background: #101116;
  }

  img.fallback {
    opacity: 0.85;
  }
</style>

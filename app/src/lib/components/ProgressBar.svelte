<script lang="ts">
  import type { ProgressBarMode } from "$lib/scanProgress";

  type Props = {
    mode: ProgressBarMode;
    processed?: number;
    total?: number;
  };

  let { mode, processed = 0, total = 0 }: Props = $props();

  let ratio = $derived(
    mode === "determinate" && total > 0 ? Math.min(100, (processed / total) * 100) : 0,
  );
</script>

{#if mode !== "hidden"}
  <div class="progress-wrap" role="progressbar" aria-valuemin={0} aria-valuemax={100} aria-valuenow={mode === "determinate" ? Math.round(ratio) : undefined}>
    <div class="progress-track">
      {#if mode === "determinate"}
        <div class="progress-fill determinate" style:width="{ratio}%"></div>
      {:else}
        <div class="progress-fill indeterminate"></div>
      {/if}
    </div>
    {#if mode === "determinate" && total > 0}
      <span class="progress-meta">{processed} / {total}</span>
    {/if}
  </div>
{/if}

<style>
  .progress-wrap {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
  }

  .progress-track {
    height: 0.45rem;
    border-radius: 999px;
    background: #1a1b21;
    border: 1px solid #3f4150;
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    border-radius: 999px;
    background: #5c7cfa;
  }

  .progress-fill.determinate {
    transition: width 0.18s ease;
  }

  .progress-fill.indeterminate {
    width: 35%;
    animation: indeterminate 1.1s ease-in-out infinite;
  }

  @keyframes indeterminate {
    0% {
      transform: translateX(-120%);
    }
    100% {
      transform: translateX(320%);
    }
  }

  .progress-meta {
    font-size: 0.72rem;
    color: #aeb1bf;
  }
</style>

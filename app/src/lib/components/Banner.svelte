<script lang="ts">
  import type { TabBanner } from "$lib/types";

  type Props = {
    banner: TabBanner;
    onDismiss?: () => void;
  };

  let { banner, onDismiss }: Props = $props();
</script>

{#if banner}
  <div class="banner banner-{banner.kind}" role="status">
    <span class="banner-message">{banner.message}</span>
    {#if onDismiss}
      <button type="button" class="banner-dismiss" aria-label="Dismiss" onclick={() => onDismiss?.()}>
        ×
      </button>
    {/if}
  </div>
{/if}

<style>
  .banner {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 0.6rem;
    padding: 0.55rem 0.75rem;
    border-radius: 8px;
    font-size: 0.82rem;
    line-height: 1.35;
  }

  .banner-success {
    border: 1px solid var(--border-success);
    background: var(--success-banner-bg);
    color: var(--success-banner-text);
  }

  .banner-error {
    border: 1px solid var(--border-error);
    background: var(--error-banner-bg);
    color: var(--error-banner-text);
  }

  .banner-message {
    flex: 1;
    min-width: 0;
    word-break: break-word;
  }

  .banner-dismiss {
    flex-shrink: 0;
    border: 0;
    background: transparent;
    color: inherit;
    font-size: 1.1rem;
    line-height: 1;
    cursor: pointer;
    padding: 0 0.15rem;
    opacity: 0.85;
  }

  .banner-dismiss:hover {
    opacity: 1;
  }
</style>

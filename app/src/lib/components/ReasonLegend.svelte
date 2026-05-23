<script lang="ts">
  import { reasonLegendEntries } from "$lib/settingsHelp";
</script>

<details class="reason-legend">
  <summary>What do reason tags mean?</summary>
  <ul>
    {#each reasonLegendEntries as entry (entry.kind)}
      <li>
        <span class="legend-label">{entry.label}</span>
        <span class="legend-desc">{entry.description}</span>
      </li>
    {/each}
  </ul>
</details>

<style>
  .reason-legend {
    border: 1px solid #3f4150;
    border-radius: 8px;
    background: #1e1f26;
    padding: 0.45rem 0.6rem;
    font-size: 0.76rem;
    color: #aeb1bf;
  }

  .reason-legend summary {
    cursor: pointer;
    color: #d7d8e0;
    font-weight: 520;
  }

  .reason-legend ul {
    margin: 0.45rem 0 0;
    padding: 0 0 0 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.28rem;
  }

  .legend-label {
    font-weight: 600;
    color: #dce3ff;
    margin-right: 0.35rem;
  }
</style>

<script lang="ts">
  import type { Collection, ExportFormat, ExportJob, ExportRequest } from './api';

  export let collections: Collection[] = [];
  export let formats: ExportFormat[] = [];
  export let exporting = false;
  export let job: ExportJob | null = null;
  export let error = '';
  export let onExport: (request: ExportRequest) => void;

  let selectedCollection = '';
  let selectedFormat: ExportFormat = 'json';
  let includeGraph = false;

  $: if (collections.length > 0 && !collections.some((collection) => collection.name === selectedCollection)) {
    selectedCollection = collections[0].name;
  }
  $: if (formats.length > 0 && !formats.includes(selectedFormat)) {
    selectedFormat = formats[0];
  }

  function submit(): void {
    onExport({ collection: selectedCollection, format: selectedFormat, includeGraph });
  }
</script>

<section class="panel" aria-labelledby="export-title" aria-busy={exporting}>
  <div class="section-heading">
    <div>
      <p class="eyebrow">Export configurator</p>
      <h2 id="export-title">Create a portable artifact</h2>
    </div>
    {#if job}
      <span class="status">{job.progress}% complete</span>
    {/if}
  </div>

  {#if collections.length === 0}
    <p class="empty">Test an Oracle connection to load exportable collections.</p>
  {:else}
    <form on:submit|preventDefault={submit} class="export-form">
      <label for="collection">Collection</label>
      <select id="collection" bind:value={selectedCollection}>
        {#each collections as collection}
          <option value={collection.name}>{collection.name} · {collection.rowCount.toLocaleString()} rows</option>
        {/each}
      </select>

      <fieldset>
        <legend>Format</legend>
        <div class="format-options">
          {#each formats as format}
            <label class="format-option">
              <input type="radio" name="format" value={format} bind:group={selectedFormat} />
              <span>{format === 'markdown' ? 'Markdown' : format.toUpperCase()}</span>
            </label>
          {/each}
        </div>
      </fieldset>

      <label class="checkbox" for="include-graph">
        <input id="include-graph" type="checkbox" bind:checked={includeGraph} />
        <span>Include relationship graph when available</span>
      </label>

      <button type="submit" disabled={exporting}>{exporting ? 'Creating export…' : 'Create export'}</button>
    </form>
  {/if}

  {#if job}
    <div class="progress" aria-live="polite">
      <span>Job {job.id}</span>
      <progress max="100" value={job.progress}>{job.progress}%</progress>
      {#if job.status === 'completed'}
        <a href={job.downloadUrl} download={job.filename}>Download {job.filename ?? 'export'}</a>
      {/if}
    </div>
  {/if}
  {#if error}<p class="error" role="alert">{error}</p>{/if}
</section>

<style>
  .panel { padding: 1.5rem; }
  .section-heading { display: flex; gap: 1rem; align-items: center; justify-content: space-between; }
  .eyebrow { margin: 0 0 .35rem; color: var(--accent); font: 700 .72rem/1 var(--mono); letter-spacing: .12em; text-transform: uppercase; }
  h2 { margin: 0; font-size: 1.15rem; letter-spacing: -.015em; }
  .status { border: 1px solid var(--border); border-radius: 999px; color: var(--muted); font: 600 .72rem/1 var(--mono); padding: .28rem .55rem; }
  .empty, .error { margin: 1.3rem 0 0; color: var(--muted); }
  .error { color: var(--danger); }
  .export-form { display: grid; gap: 1rem; margin-top: 1.35rem; }
  select { width: 100%; min-height: 44px; border: 1px solid var(--border); border-radius: .45rem; background: var(--field); color: var(--text); padding: .65rem .75rem; }
  label, legend { color: var(--muted); font: 600 .78rem/1 var(--mono); }
  fieldset { display: grid; gap: .55rem; border: 0; margin: 0; padding: 0; }
  .format-options { display: flex; flex-wrap: wrap; gap: .5rem; }
  .format-option { display: inline-flex; min-height: 44px; gap: .4rem; align-items: center; border: 1px solid var(--border); border-radius: .45rem; color: var(--text); cursor: pointer; padding: 0 .75rem; }
  .checkbox { display: flex; min-height: 44px; gap: .65rem; align-items: center; color: var(--text); cursor: pointer; font-family: var(--sans); }
  button { min-height: 44px; border: 0; border-radius: .45rem; background: var(--accent); color: var(--accent-ink); cursor: pointer; font-weight: 700; padding: .7rem 1rem; }
  button:hover:not(:disabled) { background: var(--accent-hover); }
  button:disabled { cursor: wait; opacity: .65; }
  .progress { display: grid; gap: .55rem; border-top: 1px solid var(--border); margin-top: 1.3rem; padding-top: 1rem; color: var(--muted); font: .78rem/1 var(--mono); }
  progress { accent-color: var(--accent); width: 100%; }
  a { color: var(--accent); font-family: var(--sans); font-weight: 700; }
  @media (max-width: 650px) { .section-heading { align-items: flex-start; flex-direction: column; } }
</style>

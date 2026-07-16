<script lang="ts">
  import type { ExportJob } from './api';

  export let jobs: ExportJob[] = [];
  export let loading = false;
  export let error = '';
</script>

<section class="panel" aria-labelledby="history-title" aria-busy={loading}>
  <div class="section-heading">
    <div>
      <p class="eyebrow">Export history</p>
      <h2 id="history-title">Recent artifacts</h2>
    </div>
    <span class="count">{jobs.length} {jobs.length === 1 ? 'export' : 'exports'}</span>
  </div>

  {#if loading}
    <p class="empty">Loading export history…</p>
  {:else if error}
    <p class="error" role="alert">{error}</p>
  {:else if jobs.length === 0}
    <p class="empty">Artifacts created during this session will appear here.</p>
  {:else}
    <ul class="history-list" aria-label="Recent export artifacts">
      {#each jobs as job (job.id)}
        <li>
          <div>
            <strong>{job.collection}</strong>
            <span>{job.format.toUpperCase()} · {job.status} · {job.progress}%</span>
          </div>
          <a href={job.downloadUrl} download={job.filename} aria-label={`Download ${job.filename ?? job.collection}`}>Download</a>
        </li>
      {/each}
    </ul>
  {/if}
</section>

<style>
  .panel { padding: 1.5rem; }
  .section-heading { display: flex; gap: 1rem; align-items: center; justify-content: space-between; }
  .eyebrow { margin: 0 0 .35rem; color: var(--accent); font: 700 .72rem/1 var(--mono); letter-spacing: .12em; text-transform: uppercase; }
  h2 { margin: 0; font-size: 1.15rem; letter-spacing: -.015em; }
  .count { color: var(--muted); font: .75rem/1 var(--mono); }
  .empty, .error { margin: 1.3rem 0 0; color: var(--muted); }
  .error { color: var(--danger); }
  .history-list { display: grid; gap: .35rem; list-style: none; margin: 1.1rem 0 0; padding: 0; }
  li { display: flex; gap: 1rem; align-items: center; justify-content: space-between; border-top: 1px solid var(--border); padding: .9rem 0; }
  strong, span { display: block; }
  strong { color: var(--text); font-size: .92rem; }
  span { color: var(--muted); font: .72rem/1.5 var(--mono); }
  a { min-height: 44px; display: inline-flex; align-items: center; color: var(--accent); font-size: .85rem; font-weight: 700; }
</style>

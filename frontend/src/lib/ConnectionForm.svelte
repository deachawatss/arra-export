<script lang="ts">
  import type { ConnectionStatus } from './api';

  export let url: string;
  export let testing = false;
  export let connection: ConnectionStatus | null = null;
  export let error = '';
  export let onTest: () => void;
</script>

<section class="panel" aria-labelledby="connection-title">
  <div class="section-heading">
    <div>
      <p class="eyebrow">Oracle connection</p>
      <h2 id="connection-title">Choose the knowledge source</h2>
    </div>
    {#if connection?.ok}
      <span class="status success">Connected</span>
    {:else}
      <span class="status neutral">Not tested</span>
    {/if}
  </div>

  <form on:submit|preventDefault={onTest} class="connection-form">
    <label for="oracle-url">Oracle URL</label>
    <div class="url-controls">
      <input id="oracle-url" name="oracle-url" type="url" bind:value={url} required autocomplete="url" placeholder="http://localhost:47778" />
      <button type="submit" disabled={testing}>{testing ? 'Testing…' : 'Test connection'}</button>
    </div>
  </form>

  <div class:message-error={error} class:message-success={connection?.ok} class="connection-message" aria-live="polite">
    {#if error}
      {error}
    {:else if connection?.ok}
      {connection.status} · {connection.totalRows.toLocaleString()} records across {connection.collections.length} collections
    {:else}
      Test a running Oracle before choosing an export.
    {/if}
  </div>
</section>

<style>
  .panel { padding: 1.5rem; }
  .section-heading, .url-controls { display: flex; gap: 1rem; align-items: center; justify-content: space-between; }
  .eyebrow { margin: 0 0 .35rem; color: var(--accent); font: 700 .72rem/1 var(--mono); letter-spacing: .12em; text-transform: uppercase; }
  h2 { margin: 0; font-size: 1.15rem; letter-spacing: -.015em; }
  .status { border: 1px solid var(--border); border-radius: 999px; padding: .28rem .55rem; color: var(--muted); font: 600 .72rem/1 var(--mono); }
  .status.success { border-color: var(--success-border); background: var(--success-bg); color: var(--success); }
  .connection-form { display: grid; gap: .5rem; margin-top: 1.4rem; }
  label { color: var(--muted); font: 600 .78rem/1 var(--mono); }
  input { min-width: 0; flex: 1; border: 1px solid var(--border); border-radius: .45rem; background: var(--field); color: var(--text); font: .95rem/1.2 var(--mono); padding: .75rem .85rem; }
  input:focus-visible { outline: 2px solid var(--accent); outline-offset: 2px; }
  button { min-height: 44px; border: 0; border-radius: .45rem; background: var(--accent); color: var(--accent-ink); cursor: pointer; font-weight: 700; padding: .7rem 1rem; }
  button:hover:not(:disabled) { background: var(--accent-hover); }
  button:disabled { cursor: wait; opacity: .65; }
  .connection-message { margin: 1rem 0 0; color: var(--muted); font-size: .9rem; }
  .connection-message.message-success { color: var(--success); }
  .connection-message.message-error { color: var(--danger); }
  @media (max-width: 650px) { .section-heading, .url-controls { align-items: stretch; flex-direction: column; } }
</style>

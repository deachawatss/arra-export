<script lang="ts">
  import { onMount } from 'svelte';
  import ConnectionForm from '../lib/ConnectionForm.svelte';
  import ExportConfigurator from '../lib/ExportConfigurator.svelte';
  import ExportHistory from '../lib/ExportHistory.svelte';
  import {
    createExport,
    getCollections,
    getHistory,
    testConnection,
    type CollectionsResponse,
    type ConnectionStatus,
    type ExportJob,
    type ExportRequest
  } from '../lib/api';

  let oracleUrl = 'http://localhost:47778';
  let connection: ConnectionStatus | null = null;
  let collections: CollectionsResponse = { collections: [], formats: [] };
  let history: ExportJob[] = [];
  let currentJob: ExportJob | null = null;
  let testing = false;
  let loadingHistory = true;
  let exporting = false;
  let connectionError = '';
  let exportError = '';
  let historyError = '';

  onMount(() => {
    void loadHistory();
  });

  async function loadHistory(): Promise<void> {
    loadingHistory = true;
    historyError = '';
    try {
      history = await getHistory();
    } catch (error) {
      historyError = error instanceof Error ? error.message : 'The export history could not be loaded.';
    } finally {
      loadingHistory = false;
    }
  }

  async function testOracle(): Promise<void> {
    testing = true;
    connectionError = '';
    exportError = '';
    try {
      connection = await testConnection(oracleUrl);
      if (!connection.ok) {
        connectionError = 'Oracle did not confirm a usable export connection.';
        return;
      }
      collections = await getCollections();
    } catch (error) {
      connection = null;
      collections = { collections: [], formats: [] };
      connectionError = error instanceof Error ? error.message : 'The Oracle connection could not be tested.';
    } finally {
      testing = false;
    }
  }

  async function submitExport(request: ExportRequest): Promise<void> {
    exporting = true;
    exportError = '';
    try {
      currentJob = await createExport(request);
      history = [currentJob, ...history.filter((job) => job.id !== currentJob?.id)];
    } catch (error) {
      exportError = error instanceof Error ? error.message : 'The export could not be created.';
    } finally {
      exporting = false;
    }
  }
</script>

<svelte:head>
  <title>Arra Export</title>
  <meta name="description" content="Export a portable snapshot of Oracle knowledge." />
</svelte:head>

<a class="skip-link" href="#workspace">Skip to export workspace</a>

<main id="workspace">
  <header class="masthead">
    <div class="brand" aria-label="Arra Export">
      <span class="brand-mark" aria-hidden="true">A</span>
      <span>arra export</span>
    </div>
    <p class="masthead-note">Single Oracle · portable knowledge artifacts</p>
  </header>

  <section class="intro" aria-labelledby="page-title">
    <p class="eyebrow">Knowledge portability</p>
    <h1 id="page-title">Export the Oracle you can inspect, move, and keep.</h1>
    <p>Connect one running Oracle, select a collection, and generate an artifact in a format suited to your next system.</p>
  </section>

  <section class="workspace-grid" aria-label="Export workspace">
    <ConnectionForm bind:url={oracleUrl} {testing} {connection} error={connectionError} onTest={testOracle} />
    <ExportConfigurator
      collections={collections.collections}
      formats={collections.formats}
      {exporting}
      job={currentJob}
      error={exportError}
      onExport={submitExport}
    />
  </section>

  <ExportHistory jobs={history} loading={loadingHistory} error={historyError} />
</main>

<style>
  :global(*) { box-sizing: border-box; }
  :global(html) { background: var(--page); }
  :global(body) { margin: 0; min-width: 320px; background: var(--page); color: var(--text); font-family: var(--sans); }
  :global(button), :global(input), :global(select) { font: inherit; }
  :global(:focus-visible) { outline: 2px solid var(--accent); outline-offset: 3px; }

  :global(:root) {
    --page: oklch(0.13 0.02 265);
    --panel: oklch(0.21 0.02 265);
    --field: oklch(0.18 0.02 265);
    --border: oklch(0.34 0.02 265);
    --text: oklch(0.96 0.01 257);
    --muted: oklch(0.71 0.04 257);
    --accent: oklch(0.82 0.13 178);
    --accent-hover: oklch(0.88 0.1 178);
    --accent-ink: oklch(0.13 0.02 265);
    --success: oklch(0.85 0.13 156);
    --success-bg: oklch(0.5 0.1 156 / 0.18);
    --success-border: oklch(0.85 0.13 156 / 0.36);
    --danger: oklch(0.82 0.15 25);
    --sans: "Fira Sans", Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
    --mono: "Fira Code", "SFMono-Regular", Consolas, monospace;
  }

  .skip-link { position: fixed; z-index: 2; top: 1rem; left: 1rem; transform: translateY(-150%); background: var(--accent); border-radius: .35rem; color: var(--accent-ink); font-weight: 700; padding: .6rem .85rem; }
  .skip-link:focus { transform: translateY(0); }
  main { width: min(1280px, calc(100% - 3rem)); margin: 0 auto; padding: 1.5rem 0 3.5rem; }
  .masthead { display: flex; gap: 1rem; align-items: center; justify-content: space-between; border-bottom: 1px solid var(--border); padding-bottom: 1.25rem; }
  .brand { display: flex; gap: .55rem; align-items: center; color: var(--text); font: 700 .95rem/1 var(--mono); letter-spacing: -.03em; }
  .brand-mark { display: grid; width: 1.9rem; height: 1.9rem; place-items: center; border: 1px solid var(--accent); border-radius: .45rem; color: var(--accent); }
  .masthead-note { margin: 0; color: var(--muted); font: .76rem/1.4 var(--mono); text-align: right; }
  .intro { max-width: 56rem; padding: 4rem 0 2.5rem; }
  .eyebrow { margin: 0 0 .75rem; color: var(--accent); font: 700 .72rem/1 var(--mono); letter-spacing: .14em; text-transform: uppercase; }
  h1 { max-width: 18ch; margin: 0; font-size: clamp(2.35rem, 5vw, 4.8rem); line-height: .98; letter-spacing: -.065em; }
  .intro > p:last-child { max-width: 43rem; color: var(--muted); font-size: 1.08rem; line-height: 1.65; }
  .workspace-grid { display: grid; grid-template-columns: minmax(0, 1fr) minmax(0, 1.15fr); gap: 1rem; margin-bottom: 1rem; }
  :global(.panel) { background: var(--panel); border: 1px solid var(--border); border-radius: .7rem; }
  @media (max-width: 900px) { .workspace-grid { grid-template-columns: 1fr; } }
  @media (max-width: 650px) { main { width: min(100% - 2rem, 1280px); } .masthead { align-items: flex-start; flex-direction: column; } .masthead-note { text-align: left; } .intro { padding-top: 2.8rem; } }
  @media (prefers-reduced-motion: reduce) { *, *::before, *::after { scroll-behavior: auto !important; transition-duration: .01ms !important; animation-duration: .01ms !important; animation-iteration-count: 1 !important; } }
</style>

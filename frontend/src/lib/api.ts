export type ExportFormat = 'json' | 'csv' | 'markdown' | 'jsonl';

export interface Collection {
  name: string;
  rowCount: number;
}

export interface ConnectionStatus {
  ok: boolean;
  status: string;
  collections: Collection[];
  totalRows: number;
  url?: string;
}

export interface CollectionsResponse {
  collections: Collection[];
  formats: ExportFormat[];
}

export interface ExportRequest {
  collection: string;
  format: ExportFormat;
  includeGraph: boolean;
}

export interface ExportJob {
  id: string;
  collection: string;
  format: ExportFormat;
  includeGraph: boolean;
  status: string;
  progress: number;
  filename?: string;
  createdAt?: string;
  downloadUrl: string;
}

type JsonValue = string | number | boolean | null | JsonValue[] | { [key: string]: JsonValue };
type JsonObject = { [key: string]: JsonValue };

function isObject(value: JsonValue): value is JsonObject {
  return !Array.isArray(value) && value !== null && typeof value === 'object';
}

function requiredString(object: JsonObject, key: string): string {
  const value = object[key];
  if (typeof value !== 'string' || value.length === 0) {
    throw new Error(`The API response is missing ${key}.`);
  }
  return value;
}

function optionalString(object: JsonObject, key: string): string | undefined {
  const value = object[key];
  return typeof value === 'string' ? value : undefined;
}

function numberValue(object: JsonObject, key: string, fallback = 0): number {
  const value = object[key];
  return typeof value === 'number' && Number.isFinite(value) ? value : fallback;
}

function collection(value: JsonValue): Collection {
  if (!isObject(value)) {
    throw new Error('The API returned an invalid collection.');
  }
  return { name: requiredString(value, 'name'), rowCount: numberValue(value, 'rowCount') };
}

function exportFormat(value: string): ExportFormat | null {
  return value === 'json' || value === 'csv' || value === 'markdown' || value === 'jsonl' ? value : null;
}

function exportJob(value: JsonValue): ExportJob {
  if (!isObject(value)) {
    throw new Error('The API returned an invalid export job.');
  }
  const format = exportFormat(requiredString(value, 'format'));
  if (format === null) {
    throw new Error('The API returned an unsupported export format.');
  }
  return {
    id: requiredString(value, 'id'),
    collection: requiredString(value, 'collection'),
    format,
    includeGraph: value.includeGraph === true,
    status: requiredString(value, 'status'),
    progress: numberValue(value, 'progress'),
    filename: optionalString(value, 'filename'),
    createdAt: optionalString(value, 'createdAt'),
    downloadUrl: requiredString(value, 'downloadUrl')
  };
}

async function readResponse(response: Response): Promise<JsonObject> {
  const payload: JsonValue = await response.json();
  if (!isObject(payload)) {
    throw new Error('The API returned an invalid JSON response.');
  }
  if (!response.ok) {
    throw new Error(optionalString(payload, 'error') ?? `Request failed with status ${response.status}.`);
  }
  return payload;
}

export async function testConnection(url: string): Promise<ConnectionStatus> {
  const response = await fetch('/api/test-connection', {
    method: 'POST',
    headers: { 'content-type': 'application/json', accept: 'application/json' },
    body: JSON.stringify({ url })
  });
  const payload = await readResponse(response);
  const collections = Array.isArray(payload.collections) ? payload.collections.map(collection) : [];
  return {
    ok: payload.ok === true,
    status: requiredString(payload, 'status'),
    collections,
    totalRows: numberValue(payload, 'totalRows'),
    url: optionalString(payload, 'url')
  };
}

export async function getCollections(): Promise<CollectionsResponse> {
  const payload = await readResponse(await fetch('/api/collections', { headers: { accept: 'application/json' } }));
  const formats = Array.isArray(payload.formats)
    ? payload.formats.flatMap((value) => {
        const format = typeof value === 'string' ? exportFormat(value) : null;
        return format === null ? [] : [format];
      })
    : [];
  return {
    collections: Array.isArray(payload.collections) ? payload.collections.map(collection) : [],
    formats
  };
}

export async function createExport(request: ExportRequest): Promise<ExportJob> {
  const response = await fetch('/api/export', {
    method: 'POST',
    headers: { 'content-type': 'application/json', accept: 'application/json' },
    body: JSON.stringify(request)
  });
  return exportJob(await readResponse(response));
}

export async function getHistory(): Promise<ExportJob[]> {
  const payload: JsonValue = await (await fetch('/api/export/history', { headers: { accept: 'application/json' } })).json();
  if (!Array.isArray(payload)) {
    throw new Error('The API returned an invalid export history.');
  }
  return payload.map(exportJob);
}

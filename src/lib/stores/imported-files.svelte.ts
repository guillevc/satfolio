// TODO: persist to SQLite — list resets on app restart

import { SvelteDate } from "svelte/reactivity";
import type { TradesSummary } from "$lib/types/bindings";

export interface ImportedFile {
  id: string;
  filename: string;
  path: string;
  summary: TradesSummary;
  importedAt: Date;
}

export const importedFiles = $state<{ list: ImportedFile[] }>({ list: [] });

export function addImportedFile(
  path: string,
  summary: TradesSummary,
): ImportedFile {
  const filename = path.split("/").pop() ?? path;
  const file: ImportedFile = {
    id: crypto.randomUUID(),
    filename,
    path,
    summary,
    importedAt: new SvelteDate(),
  };
  importedFiles.list.push(file);
  return file;
}

export function removeImportedFile(id: string): void {
  // In-memory only — full cascade deletion (trades + DB row) comes with backend work
  importedFiles.list = importedFiles.list.filter((f) => f.id !== id);
}

/** Temporary: matches by filename. Will be replaced with SHA-256 hash when backend supports it. */
export function isFilenameDuplicate(path: string): boolean {
  const filename = path.split("/").pop() ?? path;
  return importedFiles.list.some((f) => f.filename === filename);
}

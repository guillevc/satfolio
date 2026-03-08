import { listImports, removeImport } from "$lib/api";
import type { ImportRecord } from "$lib/types/bindings";

export const importedFiles = $state<{
  list: ImportRecord[];
  loading: boolean;
}>({ list: [], loading: false });

export async function loadImportedFiles(): Promise<void> {
  importedFiles.loading = true;
  try {
    importedFiles.list = await listImports();
  } catch (e) {
    console.error("Failed to load imports:", e);
  } finally {
    importedFiles.loading = false;
  }
}

export function addImport(record: ImportRecord): void {
  importedFiles.list = [record, ...importedFiles.list];
}

export async function deleteImport(id: number): Promise<void> {
  await removeImport(id);
  importedFiles.list = importedFiles.list.filter((f) => f.id !== id);
}

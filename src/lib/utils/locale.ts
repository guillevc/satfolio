import { locale } from "@tauri-apps/plugin-os";

/** OS locale BCP-47 tag (e.g. "en-ES"). Falls back to browser language. */
export let systemLocale: string = navigator.language;

/** Call once before mount(). Reads the real OS locale via Tauri plugin. */
export async function initLocale(): Promise<void> {
  try {
    const osLocale = await locale();
    if (osLocale) systemLocale = osLocale;
  } catch (e) {
    console.warn("Failed to read OS locale, using navigator fallback:", e);
  }
}

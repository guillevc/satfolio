import { locale } from "@tauri-apps/plugin-os";

/** Extract ISO 3166-1 alpha-2 region from a BCP-47 tag (handles script subtags). */
function extractRegion(bcp47: string): string | undefined {
  return bcp47.split("-").find((p) => /^[A-Z]{2}$/.test(p));
}

/** Build a region-native locale for Intl.NumberFormat.
 *  en-ES → es-ES so NumberFormat uses Spanish grouping / symbol placement.
 *  Falls back to the original tag when the candidate isn't supported
 *  (e.g. en-US → "us-US" unsupported → stays en-US). */
function toFormattingLocale(bcp47: string): string {
  const region = extractRegion(bcp47);
  if (!region) return bcp47;
  const candidate = `${region.toLowerCase()}-${region}`;
  return Intl.NumberFormat.supportedLocalesOf([candidate]).length > 0
    ? candidate
    : bcp47;
}

/** Raw OS locale — for date formatting and display (keeps English labels). */
export let systemLocale: string = navigator.language;

/** Region-native locale — for Intl.NumberFormat (respects regional conventions). */
export let formattingLocale: string = toFormattingLocale(navigator.language);

/** Call once before mount(). Reads the real OS locale via Tauri plugin. */
export async function initLocale(): Promise<void> {
  try {
    const osLocale = await locale();
    if (osLocale) {
      systemLocale = osLocale;
      formattingLocale = toFormattingLocale(osLocale);
    }
  } catch {
    // keep navigator fallback
  }
}

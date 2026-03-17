const STORAGE_KEY = "satfolio:quote";
export type QuoteCurrency = "EUR" | "USD" | "GBP";

const stored =
  typeof localStorage !== "undefined" &&
  typeof localStorage.getItem === "function"
    ? localStorage.getItem(STORAGE_KEY)
    : null;
const initial: QuoteCurrency =
  stored === "USD" || stored === "GBP" ? stored : "EUR";

let quote = $state<QuoteCurrency>(initial);

export function getQuote(): QuoteCurrency {
  return quote;
}

export function setQuote(v: QuoteCurrency): void {
  quote = v;
  if (
    typeof localStorage !== "undefined" &&
    typeof localStorage.setItem === "function"
  ) {
    localStorage.setItem(STORAGE_KEY, v);
  }
}

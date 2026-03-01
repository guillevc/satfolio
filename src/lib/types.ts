export type View = "dashboard" | "trades" | "import" | "settings";

export const viewTitles: Record<View, string> = {
  dashboard: "Dashboard",
  trades: "Trades",
  import: "Import",
  settings: "Settings",
};

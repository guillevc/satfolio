export type View = "dashboard" | "trades" | "simulator" | "settings";

export const viewTitles: Record<View, string> = {
  dashboard: "Dashboard",
  trades: "Trades",
  simulator: "Simulator",
  settings: "Settings",
};

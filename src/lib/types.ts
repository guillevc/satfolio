export type View = 'dashboard' | 'ledger' | 'simulator' | 'settings';

export const viewTitles: Record<View, string> = {
	dashboard: 'Dashboard',
	ledger: 'Ledger',
	simulator: 'Simulator',
	settings: 'Settings',
};

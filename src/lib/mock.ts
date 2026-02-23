export interface MockTrade {
	type: 'buy' | 'sell';
	amount_btc: number;
	price_usd: number;
	total_usd: number;
	date: string;
	fee_usd: number;
}

export interface MockPositionSummary {
	break_even_price: number;
	total_held_btc: number;
	total_invested_usd: number;
	total_proceeds_usd: number;
	total_fees_usd: number;
	buy_count: number;
	sell_count: number;
}

export interface MockBtcPrice {
	price: number;
	change_24h: number;
}

export const mockPositionSummary: MockPositionSummary = {
	break_even_price: 42500,
	total_held_btc: 1.25,
	total_invested_usd: 53125,
	total_proceeds_usd: 8200,
	total_fees_usd: 124.5,
	buy_count: 3,
	sell_count: 1,
};

export const mockBtcPrice: MockBtcPrice = {
	price: 64230,
	change_24h: 2.4,
};

export const mockTrades: MockTrade[] = [
	{
		type: 'buy',
		amount_btc: 0.5,
		price_usd: 42000,
		total_usd: 21000,
		date: '2024-12-15',
		fee_usd: 31.5,
	},
	{
		type: 'buy',
		amount_btc: 0.5,
		price_usd: 38500,
		total_usd: 19250,
		date: '2024-11-28',
		fee_usd: 28.88,
	},
	{
		type: 'sell',
		amount_btc: 0.25,
		price_usd: 32800,
		total_usd: 8200,
		date: '2024-11-10',
		fee_usd: 24.6,
	},
	{
		type: 'buy',
		amount_btc: 0.5,
		price_usd: 35000,
		total_usd: 17500,
		date: '2024-10-22',
		fee_usd: 39.52,
	},
];

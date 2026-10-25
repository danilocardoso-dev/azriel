import type { MarketSnapshot, TradingLive, TradingOverview, TradingSettings, TradingSettingsInput } from "../types";
import { invokeDatabase } from "./tauri";

export const TRADING_UPDATE_EVENT = "trading-update";

export const tradingRepository = {
  overview: () => invokeDatabase<TradingOverview>("get_trading_overview"),
  live: () => invokeDatabase<TradingLive>("get_trading_live"),
  updateSettings: (input: TradingSettingsInput) => invokeDatabase<TradingSettings>("update_trading_settings", { input }),
  updateKeys: (apiKey: string, secretKey: string) => invokeDatabase<TradingSettings>("update_trading_keys", { input: { apiKey, secretKey } }),
  start: () => invokeDatabase<void>("start_trading"),
  stop: () => invokeDatabase<void>("stop_trading"),
  runCycleNow: () => invokeDatabase<void>("run_trading_cycle_now"),
  closePosition: () => invokeDatabase<void>("close_trading_position"),
  market: (symbol: string, interval: string) => invokeDatabase<MarketSnapshot>("get_market_snapshot", { symbol, interval }),
  symbols: () => invokeDatabase<string[]>("list_market_symbols"),
  setSymbol: (symbol: string) => invokeDatabase<TradingSettings>("set_trading_symbol", { symbol }),
};

import { invokeDatabase } from "./tauri";

export type MoneyKind = "expense" | "income";
export type MoneyRecurrence = "once" | "installments" | "monthly";

export interface MoneyItem {
  id: string; kind: MoneyKind; title: string; amountCents: number; firstDate: string;
  recurrence: MoneyRecurrence; installments: number | null; project: string | null;
}
export type MoneyItemInput = MoneyItem;
export interface MoneyEntry { item: MoneyItem; date: string; label: string; paid: boolean }
export interface MoneyProject { name: string; monthCents: number; receivedCents: number; averageCents: number; totalCents: number }
export interface MoneyTotals { expensesCents: number; expensesPaidCents: number; incomeCents: number; incomeReceivedCents: number; balanceCents: number }
export interface MoneyMonth { month: string; expenses: MoneyEntry[]; income: MoneyEntry[]; totals: MoneyTotals; projects: MoneyProject[] }

export const moneyRepository = {
  month: (month: string) => invokeDatabase<MoneyMonth>("money_month", { month }),
  save: (input: MoneyItemInput) => invokeDatabase<MoneyItem>("money_save_item", { input }),
  remove: (id: string) => invokeDatabase<void>("money_delete_item", { id }),
  setPaid: (id: string, month: string, paid: boolean) => invokeDatabase<void>("money_set_paid", { id, month, paid }),
  projects: () => invokeDatabase<string[]>("money_projects"),
};

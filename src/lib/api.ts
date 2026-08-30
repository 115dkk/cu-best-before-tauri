// Thin, typed wrappers over the Tauri commands defined in
// src-tauri/src/commands.rs. Shapes mirror docs/BACKEND-CONTRACT.md exactly;
// no domain logic lives here.
import { invoke } from "@tauri-apps/api/core";

export type Product = "onigiri" | "gimbap" | "lunchbox" | "sandwich" | "burger";
export type Location = "store" | "walk_in";

/** Canonical product order (matches `Product::ALL` in core). */
export const PRODUCTS: readonly Product[] = ["onigiri", "gimbap", "lunchbox", "sandwich", "burger"];
export const PRODUCT_LABEL: Record<Product, string> = {
  onigiri: "삼각김밥",
  gimbap: "김밥",
  lunchbox: "도시락",
  sandwich: "샌드위치",
  burger: "햄버거",
};

export const LOCATIONS: readonly Location[] = ["store", "walk_in"];
export const LOCATION_LABEL: Record<Location, string> = {
  store: "매장",
  walk_in: "워크인",
};

/** ISO local datetime without zone, e.g. "2026-08-30T14:00:00". */
export type LocalDateTime = string;

export interface Entry {
  at: LocalDateTime;
  quantity: number;
}

export type Section = Record<Product, Entry[]>;

export interface Sheet {
  id: string;
  created_at: LocalDateTime;
  updated_at: LocalDateTime;
  sections: Record<Location, Section>;
}

export interface SheetSummary {
  id: string;
  created_at: LocalDateTime;
  updated_at: LocalDateTime;
  entry_count: number;
  total_quantity: number;
}

export interface TimeOption {
  at: LocalDateTime;
  hour: number;
  label: string;
}

export interface DateOption {
  /** ISO date, e.g. "2026-08-30". */
  date: string;
  label: string;
  times: TimeOption[];
}

export interface SlotOptions {
  product: Product;
  dates: DateOption[];
}

export interface ExportResult {
  path: string;
  file_name: string;
  bytes: number;
}

export const api = {
  listSheets: () => invoke<SheetSummary[]>("list_sheets"),
  createSheet: () => invoke<Sheet>("create_sheet"),
  getSheet: (id: string) => invoke<Sheet>("get_sheet", { id }),
  /** Returns the normalized sheet (sorted, duplicates merged, updated_at bumped). */
  saveSheet: (sheet: Sheet) => invoke<Sheet>("save_sheet", { sheet }),
  deleteSheet: (id: string) => invoke<void>("delete_sheet", { id }),
  slotOptions: (product: Product) => invoke<SlotOptions>("slot_options", { product }),
  exportSheet: (id: string) => invoke<ExportResult>("export_sheet", { id }),
};

/** Commands reject with a plain string (Display of core::Error). */
export function errorMessage(e: unknown): string {
  if (typeof e === "string") return e;
  if (e instanceof Error) return e.message;
  return String(e);
}

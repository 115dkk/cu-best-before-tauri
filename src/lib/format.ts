// Display-only formatting of local datetimes coming from the backend.
// Mirrors the label rules in core::slots so the app and the exported PNG agree.

const WEEKDAY_KO = ["일", "월", "화", "수", "목", "금", "토"] as const;

export interface LocalParts {
  year: number;
  month: number;
  day: number;
  hour: number;
  minute: number;
  weekday: number;
}

/** Parses "2026-08-30T14:00:00" (or "2026-08-30") without timezone shifts. */
export function parseLocal(iso: string): LocalParts {
  const m = /^(\d{4})-(\d{2})-(\d{2})(?:T(\d{2}):(\d{2}))?/.exec(iso);
  if (!m) throw new Error(`invalid local datetime: ${iso}`);
  const year = Number(m[1]);
  const month = Number(m[2]);
  const day = Number(m[3]);
  const hour = m[4] === undefined ? 0 : Number(m[4]);
  const minute = m[5] === undefined ? 0 : Number(m[5]);
  const weekday = new Date(year, month - 1, day).getDay();
  return { year, month, day, hour, minute, weekday };
}

/** "8/30 (일)" */
export function dateLabel(iso: string): string {
  const p = parseLocal(iso);
  return `${p.month}/${p.day} (${WEEKDAY_KO[p.weekday]})`;
}

/** 14 → "오후 2시", 2 → "오전 2시" */
export function hourLabel(hour: number): string {
  const meridiem = hour < 12 ? "오전" : "오후";
  const h12 = hour % 12 === 0 ? 12 : hour % 12;
  return `${meridiem} ${h12}시`;
}

/** "8/30 14시" — same text the PNG uses for an entry. */
export function entryLabel(iso: string): string {
  const p = parseLocal(iso);
  return `${p.month}/${p.day} ${String(p.hour).padStart(2, "0")}시`;
}

/** "8/30 (일) 오전 8:02" — sheet creation time. */
export function sheetLabel(iso: string): string {
  const p = parseLocal(iso);
  const meridiem = p.hour < 12 ? "오전" : "오후";
  const h12 = p.hour % 12 === 0 ? 12 : p.hour % 12;
  return `${p.month}/${p.day} (${WEEKDAY_KO[p.weekday]}) ${meridiem} ${h12}:${String(p.minute).padStart(2, "0")}`;
}

/** Date part of a local datetime: "2026-08-30T14:00:00" → "2026-08-30". */
export function datePart(iso: string): string {
  return iso.slice(0, 10);
}

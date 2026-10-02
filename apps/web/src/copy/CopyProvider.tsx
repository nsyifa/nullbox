import { createContext, type ReactNode, useContext, useEffect, useMemo, useState } from "react";
import en, { type Copy } from "./en";
import ja from "./ja";

/** Add a locale here (and a dictionary of type `Copy`) to support more languages. */
export const dictionaries = { en, ja } satisfies Record<string, Copy>;
export type Locale = keyof typeof dictionaries;

const STORAGE_KEY = "nullbox.locale";

function initialLocale(): Locale {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored && stored in dictionaries) return stored as Locale;
  } catch {
    // storage blocked: fall through to the browser language
  }
  return navigator.language.toLowerCase().startsWith("ja") ? "ja" : "en";
}

interface CopyContextValue {
  copy: Copy;
  locale: Locale;
  setLocale: (locale: Locale) => void;
}

const CopyContext = createContext<CopyContextValue | null>(null);

export function CopyProvider({
  children,
  locale: forced,
}: {
  children: ReactNode;
  locale?: Locale;
}) {
  const [locale, setLocaleState] = useState<Locale>(() => forced ?? initialLocale());

  useEffect(() => {
    document.documentElement.lang = locale;
  }, [locale]);

  const value = useMemo<CopyContextValue>(
    () => ({
      copy: dictionaries[locale],
      locale,
      setLocale: (next) => {
        setLocaleState(next);
        try {
          localStorage.setItem(STORAGE_KEY, next);
        } catch {
          // not persisted, still switched
        }
      },
    }),
    [locale],
  );

  return <CopyContext.Provider value={value}>{children}</CopyContext.Provider>;
}

function useCopyContext(): CopyContextValue {
  const ctx = useContext(CopyContext);
  if (!ctx) throw new Error("useCopy() must be used inside <CopyProvider>");
  return ctx;
}

/** The active dictionary. */
export function useCopy(): Copy {
  return useCopyContext().copy;
}

export function useLocale(): [Locale, (locale: Locale) => void] {
  const { locale, setLocale } = useCopyContext();
  return [locale, setLocale];
}

const RELATIVE_UNITS: [Intl.RelativeTimeFormatUnit, number][] = [
  ["year", 365 * 24 * 3600],
  ["month", 30 * 24 * 3600],
  ["week", 7 * 24 * 3600],
  ["day", 24 * 3600],
  ["hour", 3600],
  ["minute", 60],
  ["second", 1],
];

/** Locale-aware formatting via `Intl` (no date or i18n library). */
export function useFormat() {
  const [locale] = useLocale();
  return useMemo(() => {
    const dateTime = new Intl.DateTimeFormat(locale, { dateStyle: "medium", timeStyle: "short" });
    const relative = new Intl.RelativeTimeFormat(locale, { numeric: "auto" });
    const number = new Intl.NumberFormat(locale);
    return {
      dateTime: (iso: string) => dateTime.format(new Date(iso)),
      number: (n: number) => number.format(n),
      /** "3 hours ago" / "3 時間前" */
      ago: (iso: string, now = Date.now()) => formatAgo(relative, (now - Date.parse(iso)) / 1000),
      /** An age in seconds, e.g. for the oldest ticket. */
      age: (seconds: number) => formatAgo(relative, seconds),
    };
  }, [locale]);
}

function formatAgo(relative: Intl.RelativeTimeFormat, seconds: number): string {
  for (const [unit, size] of RELATIVE_UNITS) {
    if (Math.abs(seconds) >= size || unit === "second") {
      return relative.format(-Math.round(seconds / size), unit);
    }
  }
  return "";
}

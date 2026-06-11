import {
  createContext,
  useCallback,
  useContext,
  useMemo,
  type ReactNode,
} from "react";
import { messagesFor, type Locale } from "./messages";

type Messages = ReturnType<typeof messagesFor>;

interface LocaleContextValue {
  locale: Locale;
  setLocale: (locale: Locale) => void;
  m: Messages;
}

const LocaleContext = createContext<LocaleContextValue | null>(null);

export function LocaleProvider({
  locale,
  onLocaleChange,
  children,
}: {
  locale: Locale;
  onLocaleChange?: (locale: Locale) => void;
  children: ReactNode;
}) {
  const setLocale = useCallback(
    (next: Locale) => {
      onLocaleChange?.(next);
    },
    [onLocaleChange]
  );
  const value = useMemo(
    () => ({ locale, setLocale, m: messagesFor(locale) }),
    [locale, setLocale]
  );
  return (
    <LocaleContext.Provider value={value}>{children}</LocaleContext.Provider>
  );
}

export function useLocale() {
  const ctx = useContext(LocaleContext);
  if (!ctx) {
    const fallback = messagesFor("en");
    return {
      locale: "en" as Locale,
      setLocale: () => {},
      m: fallback,
    };
  }
  return ctx;
}

export { normalizeLocale, type Locale } from "./messages";

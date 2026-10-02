import { useCallback, useState, useSyncExternalStore } from "react";
import { NavLink, Outlet, useLocation } from "react-router";
import { getActorName, setActorName, subscribeActor } from "../api/actor";
import { dictionaries, type Locale, useCopy, useLocale } from "../copy/CopyProvider";
import { useKonami } from "../hooks/useKonami";
import styles from "./AppShell.module.css";
import { EnterpriseMode } from "./EnterpriseMode";

const LANGUAGE_NAMES: Record<Locale, string> = { en: "English", ja: "日本語" };

export function AppShell() {
  const copy = useCopy();
  const { pathname } = useLocation();
  const [enterprise, setEnterprise] = useState(false);
  useKonami(useCallback(() => setEnterprise(true), []));

  return (
    <div className={styles.shell}>
      <header className={styles.header}>
        <NavLink to="/" className={styles.brand}>
          <span className={styles.logo} aria-hidden="true">
            ∅
          </span>
          <span>
            <strong>{copy.app.name}</strong>
            <small>{copy.app.subtitle}</small>
          </span>
        </NavLink>
        <nav className={styles.nav}>
          <NavLink to="/" end>
            {copy.app.nav.submit}
          </NavLink>
          <NavLink to="/inbox">{copy.app.nav.inbox}</NavLink>
        </nav>
        <div className={styles.tools}>
          {pathname.startsWith("/inbox") && <ActorPicker />}
          <LanguageSwitcher />
        </div>
      </header>
      <main className={styles.main}>
        <Outlet />
      </main>
      <footer className={styles.footer}>{copy.app.footer}</footer>
      {enterprise && <EnterpriseMode onExit={() => setEnterprise(false)} />}
    </div>
  );
}

function LanguageSwitcher() {
  const copy = useCopy();
  const [locale, setLocale] = useLocale();
  return (
    <fieldset className={styles.switcher} aria-label={copy.app.language}>
      {(Object.keys(dictionaries) as Locale[]).map((option) => (
        <button
          key={option}
          type="button"
          lang={option}
          aria-pressed={option === locale}
          onClick={() => setLocale(option)}
        >
          {LANGUAGE_NAMES[option]}
        </button>
      ))}
    </fieldset>
  );
}

/** Not a login: the worker just tells us who they are (see api/actor.ts). */
function ActorPicker() {
  const copy = useCopy();
  const name = useSyncExternalStore(subscribeActor, getActorName);
  return (
    <label className={styles.actor} title={copy.worker.actorHint}>
      <span>{copy.worker.you}</span>
      <input
        value={name}
        placeholder={copy.worker.actorPlaceholder}
        maxLength={100}
        onChange={(event) => setActorName(event.target.value)}
      />
    </label>
  );
}

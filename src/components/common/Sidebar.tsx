import { NavLink } from "react-router-dom";
import { useTranslation } from "react-i18next";

const navKeys = [
  { to: "/", key: "sidebar.dashboard", end: true },
  { to: "/env", key: "sidebar.env", end: false },
  { to: "/deploy", key: "sidebar.deploy", end: false },
  { to: "/recipe", key: "sidebar.recipe", end: false },
  { to: "/flow", key: "sidebar.flow", end: false },
  { to: "/log", key: "sidebar.log", end: false },
];

export function Sidebar() {
  const { t, i18n } = useTranslation();

  const toggleLang = () => {
    const next = i18n.language === "zh" ? "en" : "zh";
    i18n.changeLanguage(next);
    localStorage.setItem("lang", next);
  };

  return (
    <aside className="flex w-56 flex-col border-r border-[var(--color-border)] bg-[var(--color-surface)]">
      <div className="flex items-center gap-2 border-b border-[var(--color-border)] px-4 py-4">
        <span className="text-xl">🦀</span>
        <h1 className="text-sm font-bold text-[var(--color-text)]">
          One Key OpenClaw
        </h1>
      </div>
      <nav className="flex flex-1 flex-col gap-1 p-2">
        {navKeys.map((item) => (
          <NavLink
            key={item.to}
            to={item.to}
            end={item.end}
            className={({ isActive }) =>
              `rounded-md px-3 py-2 text-sm transition-colors ${
                isActive
                  ? "bg-[var(--color-primary)] text-white"
                  : "text-[var(--color-text-muted)] hover:bg-[var(--color-surface-hover)] hover:text-[var(--color-text)]"
              }`
            }
          >
            {t(item.key)}
          </NavLink>
        ))}
      </nav>
      <div className="border-t border-[var(--color-border)] p-2">
        <button
          onClick={toggleLang}
          className="w-full rounded-md px-3 py-2 text-xs text-[var(--color-text-muted)] transition-colors hover:bg-[var(--color-surface-hover)] hover:text-[var(--color-text)]"
        >
          🌐 {i18n.language === "zh" ? "English" : "中文"}
        </button>
      </div>
    </aside>
  );
}

import { NavLink } from "react-router-dom";

const navItems = [
  { to: "/", label: "🔍 环境检测", end: true },
  { to: "/deploy", label: "🚀 部署", end: false },
  { to: "/log", label: "📝 日志", end: false },
];

export function Sidebar() {
  return (
    <aside className="flex w-56 flex-col border-r border-[var(--color-border)] bg-[var(--color-surface)]">
      <div className="flex items-center gap-2 border-b border-[var(--color-border)] px-4 py-4">
        <span className="text-xl">🦀</span>
        <h1 className="text-sm font-bold text-[var(--color-text)]">
          One Key OpenClaw
        </h1>
      </div>
      <nav className="flex flex-1 flex-col gap-1 p-2">
        {navItems.map((item) => (
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
            {item.label}
          </NavLink>
        ))}
      </nav>
    </aside>
  );
}

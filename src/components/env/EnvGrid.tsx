import type { EnvItem } from "../../ipc/types";
import { EnvCard } from "./EnvCard";

interface EnvGridProps {
  items: EnvItem[];
  filter: "all" | "ok" | "missing" | "versionMismatch";
  onReprobe: (id: string) => void;
}

export function EnvGrid({ items, filter, onReprobe }: EnvGridProps) {
  const filtered =
    filter === "all"
      ? items
      : items.filter((item) => item.status.type === filter);

  if (filtered.length === 0) {
    return (
      <div className="flex items-center justify-center py-12 text-[var(--color-text-muted)]">
        {items.length === 0 ? "暂无环境检测数据" : "没有匹配的环境项"}
      </div>
    );
  }

  return (
    <div className="grid grid-cols-1 gap-4 md:grid-cols-2 xl:grid-cols-3">
      {filtered.map((item) => (
        <EnvCard key={item.id} item={item} onReprobe={onReprobe} />
      ))}
    </div>
  );
}

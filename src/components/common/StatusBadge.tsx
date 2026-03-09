import type { EnvStatus } from "../../ipc/types";

interface StatusBadgeProps {
  status: EnvStatus;
}

export function StatusBadge({ status }: StatusBadgeProps) {
  const config = getStatusConfig(status);

  return (
    <span
      className={`inline-flex items-center gap-1.5 rounded-full px-2.5 py-0.5 text-xs font-medium ${config.className}`}
    >
      <span>{config.icon}</span>
      <span>{config.label}</span>
    </span>
  );
}

function getStatusConfig(status: EnvStatus) {
  switch (status.type) {
    case "ok":
      return {
        icon: "✅",
        label: "已安装",
        className: "bg-green-500/10 text-green-400",
      };
    case "missing":
      return {
        icon: "❌",
        label: "缺失",
        className: "bg-red-500/10 text-red-400",
      };
    case "versionMismatch":
      return {
        icon: "⚠️",
        label: "版本不符",
        className: "bg-yellow-500/10 text-yellow-400",
      };
    case "error":
      return {
        icon: "⚠️",
        label: "错误",
        className: "bg-red-500/10 text-red-400",
      };
    case "checking":
      return {
        icon: "🔄",
        label: "检测中",
        className: "bg-blue-500/10 text-blue-400",
      };
  }
}

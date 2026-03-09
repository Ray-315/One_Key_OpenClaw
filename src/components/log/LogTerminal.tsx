import { useEffect, useRef } from "react";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import "@xterm/xterm/css/xterm.css";
import { useLogStore } from "../../store/logStore";
import { formatTime } from "../../utils/format";

export function LogTerminal() {
  const termRef = useRef<HTMLDivElement>(null);
  const xtermRef = useRef<Terminal | null>(null);
  const fitAddonRef = useRef<FitAddon | null>(null);
  const lastLogIdRef = useRef<number>(0);

  const logs = useLogStore((s) => s.logs);
  const filterLevel = useLogStore((s) => s.filterLevel);

  // Initialize terminal
  useEffect(() => {
    if (!termRef.current) return;

    const terminal = new Terminal({
      theme: {
        background: "#0f1117",
        foreground: "#e4e5e9",
        cursor: "#e4e5e9",
        selectionBackground: "#6c63ff44",
      },
      fontSize: 13,
      fontFamily: '"Fira Code", "Cascadia Code", monospace',
      cursorBlink: false,
      disableStdin: true,
      scrollback: 10000,
    });

    const fitAddon = new FitAddon();
    terminal.loadAddon(fitAddon);
    terminal.open(termRef.current);
    fitAddon.fit();

    xtermRef.current = terminal;
    fitAddonRef.current = fitAddon;

    const resizeObserver = new ResizeObserver(() => {
      fitAddon.fit();
    });
    resizeObserver.observe(termRef.current);

    return () => {
      resizeObserver.disconnect();
      terminal.dispose();
    };
  }, []);

  // Write new logs to terminal
  useEffect(() => {
    const terminal = xtermRef.current;
    if (!terminal) return;

    const newLogs = logs.filter((log) => log.id > lastLogIdRef.current);
    if (newLogs.length === 0) return;

    for (const log of newLogs) {
      if (filterLevel !== "all" && log.level !== filterLevel) continue;

      const time = formatTime(log.timestamp);
      const level = log.level.toUpperCase().padEnd(5);
      const color = getLevelColor(log.level);

      terminal.writeln(`${color} ${time}  ${level}  ${log.message}\x1b[0m`);
    }

    lastLogIdRef.current = newLogs[newLogs.length - 1].id;
  }, [logs, filterLevel]);

  return (
    <div
      ref={termRef}
      className="h-full min-h-[300px] w-full rounded-lg border border-[var(--color-border)] bg-[var(--color-bg)]"
    />
  );
}

function getLevelColor(level: string): string {
  switch (level) {
    case "error":
      return "\x1b[31m"; // red
    case "warn":
      return "\x1b[33m"; // yellow
    case "info":
      return "\x1b[36m"; // cyan
    case "debug":
      return "\x1b[90m"; // gray
    default:
      return "\x1b[0m";
  }
}

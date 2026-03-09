import { useMemo } from "react";
import {
  ReactFlow,
  Background,
  Controls,
  MiniMap,
  useNodesState,
  useEdgesState,
  type Node,
  type Edge,
  MarkerType,
} from "@xyflow/react";
import "@xyflow/react/dist/style.css";

import type { TaskGraphData, TaskGraphNode } from "../../ipc/types";
import type { StepStatus } from "../../ipc/types";

// ---------------------------------------------------------------------------
// Node colour by step status
// ---------------------------------------------------------------------------

const STATUS_COLORS: Record<string, string> = {
  pending: "#6b7280",   // gray
  waiting: "#a78bfa",   // violet
  running: "#3b82f6",   // blue
  success: "#22c55e",   // green
  skipped: "#f59e0b",   // amber
  cancelled: "#6b7280", // gray
  failed: "#ef4444",    // red
};

function stepStatusKey(status: StepStatus): string {
  if (typeof status === "object") return status.type;
  return status as string;
}

// ---------------------------------------------------------------------------
// Build React Flow nodes / edges from TaskGraphData
// ---------------------------------------------------------------------------

const LAYER_GAP_X = 220;
const NODE_GAP_Y = 100;
const NODE_WIDTH = 180;
const NODE_HEIGHT = 56;

function buildFlowNodes(
  graphNodes: TaskGraphNode[],
  stepStatuses: Record<string, StepStatus>
): Node[] {
  // Group nodes by layer for Y positioning.
  const layerCounts: Record<number, number> = {};
  const layerOffset: Record<number, number> = {};

  for (const n of graphNodes) {
    layerCounts[n.layer] = (layerCounts[n.layer] ?? 0) + 1;
  }
  for (const layer in layerCounts) {
    layerOffset[layer] = 0;
  }

  return graphNodes.map((n) => {
    const layer = n.layer;
    const count = layerCounts[layer] ?? 1;
    const offset = layerOffset[layer]++;
    const totalHeight = count * NODE_HEIGHT + (count - 1) * (NODE_GAP_Y - NODE_HEIGHT);
    const startY = -totalHeight / 2;
    const y = startY + offset * NODE_GAP_Y;

    const status = stepStatuses[n.id];
    const statusKey = status ? stepStatusKey(status) : "pending";
    const color = STATUS_COLORS[statusKey] ?? STATUS_COLORS.pending;

    const isRunning = statusKey === "running";
    const isFailed = statusKey === "failed";
    const isSuccess = statusKey === "success";

    return {
      id: n.id,
      type: "default",
      position: { x: layer * LAYER_GAP_X, y },
      data: {
        label: (
          <div className="flex flex-col items-start gap-0.5 text-left">
            <span className="text-xs font-semibold leading-tight truncate w-full">
              {n.name}
            </span>
            {n.description && (
              <span className="text-[10px] text-gray-400 leading-tight truncate w-full">
                {n.description}
              </span>
            )}
          </div>
        ),
      },
      style: {
        background: `${color}22`,
        border: `2px solid ${color}${isRunning ? "" : "99"}`,
        borderRadius: 8,
        width: NODE_WIDTH,
        minHeight: NODE_HEIGHT,
        boxShadow: isRunning
          ? `0 0 12px ${color}66`
          : isFailed
          ? `0 0 8px ${color}44`
          : isSuccess
          ? `0 0 6px ${color}33`
          : "none",
        color: "#e2e8f0",
        fontSize: 12,
        padding: "8px 10px",
        cursor: "default",
      },
    };
  });
}

function buildFlowEdges(graphEdges: { id: string; source: string; target: string }[]): Edge[] {
  return graphEdges.map((e) => ({
    id: e.id,
    source: e.source,
    target: e.target,
    type: "smoothstep",
    style: { stroke: "#475569", strokeWidth: 1.5 },
    markerEnd: {
      type: MarkerType.ArrowClosed,
      color: "#475569",
    },
    animated: false,
  }));
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

interface TaskFlowProps {
  graphData: TaskGraphData;
  stepStatuses?: Record<string, StepStatus>;
  className?: string;
}

/**
 * Renders the task DAG using React Flow.
 * Pass `stepStatuses` to colour-code nodes by their runtime status.
 */
export function TaskFlow({
  graphData,
  stepStatuses = {},
  className = "",
}: TaskFlowProps) {
  const initialNodes = useMemo(
    () => buildFlowNodes(graphData.nodes, stepStatuses),
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [graphData]
  );
  const initialEdges = useMemo(
    () => buildFlowEdges(graphData.edges),
    [graphData]
  );

  // Re-colour nodes when statuses change.
  const coloredNodes = useMemo(
    () => buildFlowNodes(graphData.nodes, stepStatuses),
    [graphData, stepStatuses]
  );

  const [nodes, , onNodesChange] = useNodesState(initialNodes);
  const [edges, , onEdgesChange] = useEdgesState(initialEdges);

  // Keep nodes in sync when statuses change.
  const syncedNodes = useMemo(() => {
    return nodes.map((n) => {
      const updated = coloredNodes.find((cn) => cn.id === n.id);
      if (!updated) return n;
      return { ...n, data: updated.data, style: updated.style };
    });
  }, [nodes, coloredNodes]);

  if (graphData.nodes.length === 0) {
    return (
      <div
        className={`flex items-center justify-center text-[var(--color-text-muted)] ${className}`}
      >
        <p className="text-sm">暂无步骤数据</p>
      </div>
    );
  }

  return (
    <div className={`${className}`} style={{ background: "transparent" }}>
      <ReactFlow
        nodes={syncedNodes}
        edges={edges}
        onNodesChange={onNodesChange}
        onEdgesChange={onEdgesChange}
        fitView
        attributionPosition="bottom-right"
        proOptions={{ hideAttribution: false }}
        style={{ background: "transparent" }}
      >
        <Background color="#334155" gap={20} size={1} />
        <Controls
          style={{ background: "#1e293b", border: "1px solid #334155", color: "#94a3b8" }}
        />
        <MiniMap
          nodeColor={(n) => {
            const st = stepStatuses[(n as Node).id];
            const key = st ? stepStatusKey(st) : "pending";
            return STATUS_COLORS[key] ?? STATUS_COLORS.pending;
          }}
          style={{ background: "#0f172a", border: "1px solid #334155" }}
          maskColor="rgba(15,23,42,0.7)"
        />
      </ReactFlow>
    </div>
  );
}

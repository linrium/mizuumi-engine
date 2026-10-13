"use client"

import { IconCircleCheck, IconLoader2, IconTable } from "@tabler/icons-react"
import {
  Background,
  BackgroundVariant,
  Controls,
  type Edge,
  Handle,
  MarkerType,
  type Node,
  type NodeProps,
  Position,
  ReactFlow,
  type ReactFlowInstance,
} from "@xyflow/react"
import { useCallback, useEffect, useMemo, useState } from "react"
import "@xyflow/react/dist/style.css"
import { expandLineageNode } from "@/app/(app)/unitycatalog/(browser)/[catalog]/[schema]/[table]/lineage/actions"
import type { LineageAssetNode, TableLineageNode } from "@/lib/dagster"

const NODE_WIDTH = 288
const NODE_HEIGHT = 192
const COLUMN_GAP = 480
const ROW_GAP = 220
const GROUP_PADDING_X = 32
const GROUP_HEADER_HEIGHT = 74
const GROUP_PADDING_BOTTOM = 32

type AssetNodeData = Record<string, unknown> & {
  asset: LineageAssetNode
  current: boolean
  expanded: boolean
  loading: boolean
  onExpand: (asset: LineageAssetNode) => void
}

type GroupNodeData = Record<string, unknown> & {
  groupName: string
  locationName: string
}

type PositionedAsset = {
  asset: LineageAssetNode
  current: boolean
  position: { x: number; y: number }
}

function assetId(asset: LineageAssetNode) {
  return assetPathId(asset.assetKey.path)
}

function assetPathId(path: string[]) {
  return `asset:${JSON.stringify(path)}`
}

function groupId(asset: LineageAssetNode) {
  return `group:${asset.repository.location.name}:${asset.groupName}`
}

function formatTimestamp(timestamp: string | undefined) {
  if (!timestamp) return null

  return new Intl.DateTimeFormat(undefined, {
    dateStyle: "medium",
    timeStyle: "short",
  }).format(new Date(Number(timestamp)))
}

function AssetNode({ data }: NodeProps<Node<AssetNodeData, "asset">>) {
  const { asset, current, expanded, loading, onExpand } = data
  const latestMaterialization = formatTimestamp(
    asset.assetMaterializations[0]?.timestamp,
  )
  const latestObservation = formatTimestamp(
    asset.assetObservations[0]?.timestamp,
  )
  const status = latestMaterialization
    ? "Materialized"
    : latestObservation
      ? "Observed"
      : "No events"
  const timestamp = latestMaterialization ?? latestObservation
  const statusTone = latestMaterialization
    ? "bg-emerald-500/10 text-emerald-700 dark:text-emerald-300"
    : latestObservation
      ? "bg-sky-500/10 text-sky-700 dark:text-sky-300"
      : "bg-muted text-muted-foreground"

  return (
    <div
      className={`relative flex h-[192px] w-[288px] flex-col overflow-hidden rounded-md border bg-card transition-colors ${
        current
          ? "border-primary ring-1 ring-primary/30"
          : expanded
            ? "border-border"
            : "border-border hover:border-primary/60"
      }`}
    >
      <button
        type="button"
        className="absolute inset-0 z-10 rounded-md bg-transparent disabled:cursor-default enabled:cursor-pointer"
        aria-label={
          loading
            ? `Loading neighbors for ${asset.assetKey.path.join(".")}`
            : expanded
              ? `Neighbors loaded for ${asset.assetKey.path.join(".")}`
              : `Expand neighbors for ${asset.assetKey.path.join(".")}`
        }
        disabled={expanded || loading}
        onClick={() => onExpand(asset)}
      />
      <Handle type="target" position={Position.Left} className="opacity-0" />
      <div className="flex min-h-12 items-start gap-2 border-b bg-muted/45 px-3 py-2.5">
        <IconTable className="mt-0.5 size-4 shrink-0 text-muted-foreground" />
        <div className="min-w-0 flex-1 font-mono">
          <div className="truncate text-sm font-semibold">
            {asset.assetKey.path.at(-1)}
          </div>
          <div className="truncate text-[10px] leading-4 text-muted-foreground">
            {asset.assetKey.path.join(".")}
          </div>
        </div>
        {loading ? (
          <IconLoader2 className="mt-0.5 size-3.5 shrink-0 animate-spin text-primary" />
        ) : expanded ? (
          <IconCircleCheck className="mt-0.5 size-3.5 shrink-0 text-muted-foreground" />
        ) : null}
      </div>
      <p className="line-clamp-3 h-[72px] shrink-0 px-3 py-2 text-[11px] leading-[1.125rem] text-muted-foreground">
        {asset.description ?? "No description"}
      </p>
      <div
        className={`flex min-h-8 shrink-0 items-center justify-between gap-3 border-y px-3 text-[11px] ${statusTone}`}
      >
        <span className="font-medium">{status}</span>
        <span className="text-muted-foreground">{timestamp ?? "—"}</span>
      </div>
      <div className="flex min-h-10 shrink-0 items-center gap-1.5 overflow-hidden px-3">
        {asset.kinds.map((kind) => (
          <span
            key={kind}
            className="whitespace-nowrap rounded-full border border-border/70 bg-muted/70 px-2 py-0.5 text-[10px] font-medium text-muted-foreground"
          >
            {kind}
          </span>
        ))}
      </div>
      <Handle type="source" position={Position.Right} className="opacity-0" />
    </div>
  )
}

function GroupNode({ data }: NodeProps<Node<GroupNodeData, "lineageGroup">>) {
  return (
    <div className="size-full overflow-hidden rounded-lg border border-border bg-card/75">
      <div className="flex h-10 flex-col justify-center border-b border-border/80 bg-muted/35 px-4">
        <div className="font-mono text-sm font-semibold">{data.groupName}</div>
        {/*<div className="mt-0.5 font-mono text-[10px] text-muted-foreground">
          {data.locationName}
        </div>*/}
      </div>
    </div>
  )
}

const nodeTypes = {
  asset: AssetNode,
  lineageGroup: GroupNode,
}

function centeredY(index: number, count: number, maxRows: number) {
  return ((maxRows - count) / 2 + index) * ROW_GAP
}

function createElements(
  lineageNodes: TableLineageNode[],
  rootId: string,
  loadingIds: Set<string>,
  onExpand: (asset: LineageAssetNode) => void,
) {
  const expandedIds = new Set(lineageNodes.map(assetId))
  const assets = new Map<string, LineageAssetNode>()
  const edgeMap = new Map<string, Edge>()

  for (const node of lineageNodes) assets.set(assetId(node), node)

  for (const node of lineageNodes) {
    const nodeId = assetId(node)
    for (const { asset } of node.dependencies) {
      const upstreamId = assetId(asset)
      if (!assets.has(upstreamId)) assets.set(upstreamId, asset)
      edgeMap.set(`${upstreamId}->${nodeId}`, {
        id: `${upstreamId}->${nodeId}`,
        source: upstreamId,
        target: nodeId,
      })
    }
    for (const { asset } of node.dependedBy) {
      const downstreamId = assetId(asset)
      if (!assets.has(downstreamId)) assets.set(downstreamId, asset)
      edgeMap.set(`${nodeId}->${downstreamId}`, {
        id: `${nodeId}->${downstreamId}`,
        source: nodeId,
        target: downstreamId,
      })
    }
  }

  const depths = new Map<string, number>([[rootId, 0]])
  for (let pass = 0; pass < assets.size; pass += 1) {
    let changed = false
    for (const edge of edgeMap.values()) {
      const sourceDepth = depths.get(edge.source)
      const targetDepth = depths.get(edge.target)
      if (sourceDepth !== undefined && targetDepth === undefined) {
        depths.set(edge.target, sourceDepth + 1)
        changed = true
      } else if (sourceDepth === undefined && targetDepth !== undefined) {
        depths.set(edge.source, targetDepth - 1)
        changed = true
      }
    }
    if (!changed) break
  }

  const assetsByDepth = new Map<number, LineageAssetNode[]>()
  for (const asset of assets.values()) {
    const depth = depths.get(assetId(asset)) ?? 0
    assetsByDepth.set(depth, [...(assetsByDepth.get(depth) ?? []), asset])
  }
  for (const column of assetsByDepth.values()) {
    column.sort((a, b) =>
      `${a.groupName}:${assetId(a)}`.localeCompare(
        `${b.groupName}:${assetId(b)}`,
      ),
    )
  }

  const minDepth = Math.min(...assetsByDepth.keys())
  const maxRows = Math.max(...[...assetsByDepth.values()].map((v) => v.length))
  const positioned: PositionedAsset[] = []
  for (const [depth, column] of assetsByDepth) {
    column.forEach((asset, index) => {
      positioned.push({
        asset,
        current: assetId(asset) === rootId,
        position: {
          x: (depth - minDepth) * COLUMN_GAP,
          y: centeredY(index, column.length, maxRows),
        },
      })
    })
  }

  const groups = new Map<string, PositionedAsset[]>()
  for (const item of positioned) {
    const id = groupId(item.asset)
    groups.set(id, [...(groups.get(id) ?? []), item])
  }

  const groupNodes: Node<GroupNodeData, "lineageGroup">[] = []
  const assetNodes: Node<AssetNodeData, "asset">[] = []

  for (const [id, items] of groups) {
    const minX = Math.min(...items.map((item) => item.position.x))
    const maxX = Math.max(...items.map((item) => item.position.x))
    const minY = Math.min(...items.map((item) => item.position.y))
    const maxY = Math.max(...items.map((item) => item.position.y))
    const first = items[0].asset

    groupNodes.push({
      id,
      type: "lineageGroup",
      position: {
        x: minX - GROUP_PADDING_X,
        y: minY - GROUP_HEADER_HEIGHT,
      },
      data: {
        groupName: first.groupName,
        locationName: first.repository.location.name,
      },
      draggable: false,
      selectable: false,
      style: {
        width: maxX - minX + NODE_WIDTH + GROUP_PADDING_X * 2,
        height:
          maxY -
          minY +
          NODE_HEIGHT +
          GROUP_HEADER_HEIGHT +
          GROUP_PADDING_BOTTOM,
      },
    })

    for (const item of items) {
      assetNodes.push({
        id: assetId(item.asset),
        type: "asset",
        parentId: id,
        extent: "parent",
        position: {
          x: item.position.x - minX + GROUP_PADDING_X,
          y: item.position.y - minY + GROUP_HEADER_HEIGHT,
        },
        data: {
          asset: item.asset,
          current: item.current,
          expanded: expandedIds.has(assetId(item.asset)),
          loading: loadingIds.has(assetId(item.asset)),
          onExpand,
        },
        draggable: false,
      })
    }
  }

  return { nodes: [...groupNodes, ...assetNodes], edges: [...edgeMap.values()] }
}

export function LineageGraph({ current }: { current: TableLineageNode }) {
  const rootId = assetId(current)
  const [expandedNodes, setExpandedNodes] = useState<
    Record<string, TableLineageNode>
  >({ [rootId]: current })
  const [loadingIds, setLoadingIds] = useState<Set<string>>(new Set())
  const [error, setError] = useState<string>()
  const [flow, setFlow] = useState<ReactFlowInstance | null>(null)

  const expand = useCallback(
    async (asset: LineageAssetNode) => {
      const id = assetId(asset)
      if (expandedNodes[id] || loadingIds.has(id)) return

      setError(undefined)
      setLoadingIds((previous) => new Set(previous).add(id))
      const result = await expandLineageNode(asset.assetKey.path)

      if (result.node) {
        setExpandedNodes((previous) => ({
          ...previous,
          [id]: result.node as TableLineageNode,
        }))
      } else {
        setError(result.error ?? "Unable to load neighboring assets.")
      }

      setLoadingIds((previous) => {
        const next = new Set(previous)
        next.delete(id)
        return next
      })
    },
    [expandedNodes, loadingIds],
  )

  const { nodes, edges } = useMemo(
    () =>
      createElements(Object.values(expandedNodes), rootId, loadingIds, expand),
    [expandedNodes, rootId, loadingIds, expand],
  )

  useEffect(() => {
    if (!flow || nodes.length === 0) return
    void flow.fitView({ padding: 0.1, maxZoom: 1.05, duration: 300 })
  }, [flow, nodes.length])

  return (
    <div className="relative h-[clamp(480px,62vh,680px)] overflow-hidden rounded-lg border bg-muted/20 shadow-inner">
      {error ? (
        <div className="absolute top-3 left-1/2 z-20 -translate-x-1/2 rounded-md border border-destructive/30 bg-background px-3 py-2 text-xs text-destructive shadow-sm">
          {error}
        </div>
      ) : null}
      <ReactFlow
        nodes={nodes}
        edges={edges}
        nodeTypes={nodeTypes}
        fitView
        fitViewOptions={{ padding: 0.1, maxZoom: 1.05 }}
        minZoom={0.25}
        maxZoom={1.5}
        nodesConnectable={false}
        nodesDraggable={false}
        onInit={setFlow}
        defaultEdgeOptions={{
          type: "smoothstep",
          zIndex: 2,
          markerEnd: { type: MarkerType.ArrowClosed },
          style: { stroke: "var(--muted-foreground)", strokeWidth: 1.5 },
        }}
        defaultMarkerColor="var(--muted-foreground)"
      >
        <Background
          variant={BackgroundVariant.Dots}
          gap={24}
          size={1}
          color="var(--border)"
          bgColor="color-mix(in oklab, var(--muted) 45%, var(--background))"
        />
        <Controls
          showInteractive={false}
          className="overflow-hidden rounded-md border border-border bg-card shadow-sm"
        />
      </ReactFlow>
    </div>
  )
}

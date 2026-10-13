"use client"

import { IconTable } from "@tabler/icons-react"
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
} from "@xyflow/react"
import "@xyflow/react/dist/style.css"
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
  return `asset:${JSON.stringify(asset.assetKey.path)}`
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
  const { asset, current } = data
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
      className={`flex h-[192px] w-[288px] flex-col overflow-hidden rounded-md border bg-card ${
        current ? "border-primary ring-1 ring-primary/30" : "border-border"
      }`}
    >
      <Handle type="target" position={Position.Left} className="opacity-0" />
      <div className="flex min-h-12 items-start gap-2 border-b bg-muted/45 px-3 py-2.5">
        <IconTable className="mt-0.5 size-4 shrink-0 text-muted-foreground" />
        <div className="min-w-0 font-mono">
          <div className="truncate text-sm font-semibold">
            {asset.assetKey.path.at(-1)}
          </div>
          <div className="truncate text-[10px] leading-4 text-muted-foreground">
            {asset.assetKey.path.join(".")}
          </div>
        </div>
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

function createElements(current: TableLineageNode) {
  const upstream = current.dependencies.map(({ asset }) => asset)
  const downstream = current.dependedBy.map(({ asset }) => asset)
  const maxRows = Math.max(upstream.length, downstream.length, 1)
  const positioned: PositionedAsset[] = [
    ...upstream.map((asset, index) => ({
      asset,
      current: false,
      position: { x: 0, y: centeredY(index, upstream.length, maxRows) },
    })),
    {
      asset: current,
      current: true,
      position: { x: COLUMN_GAP, y: centeredY(0, 1, maxRows) },
    },
    ...downstream.map((asset, index) => ({
      asset,
      current: false,
      position: {
        x: COLUMN_GAP * 2,
        y: centeredY(index, downstream.length, maxRows),
      },
    })),
  ]

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
        data: { asset: item.asset, current: item.current },
        draggable: false,
      })
    }
  }

  const currentId = assetId(current)
  const edges: Edge[] = [
    ...upstream.map((asset) => ({
      id: `${assetId(asset)}->${currentId}`,
      source: assetId(asset),
      target: currentId,
    })),
    ...downstream.map((asset) => ({
      id: `${currentId}->${assetId(asset)}`,
      source: currentId,
      target: assetId(asset),
    })),
  ]

  return { nodes: [...groupNodes, ...assetNodes], edges }
}

export function LineageGraph({ current }: { current: TableLineageNode }) {
  const { nodes, edges } = createElements(current)

  return (
    <div className="h-[clamp(480px,62vh,680px)] overflow-hidden rounded-lg border bg-muted/20 shadow-inner">
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

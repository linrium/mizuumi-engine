import { GraphQLClient, gql } from "graphql-request"

const TABLE_LINEAGE_QUERY = gql`
  fragment LineageAssetNode on AssetNode {
    assetKey {
      path
    }
    description
    groupName
    kinds
    isMaterializable
    isObservable
    repository {
      name
      location {
        name
      }
    }
    assetMaterializations(limit: 1) {
      runId
      timestamp
    }
    assetObservations(limit: 1) {
      runId
      timestamp
    }
  }

  query TableLineage($assetKey: AssetKeyInput!) {
    assetNodeOrError(assetKey: $assetKey) {
      __typename
      ... on AssetNode {
        ...LineageAssetNode
        dependencies {
          asset {
            ...LineageAssetNode
          }
        }
        dependedBy {
          asset {
            ...LineageAssetNode
          }
        }
      }
      ... on AssetNotFoundError {
        message
      }
    }
  }
`

const ASSETS_QUERY = gql`
  query Assets {
    assetNodes {
      assetKey {
        path
      }
      description
      groupName
      kinds
      isMaterializable
      isObservable
      repository {
        name
        location {
          name
        }
      }
      assetMaterializations(limit: 1) {
        runId
        timestamp
      }
      assetObservations(limit: 1) {
        runId
        timestamp
      }
    }
  }
`

const RUNS_QUERY = gql`
  query Runs {
    runsOrError(limit: 100) {
      __typename
      ... on Runs {
        results {
          runId
          pipelineName
          status
          startTime
          endTime
          updateTime
          repositoryOrigin {
            repositoryName
            repositoryLocationName
          }
          tags {
            key
            value
          }
        }
      }
      ... on PythonError {
        message
      }
    }
  }
`

type AssetKey = {
  path: string[]
}

type AssetEvent = {
  runId: string
  timestamp: string
}

export type DagsterAsset = {
  assetKey: AssetKey
  description: string | null
  groupName: string
  kinds: string[]
  isMaterializable: boolean
  isObservable: boolean
  repository: {
    name: string
    location: {
      name: string
    }
  }
  assetMaterializations: AssetEvent[]
  assetObservations: AssetEvent[]
}

export type DagsterRun = {
  runId: string
  pipelineName: string
  status: string
  startTime: number | null
  endTime: number | null
  updateTime: number
  repositoryOrigin: {
    repositoryName: string
    repositoryLocationName: string
  } | null
  tags: {
    key: string
    value: string
  }[]
}

export type LineageAssetNode = {
  assetKey: AssetKey
  description: string | null
  groupName: string
  kinds: string[]
  isMaterializable: boolean
  isObservable: boolean
  repository: {
    name: string
    location: {
      name: string
    }
  }
  assetMaterializations: AssetEvent[]
  assetObservations: AssetEvent[]
}

export type TableLineageNode = LineageAssetNode & {
  __typename: "AssetNode"
  dependencies: { asset: LineageAssetNode }[]
  dependedBy: { asset: LineageAssetNode }[]
}

type TableLineageData = {
  assetNodeOrError:
    | TableLineageNode
    | {
        __typename: "AssetNotFoundError"
        message: string
      }
}

type TableLineageVariables = {
  assetKey: AssetKey
}

type AssetsData = {
  assetNodes: DagsterAsset[]
}

type RunsData = {
  runsOrError:
    | {
        __typename: "Runs"
        results: DagsterRun[]
      }
    | {
        __typename: "PythonError"
        message: string
      }
}

const dagster = new GraphQLClient(
  process.env.DAGSTER_GRAPHQL_URL ?? "https://api.dagster.mizuumi.test/graphql",
  {
    cache: "no-store",
    errorPolicy: "all",
  },
)

export async function getTableLineage(assetKey: string[]) {
  const response = await dagster.rawRequest<
    TableLineageData,
    TableLineageVariables
  >(TABLE_LINEAGE_QUERY, { assetKey: { path: assetKey } })

  return {
    data: response.data,
    ...(response.errors ? { errors: response.errors } : {}),
    ...(response.extensions ? { extensions: response.extensions } : {}),
  }
}

export async function getAssets(): Promise<DagsterAsset[]> {
  const response = await dagster.rawRequest<AssetsData>(ASSETS_QUERY)

  if (response.errors?.length) {
    throw new Error(response.errors.map((error) => error.message).join("; "))
  }

  return response.data.assetNodes
}

export async function getRuns(): Promise<DagsterRun[]> {
  const response = await dagster.rawRequest<RunsData>(RUNS_QUERY)

  if (response.errors?.length) {
    throw new Error(response.errors.map((error) => error.message).join("; "))
  }

  const result = response.data.runsOrError
  if (result.__typename !== "Runs") {
    throw new Error(result.message)
  }

  return result.results
}

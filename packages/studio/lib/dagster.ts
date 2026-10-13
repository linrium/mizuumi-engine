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
      jobNames
      opName
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

const LAUNCH_ASSET_MATERIALIZATION = gql`
  mutation LaunchAssetMaterialization($executionParams: ExecutionParams!) {
    launchPipelineExecution(executionParams: $executionParams) {
      __typename
      ... on LaunchRunSuccess {
        run {
          runId
        }
      }
      ... on Error {
        message
      }
      ... on RunConfigValidationInvalid {
        errors {
          message
        }
      }
      ... on InvalidStepError {
        invalidStepKey
      }
      ... on InvalidOutputError {
        invalidOutputName
        stepKey
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
  jobNames: string[]
  opName: string | null
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

type LaunchAssetMaterializationData = {
  launchPipelineExecution: {
    __typename: string
    errors?: { message: string }[]
    invalidOutputName?: string
    invalidStepKey?: string
    message?: string
    run?: { runId: string }
    stepKey?: string
  }
}

type LaunchAssetMaterializationVariables = {
  executionParams: {
    selector: {
      assetSelection: AssetKey[]
      jobName: string
      repositoryLocationName: string
      repositoryName: string
    }
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

export async function launchAssetMaterialization({
  assetKeys,
  jobName,
  repositoryLocationName,
  repositoryName,
}: {
  assetKeys: string[][]
  jobName: string
  repositoryLocationName: string
  repositoryName: string
}): Promise<{ runId: string }> {
  const response = await dagster.rawRequest<
    LaunchAssetMaterializationData,
    LaunchAssetMaterializationVariables
  >(LAUNCH_ASSET_MATERIALIZATION, {
    executionParams: {
      selector: {
        assetSelection: assetKeys.map((path) => ({ path })),
        jobName,
        repositoryLocationName,
        repositoryName,
      },
    },
  })

  if (response.errors?.length) {
    throw new Error(response.errors.map((error) => error.message).join("; "))
  }

  const result = response.data.launchPipelineExecution
  if (result.__typename === "LaunchRunSuccess" && result.run) return result.run
  if (result.__typename === "RunConfigValidationInvalid") {
    throw new Error(
      result.errors?.map((error) => error.message).join("; ") ||
        "Dagster rejected the run configuration",
    )
  }
  if (result.__typename === "InvalidStepError") {
    throw new Error(`Invalid step: ${result.invalidStepKey ?? "unknown"}`)
  }
  if (result.__typename === "InvalidOutputError") {
    throw new Error(
      `Invalid output ${result.invalidOutputName ?? "unknown"} for step ${result.stepKey ?? "unknown"}`,
    )
  }
  if (result.__typename === "InvalidSubsetError") {
    throw new DagsterInvalidSubsetError(result.message)
  }

  throw new Error(result.message ?? `Dagster returned ${result.__typename}`)
}

export class DagsterInvalidSubsetError extends Error {
  override name = "DagsterInvalidSubsetError"
}

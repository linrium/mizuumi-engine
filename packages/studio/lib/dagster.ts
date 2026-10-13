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

type AssetKey = {
  path: string[]
}

type AssetEvent = {
  runId: string
  timestamp: string
}

type LineageAssetNode = {
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

type TableLineageData = {
  assetNodeOrError:
    | (LineageAssetNode & {
        __typename: "AssetNode"
        dependencies: { asset: LineageAssetNode }[]
        dependedBy: { asset: LineageAssetNode }[]
      })
    | {
        __typename: "AssetNotFoundError"
        message: string
      }
}

type TableLineageVariables = {
  assetKey: AssetKey
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

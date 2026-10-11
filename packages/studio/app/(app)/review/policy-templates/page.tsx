import { getPolicyTemplate, listPolicyTemplates } from "../actions"
import { PolicyTemplates } from "./policy-templates"

export default async function PolicyTemplatesPage() {
  const listResult = await listPolicyTemplates()
  if (!listResult.ok) {
    return (
      <PolicyTemplates initialTemplates={[]} initialError={listResult.error} />
    )
  }

  const details = await Promise.all(
    listResult.data.map((template) => getPolicyTemplate(template.id)),
  )
  const templates = details.flatMap((result) =>
    result.ok ? [result.data] : [],
  )
  const detailError = details.find((result) => !result.ok)

  return (
    <PolicyTemplates
      initialTemplates={templates}
      initialError={
        detailError && !detailError.ok
          ? `Some template details could not be loaded: ${detailError.error}`
          : undefined
      }
    />
  )
}

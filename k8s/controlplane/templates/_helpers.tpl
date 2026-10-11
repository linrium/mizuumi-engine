{{- define "controlplane.name" -}}
{{- printf "%s" .Release.Name | trunc 52 | trimSuffix "-" -}}
{{- end -}}

{{- define "controlplane.serverName" -}}
{{- printf "%s-server" (include "controlplane.name" .) | trunc 63 | trimSuffix "-" -}}
{{- end -}}

{{- define "controlplane.postgresqlName" -}}
{{- printf "%s-postgresql" (include "controlplane.name" .) | trunc 63 | trimSuffix "-" -}}
{{- end -}}

{{- define "controlplane.serviceAccountName" -}}
{{- if .Values.serviceAccount.create -}}
{{- default (include "controlplane.serverName" .) .Values.serviceAccount.name -}}
{{- else -}}
{{- required "serviceAccount.name is required when serviceAccount.create=false" .Values.serviceAccount.name -}}
{{- end -}}
{{- end -}}

{{- define "controlplane.labels" -}}
app.kubernetes.io/name: controlplane
app.kubernetes.io/instance: {{ .Release.Name }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end -}}

{{- define "controlplane.selectorLabels" -}}
app.kubernetes.io/name: controlplane
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end -}}

{{- define "controlplane.postgresqlHost" -}}
{{- if .Values.postgresql.enabled -}}
{{- include "controlplane.postgresqlName" . -}}
{{- else -}}
{{- required "externalPostgresql.host is required when postgresql.enabled=false" .Values.externalPostgresql.host -}}
{{- end -}}
{{- end -}}

{{- define "controlplane.postgresqlPort" -}}
{{- if .Values.postgresql.enabled -}}5432{{- else -}}{{ .Values.externalPostgresql.port }}{{- end -}}
{{- end -}}

{{- define "controlplane.postgresqlDatabase" -}}
{{- if .Values.postgresql.enabled -}}{{ .Values.postgresql.database }}{{- else -}}{{ .Values.externalPostgresql.database }}{{- end -}}
{{- end -}}

{{- define "controlplane.postgresqlUsername" -}}
{{- if .Values.postgresql.enabled -}}{{ .Values.postgresql.username }}{{- else -}}{{ .Values.externalPostgresql.username }}{{- end -}}
{{- end -}}

{{- define "controlplane.postgresqlSecretName" -}}
{{- if .Values.postgresql.enabled -}}
{{- default (printf "%s-credentials" (include "controlplane.postgresqlName" .)) .Values.postgresql.existingSecretName -}}
{{- else -}}
{{- required "externalPostgresql.existingSecretName is required when postgresql.enabled=false" .Values.externalPostgresql.existingSecretName -}}
{{- end -}}
{{- end -}}

{{- define "controlplane.postgresqlPasswordKey" -}}
{{- if .Values.postgresql.enabled -}}{{ .Values.postgresql.passwordKey }}{{- else -}}{{ .Values.externalPostgresql.passwordKey }}{{- end -}}
{{- end -}}


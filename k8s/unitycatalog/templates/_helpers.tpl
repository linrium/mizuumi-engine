{{- define "unitycatalog.name" -}}
{{- printf "%s" .Release.Name | trunc 52 | trimSuffix "-" -}}
{{- end -}}

{{- define "unitycatalog.serverName" -}}
{{- printf "%s-server" (include "unitycatalog.name" .) | trunc 63 | trimSuffix "-" -}}
{{- end -}}

{{- define "unitycatalog.postgresqlName" -}}
{{- printf "%s-postgresql" (include "unitycatalog.name" .) | trunc 63 | trimSuffix "-" -}}
{{- end -}}

{{- define "unitycatalog.serviceAccountName" -}}
{{- if .Values.serviceAccount.create -}}
{{- default (include "unitycatalog.serverName" .) .Values.serviceAccount.name -}}
{{- else -}}
{{- required "serviceAccount.name is required when serviceAccount.create=false" .Values.serviceAccount.name -}}
{{- end -}}
{{- end -}}

{{- define "unitycatalog.labels" -}}
app.kubernetes.io/name: unitycatalog
app.kubernetes.io/instance: {{ .Release.Name }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end -}}

{{- define "unitycatalog.selectorLabels" -}}
app.kubernetes.io/name: unitycatalog
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end -}}

{{- define "unitycatalog.postgresqlHost" -}}
{{- if .Values.postgresql.enabled -}}
{{- include "unitycatalog.postgresqlName" . -}}
{{- else -}}
{{- required "externalPostgresql.host is required when postgresql.enabled=false" .Values.externalPostgresql.host -}}
{{- end -}}
{{- end -}}

{{- define "unitycatalog.postgresqlPort" -}}
{{- if .Values.postgresql.enabled -}}5432{{- else -}}{{ .Values.externalPostgresql.port }}{{- end -}}
{{- end -}}

{{- define "unitycatalog.postgresqlDatabase" -}}
{{- if .Values.postgresql.enabled -}}{{ .Values.postgresql.database }}{{- else -}}{{ .Values.externalPostgresql.database }}{{- end -}}
{{- end -}}

{{- define "unitycatalog.postgresqlUsername" -}}
{{- if .Values.postgresql.enabled -}}{{ .Values.postgresql.username }}{{- else -}}{{ .Values.externalPostgresql.username }}{{- end -}}
{{- end -}}

{{- define "unitycatalog.postgresqlSecretName" -}}
{{- if .Values.postgresql.enabled -}}
{{- default (printf "%s-credentials" (include "unitycatalog.postgresqlName" .)) .Values.postgresql.existingSecretName -}}
{{- else -}}
{{- required "externalPostgresql.existingSecretName is required when postgresql.enabled=false" .Values.externalPostgresql.existingSecretName -}}
{{- end -}}
{{- end -}}

{{- define "unitycatalog.postgresqlPasswordKey" -}}
{{- if .Values.postgresql.enabled -}}{{ .Values.postgresql.passwordKey }}{{- else -}}{{ .Values.externalPostgresql.passwordKey }}{{- end -}}
{{- end -}}

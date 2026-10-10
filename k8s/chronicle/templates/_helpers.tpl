{{- define "chronicle.name" -}}
{{- printf "%s" .Release.Name | trunc 52 | trimSuffix "-" -}}
{{- end -}}

{{- define "chronicle.mysqlName" -}}
{{- printf "%s-mysql" (include "chronicle.name" .) | trunc 63 | trimSuffix "-" -}}
{{- end -}}

{{- define "chronicle.serviceAccountName" -}}
{{- if .Values.serviceAccount.create -}}
{{- default (include "chronicle.name" .) .Values.serviceAccount.name -}}
{{- else -}}
{{- required "serviceAccount.name is required when serviceAccount.create=false" .Values.serviceAccount.name -}}
{{- end -}}
{{- end -}}

{{- define "chronicle.labels" -}}
app.kubernetes.io/name: chronicle
app.kubernetes.io/instance: {{ .Release.Name }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end -}}

{{- define "chronicle.selectorLabels" -}}
app.kubernetes.io/name: chronicle
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end -}}

{{- define "chronicle.mysqlHost" -}}
{{- if .Values.mysql.enabled -}}
{{- include "chronicle.mysqlName" . -}}
{{- else -}}
{{- required "externalMysql.host is required when mysql.enabled=false" .Values.externalMysql.host -}}
{{- end -}}
{{- end -}}

{{- define "chronicle.mysqlPort" -}}
{{- if .Values.mysql.enabled -}}
3306
{{- else -}}
{{- .Values.externalMysql.port -}}
{{- end -}}
{{- end -}}

{{- define "chronicle.mysqlDatabase" -}}
{{- if .Values.mysql.enabled -}}
{{- .Values.mysql.database -}}
{{- else -}}
{{- .Values.externalMysql.database -}}
{{- end -}}
{{- end -}}

{{- define "chronicle.mysqlUsername" -}}
{{- if .Values.mysql.enabled -}}
{{- .Values.mysql.username -}}
{{- else -}}
{{- .Values.externalMysql.username -}}
{{- end -}}
{{- end -}}

{{- define "chronicle.mysqlSecretName" -}}
{{- if .Values.mysql.enabled -}}
{{- if .Values.mysql.existingSecretName -}}
{{- .Values.mysql.existingSecretName -}}
{{- else -}}
{{- include "chronicle.mysqlName" . -}}
{{- end -}}
{{- else -}}
{{- required "externalMysql.existingSecretName is required when mysql.enabled=false" .Values.externalMysql.existingSecretName -}}
{{- end -}}
{{- end -}}

{{- define "chronicle.mysqlPasswordKey" -}}
{{- if .Values.mysql.enabled -}}
MYSQL_PASSWORD
{{- else -}}
{{- .Values.externalMysql.passwordKey -}}
{{- end -}}
{{- end -}}

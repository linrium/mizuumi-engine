{{- define "chronicle-ui.name" -}}
{{- printf "%s" .Release.Name | trunc 52 | trimSuffix "-" -}}
{{- end -}}

{{- define "chronicle-ui.labels" -}}
app.kubernetes.io/name: chronicle-ui
app.kubernetes.io/instance: {{ .Release.Name }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end -}}

{{- define "chronicle-ui.selectorLabels" -}}
app.kubernetes.io/name: chronicle-ui
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end -}}

{{- define "chronicle-ui.serviceAccountName" -}}
{{- if .Values.serviceAccount.create -}}
{{- default (include "chronicle-ui.name" .) .Values.serviceAccount.name -}}
{{- else -}}
{{- required "serviceAccount.name is required when serviceAccount.create=false" .Values.serviceAccount.name -}}
{{- end -}}
{{- end -}}

{{- define "dagster-umbrella.sparkAccessRoleName" -}}
{{ .Release.Name }}-spark-trigger
{{- end -}}

{{- define "dagster-umbrella.serviceAccountName" -}}
{{- default (printf "%s-dagster" .Release.Name) .Values.dagster.global.serviceAccountName -}}
{{- end -}}

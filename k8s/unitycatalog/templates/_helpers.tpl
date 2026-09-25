{{- define "unitycatalog.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" -}}
{{- end -}}

{{- define "unitycatalog.fullname" -}}
{{- if .Values.fullnameOverride -}}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" -}}
{{- else if contains (include "unitycatalog.name" .) .Release.Name -}}
{{- .Release.Name | trunc 63 | trimSuffix "-" -}}
{{- else -}}
{{- printf "%s-%s" .Release.Name (include "unitycatalog.name" .) | trunc 63 | trimSuffix "-" -}}
{{- end -}}
{{- end -}}

{{- define "unitycatalog.labels" -}}
helm.sh/chart: {{ printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | quote }}
app.kubernetes.io/name: {{ include "unitycatalog.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end -}}

{{- define "unitycatalog.serverSelectorLabels" -}}
app.kubernetes.io/name: {{ include "unitycatalog.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
app.kubernetes.io/component: server
{{- end -}}

{{- define "unitycatalog.serverName" -}}
{{- printf "%s-server" (include "unitycatalog.fullname" .) | trunc 63 | trimSuffix "-" -}}
{{- end -}}

{{- define "unitycatalog.pvcName" -}}
{{- default (printf "%s-data" (include "unitycatalog.fullname" .)) .Values.server.persistence.existingClaim -}}
{{- end -}}

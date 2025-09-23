{{/*
Expand the name of the chart.
*/}}
{{- define "zeta-reticula.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
We truncate at 63 chars because some Kubernetes name fields are limited to this (by the DNS naming spec).
If release name contains chart name it will be used as a full name.
*/}}
{{- define "zeta-reticula.fullname" -}}
{{- if .Values.fullnameOverride }}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- $name := default .Chart.Name .Values.nameOverride }}
{{- if contains $name .Release.Name }}
{{- .Release.Name | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}
{{- end }}

{{/*
Create chart name and version as used by the chart label.
*/}}
{{- define "zeta-reticula.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "zeta-reticula.labels" -}}
helm.sh/chart: {{ include "zeta-reticula.chart" . }}
{{ include "zeta-reticula.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{/*
Selector labels
*/}}
{{- define "zeta-reticula.selectorLabels" -}}
app.kubernetes.io/name: {{ include "zeta-reticula.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
Create the name of the service account to use
*/}}
{{- define "zeta-reticula.serviceAccountName" -}}
{{- if .Values.serviceAccount.create }}
{{- default (include "zeta-reticula.fullname" .) .Values.serviceAccount.name }}
{{- else }}
{{- default "default" .Values.serviceAccount.name }}
{{- end }}
{{- end }}

{{/*
Return the proper image name
*/}}
{{- define "zeta-reticula.image" -}}
{{- printf "%s:%s" .Values.zetaReticula.image.repository (.Values.zetaReticula.image.tag | default .Chart.AppVersion) }}
{{- end }}

{{/*
Return the proper image pull policy
*/}}
{{- define "zeta-reticula.imagePullPolicy" -}}
{{- .Values.zetaReticula.image.pullPolicy | default "IfNotPresent" }}
{{- end }}

{{/*
Create a default fully qualified ingress host name.
*/}}
{{- define "zeta-reticula.ingressHost" -}}
{{- if .Values.ingress.hosts }}
{{- index .Values.ingress.hosts 0 }}
{{- else }}
{{- printf "%s.local" .Release.Name }}
{{- end }}
{{- end }}

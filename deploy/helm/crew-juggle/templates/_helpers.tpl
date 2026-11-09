{{- define "crew-juggle.name" -}}{{ .Release.Name }}{{- end -}}
{{- define "crew-juggle.labels" -}}
app.kubernetes.io/name: crew-juggle
app.kubernetes.io/instance: {{ .Release.Name }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end -}}
{{- define "crew-juggle.databaseUrl" -}}
{{- if .Values.databaseUrl -}}
{{ .Values.databaseUrl }}
{{- else -}}
postgres://juggle:{{ .Values.postgres.password }}@{{ include "crew-juggle.name" . }}-db:5432/juggle
{{- end -}}
{{- end -}}

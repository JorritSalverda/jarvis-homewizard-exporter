## Installation

To install this application using Helm run the following commands: 

```bash
helm repo add jorritsalverda https://helm.jorritsalverda.com
kubectl create namespace jarvis-homewizard-exporter

helm upgrade \
  jarvis-homewizard-exporter \
  jorritsalverda/jarvis-homewizard-exporter \
  --install \
  --namespace jarvis-homewizard-exporter \
  --set secret.gcpServiceAccountKeyfile='{abc: blabla}' \
  --wait
```

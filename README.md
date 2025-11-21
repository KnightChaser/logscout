# logscout

## Example configuration file format

```yaml
# config.yaml
follow: true

include:
  - "ERROR"
  - "CRITICAL"
  - "CREATE"
  - "DELETE"

exclude:
  - "healthcheck"
  - "metrics"
  - ".swp"

sources:
  - name: "auditd"
    type: "file"
    path: "/var/log/audit/audit.log"

  - name: "tmp-inotify"
    type: "command"
    command: "inotifywait"
    args: ["-m", "-r", "/tmp"]

```

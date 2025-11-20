# logscout

## Example configuration file format

```yaml
# config.yaml
follow: true

include:
  - "ERROR"
  - "CRITICAL"

exclude:
  - "healthcheck"
  - "metrics"

sources:
  - name: "pacman"
    path: "/var/log/pacman.log"

  - name: "nginx-access"
    path: "/var/log/nginx/access.log"

  - name: "nginx-error"
    path: "/var/log/nginx/error.log"
```

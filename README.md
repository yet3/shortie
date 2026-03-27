# Shortie

Like [espanso](https://espanso.org/) but worse in every way. I'm just learning rust.

```bash
shortie start
shortie stop
shortie reload
shortie status
```

`.config/shortie/config.yaml` (supports multiple config files)
```yaml
prefix: ";"
shorts:
  - name: "l3"
    output: "localhost:3000"
  - name: "l4"
    output: "localhost:4321"
  - name: "l5"
    output: "localhost:5173"
```
```bash
;l3 -> localhost:3000 
;l4 -> localhost:4321 
;l5 -> localhost:5173 
```

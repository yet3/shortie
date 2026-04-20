# Shortie

A very simple text expander. Kind of like [espanso](https://espanso.org/) but worse in every way.

![Example gif](./public/example.gif)

#### shortie-cli
```bash
Usage: shortie <COMMAND>

Commands:
  start   Start shortie-daemon
  stop    Stop shortie-daemon
  reload  Reload shortie-daemon
  status  See status of shortie-daemon
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

#### shortie start
```
Usage: shortie start [OPTIONS]

Options:
  -c, --config <CONFIG>  Path to the directory containing .yaml config files 
  -p, --pid <PID>        Path to the directory containing temporary .pid file
  -h, --help             Print help
```

#### shortie-daemon
```bash
Usage: shortied --config <CONFIG>

Options:
  -c, --config <CONFIG>  Path to the directory containing .yaml config files
  -h, --help             Print help
```

### Functions
- `embed <FILE_PATH>`
- `now <FORMAT?>`
- `var <NAME>`

### Example Config 
`.config/shortie/config.yaml` (supports multiple config files)
```yaml
prefix: ";"
vars:
  - name: "name"
    value: "Max"
shorts:
  - name: "l3"
    content: "localhost:3000"
  - name: "l4"
    content: "localhost:4321"
  - name: "l5"
    content: "localhost:5173"
  - name: "time"
    content: "{now %H:%M:%S}"
  - name: "em1"
    content: "{embed ./templates/email_1.txt}"
  - name: "intro"
    content: "{var start} {var name}"
    vars:
      - name: "start"
        value: "My name is"

```

### Example Usage
```bash
shortie start
```

```
;l3 -> localhost:3000 
;l4 -> localhost:4321 
;l5 -> localhost:5173 
```

# alt

`alt` is an extension to [ktrl](https://github.com/ItayGarin/ktrl). 
It listens to system events such as window-focus changes, Emacs hooks, etc...
These events are aggregated and sent over to `ktrl` via its `ipc` interface.

## Usage

```
alt 0.1
Itay G. <thifixp@gmail.com>
An Event Aggregator for ktrl

USAGE:
    alt [FLAGS] [OPTIONS]

FLAGS:
        --debug      Enables debug level logging
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --cfg <CONFIG>     Path to your alt config file. Default: .alt.ron
        --log <LOGFILE>    Path to the log file. Default: .alt.log```
```

## Install

- Install `alt`

```
cargo install --path .
```

- Setup your `~/.alt.ron` file (See my example below).
- Add this to your window manager autostart (ex: i3) -

```
exec ~/.cargo/bin/alt
```

## Configuration

The default config file path is `~/.alt.ron`. 
You can override this with the `--cfg` cli argument.

### Example

```
(
  aggs: [
  // Ivy Event Aggregator
    (requirements: [
        RqFocus("emacs"),
        RqExtEvent("ivy"),
    ],
    on_ipc: "TurnOnLayerAlias(\"ivy\")",
    off_ipc: "TurnOffLayerAlias(\"ivy\")"),
  ],
)
```

This will toggle the `ivy` ktrl layer when Emacs is in foucs AND `ivy` is active

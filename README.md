<div align="center">

# Walrus
##### Wallpaper manager for [swww](https://github.com/LGFae/swww)

</div>

## Contents
* [Why?](#why)
* [Features](#features)
* [Usage](#usage)
* [Configuration](#configuration)
* [Build](#build)
* [Roadmap](#roadmap)

## Why?
I wanted to make using [swww](https://github.com/LGFae/swww) simpler.

## Features
- Simple configuration via TOML
- Plug and Play — although you will likely need to configure the path to your wallpapers
- Cycle through wallpapers
- Pause & Resume playback

## Usage
Simply start the program with:
```
walrus
```

A list of commands can be found by running
```
walrus help
```

## Configuration
The following are the default configuration values. The configuration file is located at `$HOME/.config/walrus/config.toml`
```TOML
[general]
debug = "info"
interval = 300
resolution = { width = x, height = y } # Automatically inferred, but possible to configure. Used for dynamic duration.
shuffle = true
swww_path = "/usr/bin/swww"
wallpaper_path = "~/Pictures/Wallpapers"

[transition]
bezier = [0.40, 0.0, 0.6, 1.0]
duration = 1.0
dynamic_duration = true # Changes the transition duration based on pixels travelled.
fill = "000000"
filter = "Lanczos3"
flavour = ["wipe", "wave", "grow", "outer"]
fps = refresh_rate # Automatically inferred based on highest refresh rate monitor.
resize = "crop"
step = 60
wave_size = [55, 60, 45, 50]
```

**NOTE**: if no configuration is found the program will use these defaults.

## Build
For now build from source.

## Roadmap
- [x] Manual wallpaper cycling (next & previous commands)
- [x] Configuration options for advanced features (such as min/max wave size & transition bezier)
- [x] Config hot reloading
- [ ] Advanced wallpaper scheduling, depending on time of day, etc.
- [ ] Optionally independent configuration per transition type

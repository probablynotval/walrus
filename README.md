<div align="center">

# Walrus
##### Wallpaper manager for [swww](https://github.com/LGFae/swww)

</div>

## Contents
* [Why?](#why)
* [Features](#features)
* [Usage](#usage)
* [Configuration](#configuration)
* [Roadmap](#roadmap)

## Why?
I wanted to make using [swww](https://github.com/LGFae/swww) slightly simpler. For now this is just a glorified script with a CLI.

## Features
- Configuration via TOML
- Plug and Play — although you will likely need to configure the path to your wallpapers

## Usage
Simply start the program with:
```
walrus init
```

## Configuration
The following are the default configuration values. The configuration file is located at `$HOME/.config/walrus/config.toml`
```TOML
[general]
interval = 300
path = "$HOME/Pictures/Wallpapers"
shuffle = true

[transition]
duration = 0.75
fill = "000000"
filter = "Lanczos3"
fps = 60
step = 160
resize = "crop"
```

**NOTE**: if no configuration is found the program will use these defaults.

## Roadmap
- [ ] Manual wallpaper cycling (next & previous commands)
- [ ] Advanced wallpaper scheduling, depending on time of day, etc.
- [ ] Configuration options for advanced features (such as min/max wave size & transition bezier)
- [ ] Optionally independent configuration per transition type

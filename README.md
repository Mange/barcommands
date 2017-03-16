# barcommands

Commands to run in i3blocks to get concise information about your system.

These commands will output colors from [Gruvbox](https://github.com/morhetz/gruvbox) and use icons from Font Awesome.

## Installation

Install by running `cargo install` in the project root. The binaries will then be added to your `~/.cargo/bin` directory.

## Commands

### `bar-cpu`

Prints CPU usage as a simple percent with [Font Awesome's cog icon](http://fontawesome.io/icon/cog/) in front continuously .

It will change to yellow and then to red as usage goes up.

```
[cpu_usage]
command=bar-cpu
interval=persist
markup=pango
```

### `bar-memory`

Prints memory usage as a simple percent with [Font Awesome's microchip icon](http://fontawesome.io/icon/microchip/) in front.

It will change to yellow and then to red as usage goes up. Click on it to temporarily see the numbers behind the percent.

```
[memory]
command=bar-memory
interval=10
markup=pango
```

## License

Copyright © Magnus Bergmark <magnus.bergmark@gmail.com> 2017. Code released under MIT license.

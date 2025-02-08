# Universal Bootloader

This package implements a bootloader that will load scripts using several different
methods.

## How to specify which script to load

You can specify the script to load in two ways:

* Change the nick of the computer cabinet to the name of the script you want to
run.  Anything after a `#` character in the nick will be ignored.
* Edit the bootloader code to set `init = "name-of-script.lua"` at line 2.

## How the bootloader looks for the script

The bootloader will first look for any drives in the cabinet.  If a drive is
found containing the file specified then it will load the file from that drive.

If no drive is found containing the requested file then the file will be requested
from the network using the net-boot protocol on port 8.

## Loading multiple modules

If you want to split your scripts up into multiple Lua files then you can use
the bootloader to load them.  Suppose you your code split into two files,
`main.lua` and `display.lua`.  Then you can start `main.lua` with:

```lua
bootloader:loadModule("display.lua")
```

This will load and execute `display.lua`.  Note that `display.lua` should only
contain declarations; it should (normally) run and end once all the declarations
are complete so that the code in `main.lua` can then run.
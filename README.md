# gnss-mgr

![Rust](https://github.com/renestraub/ubxlib_rust/workflows/Rust/badge.svg)
![Linux-x64](https://github.com/renestraub/ubxlib_rust/workflows/Compile%20and%20Test%20Linux-x64/badge.svg)
![Linux-armv7](https://github.com/renestraub/ubxlib_rust/workflows/Cross-compile%20Linux-armv7/badge.svg)

u-blox gnss mangement tool (and Rust library).

_A more elaborate description will follow later._


## Quick Start

`gnss-mgr` is a management tool to control and configure u-blox NEO-M8x modems.

See the command help for more information

```
$ gnss-mgr --help
gnss manager utility 0.3.7
Operates and configures u-blox NEO GNSS modems

USAGE:
    gnss-mgr [FLAGS] <device> [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -q               Be quiet, only show warnings and errors
    -V, --version    Prints version information
    -v               Be verbose, show debug output

ARGS:
    <device>    local serial device to which GNSS modem is connected (e.g. /dev/gnss0)

SUBCOMMANDS:
    config     Configures GNSS modem
    control    Performs GNSS modem control function
    help       Prints this message or the help of the given subcommand(s)
    init       Initializes GNSS
    sos        Save on shutdown operations
```


## Examples

The following examples assume your GNSS modem is availabe under `/dev/gnss0` (most likely a symlink to a TTY device like `/dev/ttyS3`).


### Initialize and get Modem Information

This subcommand detects the current modem bitrate (either 9600 or 115200) and changes it to 115200 bps. Always run the `init` subcommand once before any others.

```
./gnss-mgr /dev/gnss0 init
```

Modem information can be found in `/run/gnss/gnss0.conf`

```
Vendor:                             ublox
Model:                              NEO-M8x
Firmware:                           ADR 4.31
ubx-Protocol:                       19.20
Supported Satellite Systems:        GPS;GLO;GAL;BDS
Supported Augmentation Services:    SBAS;IMES;QZSS
SW Version:                         EXT CORE 3.01 (e3981c)
HW Version:                         00080000
```


### Configure Modem

```
./gnss-mgr /dev/gnss0 config
```


The following configuration file is parsed by the `config` subcommand. Default location is `/etc/gnss/gnss0.conf` assuming the device name is `gnss0`. Arbitratry locations can be specified via the `-f, --file` option.

```
[default]
# Indicates the version of this config file, it should not be modified.
version=2

# Select measurement and navigation output rate
# Allowed values : 1, 2  [Hz]
update-rate=
#update-rate=1


#
# Navigation settings
#
[navigation]

# Selects dynamic mode
# Supported values:
#   stationary, vehicle
mode=
#mode=vehicle

#
# Selects GNSS systems
# Allowed values:
#   GPS;GLONASS;SBAS
#   GPS;Galileo;Beidou;SBAS
#systems=
#systems=GPS;GLONASS;SBAS
systems=GPS;Galileo;Beidou;SBAS


#
# Installation settings
# For details on this section, see the relevant documentation
#
[installation]

#
# IMU orientation in degrees [°]
#   yaw: value in degrees (0 to  360)
#   pitch: value in degrees (-90 to  90)
#   roll: value in degrees (-180 to 180)
yaw=
pitch=
roll=

# Lever arm lengths in meters [m]
# Format x;y;z
# Example:
#   vrp2antenna=1.0;1.5;0.3
vrp2antenna=
vrp2imu=
```


### Perform a Cold Start

This will request a cold start of the receiver.

```
./gnss-mgr /dev/gnss0 control cold-start
```


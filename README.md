# Npkg

Npkg is a small tool that allows you to configure all your NixOS packages in one place.

Npkg is written with rust and uses [nix-editor](https://github.com/vlinkz/nix-editor) and [rnix-parser](https://github.com/nix-community/rnix-parser) to parse configuration files.

# WARNING

This tool is still new/experimental, and being that it directly modifies critical files, such as `/etc/nixos/configuration.nix`, make sure you have backups in case it messes it up or deletes such files. I've already sacrificed some of my files to the void, don't let that happen to you!

# NixOS Installation

```
git clone https://github.com/vlinkz/npkg
nix-env -f npkg -i npkg
```

# Usage with Nix Flakes

```
nix run github:vlinkz/npkg -- --help
```

# Arguments

```
USAGE:
    npkg [OPTIONS] [PACKAGES]...

ARGS:
    <PACKAGES>...    Packages

OPTIONS:
    -d, --dry-run            Do not build any packages, only edit configuration file
    -E, --env                Use nix environment 'nix-env'
    -h, --help               Print help information
    -H, --home               Use home-manager 'home.nix'
    -i, --install            Install a package
    -l, --list               List installed packages
    -o, --output <OUTPUT>    Output modified configuration file to a specified location
    -r, --remove             Remove a package
    -s, --search             Search for a package
    -S, --system             Use system 'configuration.nix'
    -V, --version            Print version information
```

# Use cases

## Installing packages

To install a package, you can run:
```
npkg -i <PACKAGE>
```
By default, this will use `nix-env` and install the package in you current environment. You can choose to use a specific available installer by using the `-S`, `-H`, or `-E` flags.

-   ```
    sudo npkg -iS hello
    ```
    will install the `hello` package as a system package by modifying your `/etc/nixos/configuration.nix` file and then calling `nixos-rebuild switch`.

-   ```
    npkg -iH hello
    ```
    will install the `hello` package using [home-manager](https://github.com/nix-community/home-manager) if it is installed. It will modify `~/.config/nixpkgs/home.nix` and then call `home-manager switch`.

-   ```
    sudo npkg -iE hello
    ```
    will install the `hello` package to the current nix environment by calling `nix-env -iA nixos.hello`.

## Removing packages

Very similar to installing packages:
```
npkg -r <PACKAGE>
```
The same `-S`, `-H`, and `-E` flags apply.

## List installed packages

```
npkg -l
```
This will list all packages installed in `/etc/nixos/configuration.nix`, `~/.config/nixpkgs/home.nix`, and with `nix-env`.

You can specify only one of these by using the `-S`, `-H`, and `-E` flags.

## Search for a package
```
npkg -s <QUERY>
```
This will print a list of packages that match the query specified. For example:
```
$ npkg -s hello greeting

* hello (2.12)
  A program that produces a familiar, friendly greeting
```

# Configuration

A configuration file is stored in `~/.config/npkg/config.json`, by default, it contains:

```json
{
  "systemconfig": "/etc/nixos/configuration.nix",
  "homeconfig": "/home/$HOME/.config/nixpkgs/home.nix",
  "flake": null
}
```

These values can be edited to point to other locations. This is useful in [nix flake based systems](https://nixos.wiki/wiki/Flakes#Using_nix_flakes_with_NixOS) or any system where config files are not in expected locations.

# But why?

I wanted to code something as a proof of concept for using [nix-editor](https://github.com/vlinkz/nix-editor) as a backed for other tools. Plus I had nothing better to do on a Saturday. Although given my limited time using this, I think I'll stick to it for a while.

# Future plans

- Add update command: `npkg -u`
    - Maybe update channels as well? :eyes:

- Check for installed packages in other locations.
    
    For example, if installing `hello` with `home-manager`, and it's already installed with `nix-env`, give an option to switch it over.

- When removing packages, automatically detect where installed instead of defaulting to `nix-env`

- Whatever else pops into my head
# Warpgate target

Basic cli to get and use the targets of warpgate.

## Installation

git clone this repo

```sh
git clone https://github.com/connorvoiseyexe-squared/wg-target
cd wg-target
cargo install --path $(pwd)
```

## Completion

zsh, add to `~/.zshrc`:

```sh
source <(COMPLETE=zsh wg-target)
```

bash, add to `~/.bashrc`:

```sh
source <(COMPLETE=bash wg-target)
```

## Usage

```sh
wg-target -h

#A really basic CLI to select a Warpgate target and ssh to it
#
#Usage: wg-target [OPTIONS] <COMMAND>
#
#Commands:
#  init  write the config file, prompting for each value
#  ssh   select a target and open an ssh session to it
#  sftp  select a target and open an sftp session to it
#  list  print the target labels, one per line
#  help  Print this message or the help of the given subcommand(s)
#
#Options:
#  -v, --verbose...  log verbosity: -v info, -vv debug, -vvv trace (overrides RUST_LOG)
#  -h, --help        Print help
#  -V, --version     Print version
```

Run init first, this will write a config file in your home directory.
Then you can do ssh or whatever after that.

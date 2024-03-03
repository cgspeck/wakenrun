# Wake And Run

Utility to wake a remote machine, sequentially run commands locally or remotely via ssh, then shut that remote machine down.

You will need to set up:

  * wake on lan
  * ssh with key files

Usage:

```shell
wakenrun --help
Usage: wakenrun [OPTIONS] <FILE>

Arguments:
  <FILE>  Config file to execute

Options:
  -g, --generate  Write a sample config to FILE
  -h, --help      Print help
  -V, --version   Print version

```

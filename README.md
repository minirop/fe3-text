# FE3 text

Compiles/Decompiles dialogue scripts and list of strings from Fire Emblem 3.

## Usage

### Decompile a dialogue

```console
$ fe3-text <file> decompile dialogue [-o <offset>]
```

### Compile a dialogue (not implemented)

```console
$ fe3-text <file> compile dialogue -o <output>
```

### Print a list of strings

```console
$ fe3-text <file> decompile list [-s <start offset>] -e <end offset>
```

### Compile a list of string

```console
$ fe3-text <file> compile list -o <output>
```
`file` is a text file with one string per line.

## TODO

- Not all kanjis have been mapped, so most will show up as as a full stop.
- Understand the unknown commands.
- Move the mapping into a config file.

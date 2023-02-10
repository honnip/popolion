# Popolion

A cli tool for ipf archive.

## Features

- Extract ipf archive
- Decrypt ies file (this is done by default)
- List files of ipf archive (soon)

Try `popo -h` to see all available commands, `popo extract -h` to see usage and
options for extract command.

## Examples

Extract

```shell
# Extract to current directory
popo extract example.ipf
popo extract example.ipf another.ipf
popo extract *.ipf

# Extract to sub-directory (with same name as filename)
popo extract -s example.ipf
```

Extract to specific directory:

```shell
# If the file already exists, it will be overwritten by default
popo extract path/to/example.ipf -d to/extracted
```

Extract files except those that match the expression:

```shell
# --exclude option takes glob pattern
popo extract path/to/example.ipf -e "*.jpg"

# also multiple patterns
popo extract example.ipf -e "*.jpg" -e "some.ipf/*"
```

Don't overwrite files

```shell
popo extract example.ipf -n
```

# Popolion

A cli tool for ipf archive.

## Features

- Extract ipf archive
- Decrypt ies file (this is done by default)
- List files of ipf archive

Try `popo -h` to see all available commands, `popo extract -h` to see usage and
options for extract command.

## Examples

List files

```shell
popo list path/to/example.ipf
```

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
popo extract -x "*.jpg" path/to/example.ipf

# also multiple patterns
popo extract -x "*.jpg" -x "some.ipf/*" example.ipf

# extract only lua files
popo extract -x "*.[!l][!u][!a]" example.ipf  # can you suggest any improvements here?
```

Never overwrite files:

```shell
popo extract -n example.ipf
```

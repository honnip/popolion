# Popolion

A cli tool for ipf archive.

Download pre-built binaries from
[Releases](https://github.com/honnip/popolion/releases)

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
popo extract *.ipf # on Windows (cmd and powershell), wildcard is not supported. Try nushell or WSL

# Extract to sub-directory (with same name as filename)
popo extract -s example.ipf  # example/...
```

Extract to specific directory:

```shell
# If the file already exists, it will be overwritten by default
popo extract path/to/example.ipf -d to/extracted
```

Extract files except those that match the expression:

```shell
# --exclude option takes glob pattern
popo extract path/to/example.ipf -x "*.png"

# also multiple patterns
popo extract example.ipf -x "*.jpg" -x "some.ipf/*"

# extract only LUA files
popo extract example.ipf -x "!*.lua"
# ..or LUA and XML files
popo extract example.ipf -x "!.{lua,xml}"
```

Never overwrite files:

```shell
popo extract -n example.ipf
```

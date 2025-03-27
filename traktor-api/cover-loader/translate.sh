#!/usr/bin/env bash

path="$1"
path=${path//\\/\/}
path=${path/Z:/}
path=${path/C:/"$HOME/.wine/drive_c"}

echo -n $path


#!/bin/bash

# stash arguments from settings.sh
STASH_ARGS=( "$@" )
set --
. /root/qnx800/qnxsdp-env.sh
set -- "${STASH_ARGS[@]}"

# run requested command
eval "$@"

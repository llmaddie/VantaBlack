savedcmd_vantablack.mod := printf '%s\n'   vantablack.o | awk '!x[$$0]++ { print("./"$$0) }' > vantablack.mod

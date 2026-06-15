savedcmd_vantablack.mod := printf '%s\n'   main.o | awk '!x[$$0]++ { print("./"$$0) }' > vantablack.mod

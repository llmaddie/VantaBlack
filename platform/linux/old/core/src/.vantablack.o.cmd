savedcmd_vantablack.o := ld.lld -m elf_x86_64 --fatal-warnings -z noexecstack   -r -o vantablack.o @vantablack.mod  ; /home/mad/linux/tools/objtool/objtool --hacks=jump_label --hacks=noinstr --hacks=skylake --ibt --orc --retpoline --rethunk --static-call --uaccess --prefix=16  --link  --module vantablack.o

vantablack.o: $(wildcard /home/mad/linux/tools/objtool/objtool)

obj-m += vantablack.o
vantablack-y := main.o helpers.o

# ============================================
# CONFIG
# ============================================

KDIR ?= /home/mad/linux
PWD := $(shell pwd)

.PHONY: all clean load unload

all:
	$(MAKE) -C $(KDIR) M=$(PWD) LLVM=1 modules

clean:
	$(MAKE) -C $(KDIR) M=$(PWD) clean

load: vantablack.ko
	sudo insmod vantablack.ko
	sudo dmesg | tail -15

unload:
	sudo rmmod vantablack
	sudo dmesg | tail -5
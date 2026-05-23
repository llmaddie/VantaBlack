KDIR ?= /lib/modules/$(uname -r)/build
PWD := $(shell pwd)

.PHONY: all clean load unload

all:
	$(MAKE) -C $(KDIR) M=$(PWD) modules

clean:
	$(MAKE) -C $(KDIR) M=$(PWD) clean

load: vantablack.ko
	sudo insmod vantablack.ko
	sudo dmesg | tail -15

unload:
	sudo rmmod vantablack
	sudo dmesg | tail -5
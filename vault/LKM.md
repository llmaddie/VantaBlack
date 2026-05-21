 stand for Loadable Kernel Module it is an executable library thats extends the capacibilites of a running kernel
 **LKMs** are typically used to add suport for new hardware or for adding system calls

When a functionality provided by an **LKM** is no longer required, it can be unloaded at any times in order  to free memory 

LKM are load using ***modprobe*** command they are located in */lib/modules* or */usr/lib/modules* and have .ko extension. The ***lsmod*** commands lists ithe loaded kernel modules


we wanna make a Loadable Kernel module For Linux

Refers to : 
- [Loadable Kernel Modules wiki](https://en.wikipedia.org/wiki/Loadable_kernel_module)
- [example of Linux LKM](https://github.com/felsocim/LKM)
- [Linux Kernel Module Guide](https://sysprog21.github.io/lkmpg/)
For documentation.

Our Linux KM was designed like that : 

![[Pasted image 20260515212722.png]]
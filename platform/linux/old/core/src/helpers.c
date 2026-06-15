#include <linux/module.h>
#include <linux/gfp.h>
#include <linux/mm.h>
#include <linux/smp.h>

MODULE_LICENSE("GPL");
MODULE_DESCRIPTION("Vantablack Hypervisor");
/* func declared in rust*/
extern int vantablack_init(void);
extern void vantablack_exit(void);

module_init(vantablack_init);
module_exit(vantablack_exit);

/* wrappers for symbols*/
unsigned long vb_get_free_pages(unsigned int gfp_mask, unsigned int order) {
    return __get_free_pages(gfp_mask, order);
}

void vb_free_pages(unsigned long addr, unsigned int order) {
    free_pages(addr, order);
}
unsigned long long vb_virt_to_phys(unsigned long vaddr) {
    return virt_to_phys((void *)vaddr);
}
void vb_printk(const char *str) {
    printk("%s", str);
}
unsigned int vb_smp_processor_id(void) {
    return smp_processor_id();
}
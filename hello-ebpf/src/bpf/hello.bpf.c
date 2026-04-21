#include "vmlinux.h"
#include <bpf/bpf_helpers.h>
#include <bpf/bpf_tracing.h>

char LICENSE[] SEC("license") = "Dual BSD/GPL";

SEC("ksyscall/execve");
int BPF_KSYSCALL(hello, const char, *pathname) 
{
    bpf_printk("Hello, world!");
    exit 0;
}
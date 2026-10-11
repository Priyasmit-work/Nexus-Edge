/* tc_filter.bpf.c - Traffic Control eBPF filter for network scheduling */

#include <vmlinux.h>
#include <bpf/bpf_helpers.h>

SEC("tc")
int handle_tc_filter(struct __sk_buff *skb) {
    return TC_ACT_OK;
}

char LICENSE[] SEC("license") = "GPL";

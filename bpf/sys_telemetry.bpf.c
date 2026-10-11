/* sys_telemetry.bpf.c - eBPF kernel probes for system telemetry */

#include <vmlinux.h>
#include <bpf/bpf_helpers.h>

SEC("tracepoint/sched/sched_switch")
int handle_sched_switch(void *ctx) {
    return 0;
}

char LICENSE[] SEC("license") = "GPL";

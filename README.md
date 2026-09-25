# NexusEdge: Autonomous Offline-First Edge Compute & Content Swarming Engine

[![License: MIT/Apache-2.0](https://img.shields.io/badge/License-MIT%2FApache--2.0-blue.svg)](#license)
[![Rust: 1.75+](https://img.shields.io/badge/Rust-1.75%2B-orange.svg)](#prerequisites)
[![Go: 1.22+](https://img.shields.io/badge/Go-1.22%2B-00ADD8.svg)](#prerequisites)
[![Linux Kernel: 6.x](https://img.shields.io/badge/Linux%20Kernel-6.x-green.svg)](#prerequisites)

> **NexusEdge** is a zero-dependency, low-footprint Rust/Go daemon that unifies Content-Addressed Storage (CAS) swarming, eBPF hardware telemetry, offline-first Delta-State CRDT execution queues, and Multipath QUIC transport into a self-healing edge compute mesh.

---

## 📌 Executive Overview & Problem Statement

Modern edge computing environments—such as remote research facilities, autonomous fleets, retail edge servers, and field operations—face two severe infrastructure bottlenecks:

1. **Heavy Payload Distribution Bottlenecks:** Distributing multi-gigabyte payloads (AI model weights like GGUF/Safetensors, raw video streams, container layers) from central cloud registries over constrained WAN links creates severe bandwidth bottlenecks, high egress costs, and single points of failure.
2. **Network Instability & Controller Dependency:** Existing orchestrators (Kubernetes, Ray, Celery) rely on continuous TCP connection to central master nodes or consensus databases (`etcd`, Redis). When network splits or ISP outages occur, task queues freeze, remote jobs fail, and worker nodes drop off the control plane.

**NexusEdge** eliminates the cloud master dependency. It models both **data assets** (model weights, container layers, video chunks) and **compute tasks** (WASM functions, ffmpeg transcoding, model inference) as **cryptographically signed, content-addressed primitives synchronized via state-based CRDTs**.

### Why Existing Tools Fall Short

| Paradigm | Exemplar Tools | Fundamental Limitation in Edge/Offline Workloads |
| :--- | :--- | :--- |
| **Container Orchestration** | Kubernetes (`k8s`/`k3s`) | Relies on `etcd` Raft consensus requiring strict odd-node quorum. Disconnecting >50% of nodes freezes the control plane. Consumes ~1 GB+ RAM for system daemons. |
| **Distributed Python / Queues** | Ray, Celery | Assumes reliable TCP streams to a centralized Head Node or Redis message broker. Network splits cause lost state, worker drops, and unrecoverable task failures. |
| **P2P File Synchronization** | Syncthing, BitTorrent | Optimized strictly for static file sync. Zero awareness of compute sandboxes, CPU/GPU hardware capabilities, process isolation, or task dependency queues. Memory bloat on large indexing tables. |
| **Point-to-Point Transport** | SSH, SCP, Rsync | Unicast single-threaded TCP streams that collapse on IP roaming or connection drop. Requires heavy rolling checksum scans across both endpoints with zero peer swarming. |

---

## 🏗 System Architecture & Workflow

NexusEdge operates as an autonomous mesh node containing five core subsystems:

```
┌───────────────────────────────────────────────────────────────────────────────────┐
│                               NEXUSEDGE DAEMON NODE                               │
│                                                                                   │
│  ┌────────────────────────┐    ┌────────────────────────┐    ┌─────────────────┐  │
│  │    Linux eBPF Kernel   │    │  FastCDC Deduplicated  │    │  WASM / cgroup  │  │
│  │   Telemetry & Metrics  │    │ CAS Storage Engine     │    │  Execution Engine│  │
│  └───────────┬────────────┘    └───────────┬────────────┘    └────────┬────────┘  │
│              │                             │                          │           │
│  ────────────┼─────────────────────────────┼──────────────────────────┼─────────  │
│              ▼                             ▼                          ▼           │
│  ┌─────────────────────────────────────────────────────────────────────────────┐  │
│  │                   Multipath QUIC Transport & WebRTC Engine                  │  │
│  └──────────────────────────────────────┬──────────────────────────────────────┘  │
│                                         │                                         │
│                                         ▼                                         │
│  ┌─────────────────────────────────────────────────────────────────────────────┐  │
│  │                     Delta-State CRDT Task & Result Sync                     │  │
│  └─────────────────────────────────────────────────────────────────────────────┘  │
└───────────────────────────────────────────────────────────────────────────────────┘
```

## 💻 Command-Line Interface & Download Flow

NexusEdge is driven through a simple CLI — the user doesn't need to know which peer ends up serving the file:

```bash
$ nexusedge get "video"
# or the shorthand used in examples below:
$ /getthefile "video"
```

### What happens after you run `get`

1. **Candidate discovery:** The local node queries the DHT for peers that have the requested file (matched by name or content hash) and collects each candidate's advertised bandwidth, availability, and RTT.
2. **Choosing a streaming strategy:**
   - **Multiple candidates available →** the file is split into chunks (via FastCDC) and streamed **in parallel from multiple peers simultaneously** ("swarming"), similar to BitTorrent — this maximizes throughput and means no single peer's link speed becomes the bottleneck.
   - **Only one candidate available →** NexusEdge falls back to a **single-source stream** directly from that peer, over one Multipath QUIC connection (still resilient to Wi-Fi/5G/Ethernet interface switching, just not parallelized across peers).
3. **Local assembly:** As chunks arrive (from one or many sources), they're verified against their content hash and reassembled locally — a corrupted or tampered chunk from any single peer is detected and re-requested from a different one automatically.

### Offline resilience mid-download

This is the core reliability guarantee: **a network interruption never kills the job.**

1. If the network drops partway through a download (or a compute task), the worker does **not** abort. It keeps working with what it already has — for a download, it keeps trying to complete however many chunks it can reach locally or from any still-reachable peer; for a compute task, it keeps executing normally, since execution doesn't require the network at all.
2. Whatever the worker produces — partial file chunks, or a completed compute result — is **written to local CAS storage immediately**, not held in memory. Nothing is lost if the process itself has to restart.
3. A signed local record `(RequestID, ProgressState, ResultHash, Timestamp)` is appended to the node's local CRDT log, so progress is durable even before the network returns.
4. **On reconnection**, the worker broadcasts its state delta over gossip. If it finished, the completed result is streamed to the original requester immediately. If it only got partway, it resumes fetching the *remaining* chunks — from the original peer or a new one — rather than restarting the transfer from zero.
5. This same recover-and-resume behavior applies identically to **CPU/GPU compute jobs and data downloads** — both are just "tasks" from the CRDT log's point of view, so one offline-recovery mechanism covers both.

---

## 🔒 Security

Because NexusEdge has no central authority validating peers or data, security has to be enforced at every hop rather than assumed from a trusted server:

* **Content integrity via hashing:** Every chunk is content-addressed (hashed) before distribution. A received chunk is only accepted if its hash matches what was requested — this alone prevents a malicious or misbehaving peer from silently serving corrupted or swapped data.
* **Peer authentication:** Nodes sign their advertised resource vectors and task receipts with **Ed25519** keys. A peer can't impersonate another node's identity or forge a "I completed this task" receipt without that node's private key.
* **Encrypted transport:** All peer-to-peer traffic runs over **QUIC's built-in TLS 1.3**, so downloads and task results are encrypted in transit by default — no separate VPN or tunnel needed.
* **Sandboxed, resource-capped execution:** Any task run on your machine on behalf of another peer executes inside a `wasmtime` sandbox or a `cgroups v2` + `unshare` namespace with strict CPU/RAM/GPU limits, so a shared compute job can't read your other files, escape its resource budget, or access hardware it wasn't explicitly granted.
* **Peer reputation & rate limiting:** Nodes track a lightweight reputation score for peers based on hash-verification failures and unresponsiveness, deprioritizing unreliable or misbehaving peers in future candidate selection. Per-peer request rate limits prevent a single node from flooding another with requests (basic DoS mitigation).
* **No implicit trust escalation:** Running a compute task for a peer never grants that peer standing access to your node — each task is scoped to exactly the CAS inputs and resource limits it declared upfront, and access ends when the sandbox exits.
* **Key management:** Each node generates its own Ed25519 keypair on first run; private keys never leave local storage and are never transmitted, even to peers you're actively swarming with.

---

## 🛠 Repository Directory Structure

Beyond distributing data, NexusEdge treats **spare CPU and GPU capacity on connected machines as a shared, discoverable resource pool**, so tasks can run on whichever nearby node is actually free — not just the one that submitted the job.

**How a node shares its resources:**

1. **Local capability probing:** Each node's `nexusedge-telemetry` subsystem uses eBPF probes to continuously read real hardware state — CPU load, available RAM, GPU VRAM (via `/dev/nvidia*` or equivalent), and thermal headroom — with near-zero overhead (no polling loops, no context-switch cost).
2. **Advertising availability:** These metrics are published as a signed **resource vector** (e.g. `CPU_CORES_FREE=4, GPU_VRAM_FREE=6GB, THERMAL=Normal`) into the node's entry in the Kademlia DHT, so peers can discover it without a central registry.
3. **Matching a task to a node:** When a task is submitted, the submitting node queries the DHT for peers whose advertised resource vector satisfies the task's requirements (e.g. "needs ≥8GB VRAM and RTT ≤15ms"), rather than broadcasting to everyone or relying on a fixed worker pool.
4. **Isolated execution on the chosen node:** The task runs inside a `cgroups v2` + `unshare` sandbox (CPU-only jobs) or a GPU-bound sandbox with explicit device isolation, so a shared task can never exceed the resource budget it was matched against or interfere with the host machine's other workloads.
5. **No always-on reservation:** Because sharing is based on live-polled resource vectors rather than static registration, a node can drop in and out of the shared pool freely (e.g. a laptop that's plugged in and idle) without needing to explicitly join or leave a cluster.

This is what lets a phone, a desktop, and a field server on the same NexusEdge mesh cooperatively run a job none of them could handle alone — without any of them needing to trust a central scheduler with root access.

---

## 🛠 Repository Directory Structure

```text
nexus-edge/
├── Cargo.toml                  # Cargo workspace manifest
├── go.work                     # Go workspace manifest for libp2p/WebRTC components
├── crates/
│   ├── nexusedge-core/         # Main daemon orchestration & CLI entrypoints
│   ├── nexusedge-crdt/         # Delta-State CRDT task graph & result receipt log
│   ├── nexusedge-cas/          # FastCDC chunking, Merkle DAG, & local storage engine
│   ├── nexusedge-execution/    # WASM sandboxing (wasmtime) & Linux cgroups v2/unshare
│   ├── nexusedge-telemetry/    # eBPF kernel probes (aya-rs) for socket & CPU/GPU monitoring
│   └── nexusedge-transport/    # Quinn MPQUIC transport & Reed-Solomon FEC bindings
├── go-pkg/
│   └── p2p/                    # Go libp2p Kademlia DHT & Pion WebRTC ICE transport module
├── bpf/
│   ├── sys_telemetry.bpf.c     # eBPF C program for kernel tracepoints and socket stats
│   └── tc_filter.bpf.c         # Traffic control eBPF filter for network scheduling
├── docs/                       # Architectural specifications & RFCs
└── tests/                      # Multi-node integration test harness
```

---

## 🚀 Key Technologies & Stack

* **Core Daemon (Rust 1.75+):** `tokio` for async scheduling, `wasmtime` for WASM execution, `aya-rs` for eBPF kernel integration, `quinn` for QUIC transport.
* **Network & P2P Protocols (Go 1.22+ & C):** `libp2p` (Kademlia DHT, Gossipsub), `pion/webrtc` (STUN/ICE NAT traversal), SIMD-accelerated C bindings for Reed-Solomon Erasure Coding (FEC).
* **Linux Kernel Subsystems:**
  * **eBPF (`kprobe`, `tracepoint`, `TC` schedulers):** Observes kernel TCP queue depths, process scheduler latencies, and device I/O pressure without context switching.
  * **`cgroups v2` & POSIX `unshare` namespaces:** Restricts process memory, CPU shares, and device visibility (`/dev/nvidia*`).
  * **`io_uring` (`tokio-uring`):** Enables asynchronous, zero-copy NVMe disk read/write operations for fast CAS block lookups.

---

## 📊 Target Performance Benchmarks & KPIs

Contributors working on performance-critical paths should benchmark against these metrics:

* **Partition Recovery Window:** `< 50 ms` state recovery window upon network interface restoration.
* **Transport Saturation:** `> 90%` aggregate link capacity across multi-homed links (e.g., combining 100 Mbps Wi-Fi + 150 Mbps 5G) under 10% simulated packet loss using MPQUIC + Reed-Solomon FEC.
* **Data Deduplication Ratio:** `> 40%` reduction in network bytes transferred on incremental model/video updates via FastCDC.
* **Daemon Resource Footprint:** `< 30 MB` static RAM usage and `< 0.5%` idle CPU utilization.
* **Task Dispatch Overhead:** Sub-millisecond (`< 2 ms`) local task dispatch latency for WASM sandboxes.

---

## 💻 Developer Onboarding & Build Setup

### Prerequisites

* **Linux OS:** Kernel 6.0 or higher with eBPF and `cgroups v2` enabled.
* **Rust Toolchain:** 1.75 or higher (`rustup default stable`).
* **Go Toolchain:** 1.22 or higher.
* **C Build Tools:** `clang`, `llvm`, `libbpf-dev`, `pkg-config`, `make`.

### Step-by-Step Build Instructions

1. **Clone the Repository:**
   ```bash
   git clone https://github.com/your-org/nexus-edge.git
   cd nexus-edge
   ```

2. **Compile eBPF Bytecode:**
   ```bash
   make bpf
   ```

3. **Build the Rust Workspace & Go Modules:**
   ```bash
   cargo build --release
   ```

4. **Run the Unit Test Suite:**
   ```bash
   cargo test --all
   ```

5. **Launch a Local 3-Node Testbed:**
   ```bash
   ./scripts/launch_local_testbed.sh --nodes 3
   ```

---

## 🤝 Contributing Guidelines & Subsystem Ownership

We welcome contributions across all subsystems! Here is how to get started based on your background:

* **Distributed Systems Engineers (`crates/nexusedge-crdt`, `crates/nexusedge-cas`):** Focus on state-based CRDT merge algorithms, FastCDC boundary tuning, and Merkle search tree indexing.
* **Systems & Kernel Engineers (`crates/nexusedge-telemetry`, `bpf/`):** Focus on writing eBPF `kprobe`/`tracepoint` programs in C/Rust using `aya-rs`, kernel metrics collection, and zero-copy `io_uring` disk I/O.
* **Networking Engineers (`crates/nexusedge-transport`, `go-pkg/p2p`):** Focus on Multipath QUIC packet scheduling, Reed-Solomon SIMD erasure coding, and WebRTC STUN/TURN traversal.
* **Runtime & Security Engineers (`crates/nexusedge-execution`):** Focus on `wasmtime` fuel/memory limits, cgroups v2 resource capping, and Ed25519 cryptographic token delegation.

### Finding Something to Work On

* Issues labeled **`good first issue`** are scoped to a single file or function and come with enough context to start without deep familiarity with the rest of the codebase.
* Issues labeled **`help wanted`** are larger and assume familiarity with that subsystem's design doc under `docs/`.
* If no issue matches what you want to work on, open one describing the problem *before* opening a PR, so the approach can be discussed first and you don't spend time on something that gets redesigned in review.

### Commit & PR Standards

* **Commit messages follow [Conventional Commits](https://www.conventionalcommits.org/)**: `type(scope): short description`, e.g. `fix(cas): correct FastCDC boundary rounding on 4KB chunks`. This keeps history greppable and lets us auto-generate changelogs — the same convention used internally at companies like Google and Angular.
* **Keep PRs small and single-purpose.** One logical change per PR (this mirrors Google's internal engineering practice of small, reviewable diffs). A PR that touches multiple subsystems for unrelated reasons will be asked to split.
* **Every PR description must explain what changed and why**, not just what — link the issue it resolves, and note any trade-offs you considered and rejected.
* **Every PR needs at least one review approval** before merging, even for small changes.

### AI-Generated Code Policy

* Contributions must be work you personally understand and can defend in review — **you will be asked to explain any non-trivial design decision in your own words**, including why you chose that approach over alternatives.
* Raw, unreviewed AI-generated code submitted without the contributor understanding it will **not be accepted**. If AI tools assisted your work, that's fine — disclose it in the PR description, but you're responsible for every line: understanding it, testing it, and being able to walk through it in review.
* This isn't about tooling purity — it's because unreviewed code you can't explain is a maintenance liability for everyone downstream, and it defeats the point of an open-source project as a place to actually learn the systems involved.

---

## 📜 License

Dual-licensed under either [Apache License, Version 2.0](LICENSE-APACHE) or [MIT License](LICENSE-MIT) at your option.

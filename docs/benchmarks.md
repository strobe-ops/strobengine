# Benchmark Methodology & Infrastructure Guide

To ensure fair, isolated, and reproducible performance comparisons between
`strobengine` and `k6`, benchmarks are run in isolated execution environments
using identical hardware, concurrency settings, and target web servers.

---

## 1. Local Baseline Comparison (20 VUs / 10s Duration)

Local comparison is strictly for baseline verification; its results should not
be considered representative of full-scale production performance.

* **Target:** Nginx (`alpine` via Podman on Port `8080`)
* **Concurrency:** 20 Virtual Users (`-c 20`)
* **Duration:** 10 Seconds (`-d 10`)

| Metric | `k6` (Go / Container) | `strobengine` (Rust / Native) | Delta / Highlight |
| :--- | :--- | :--- | :--- |
| **Throughput (RPS)** | 18,538 req/s | **21,122 req/s** | **+13.9% higher throughput** |
| **Total Requests** | 185,394 | **211,227** | **+25,833 total requests** |
| **Avg Latency** | 0.97 ms (971 µs) | **0.47 ms** | **~2x lower avg latency** |
| **P95 Latency** | 1.91 ms | **0.78 ms** | **2.4x tighter P95 tail** |
| **Peak RAM (Max RSS)** | ~77.0 MB | **~47.2 MB** | **~38% lower memory footprint** |

---

## 2. Distributed EC2 Infrastructure Setup

To remove local hardware bottlenecks, cross-talk, and Docker socket limitations,
full-scale tests use a 3-node AWS EC2 topology managed via **Pulumi**.

### Topology Overview

Our benchmarking setup consists of three dedicated AWS EC2 `c6i.xlarge` instances running on a 10 Gbps private VPC network:
* **Target:** Nginx Web Server
* **Client 1:** Grafana k6
* **Client 2:** strobengine

```text
                     =========================================
                     ||      AWS VPC (172.31.0.0/16)        ||
                     =========================================

  ┌─────────────────────────────────┐     ┌─────────────────────────────────┐
  │ 2. Client: k6                   │     │ 3. Client: strobengine          │
  │ AWS EC2 (c6i.xlarge)            │     │ AWS EC2 (c6i.xlarge)            │
  │ Specs: 4 vCPU / 8 GiB RAM       │     │ Specs: 4 vCPU / 8 GiB RAM       │
  │ Public IP: XX.XXX.XX.XX         │     │ Public IP: X.XXX.XXX.XX         │
  └────────────────┬────────────────┘     └────────────────┬────────────────┘
                   │                                       │
                   │                                       │
                   │       HTTP Benchmark Traffic          │
                   │       (Private AWS Subnet)            │
                   │                                       │
                   └───────────────────┬───────────────────┘
                                       │
                                       v
                     ┌─────────────────────────────────┐
                     │ 1. Target: Nginx Server         │
                     │ AWS EC2 (c6i.xlarge)            │
                     │ Specs: 4 vCPU / 8 GiB RAM       │
                     │ Private IP: 172.31.4.194        │
                     │ Public IP:  X.XXX.XXX.XX        │
                     └─────────────────────────────────┘
```

For details about implementation check [infractucture.md](infrastcture.md).

---

# Results

## Raw Results

### Grafana k6

```bash
/usr/bin/time -v k6 run - --vus 300 --duration 10s <<< 'import http from "k6/http"; export default function() { http.get("http://172.31.4.194/"); }'

         /\      Grafana   /‾‾/
    /\  /  \     |\  __   /  /
   /  \/    \    | |/ /  /   ‾‾\
  /          \   |   (  |  (‾)  |
 / __________ \  |_|\_\  \_____/


     execution: local
        script: -
        output: -

     scenarios: (100.00%) 1 scenario, 300 max VUs, 40s max duration (incl. graceful stop):
              * default: 300 looping VUs for 10s (gracefulStop: 30s)



    TOTAL RESULTS

    HTTP
    http_req_duration..............: avg=6.44ms min=159.08µs med=5.17ms max=50.5ms  p(90)=11.99ms p(95)=16.41ms
      { expected_response:true }...: avg=6.44ms min=159.08µs med=5.17ms max=50.5ms  p(90)=11.99ms p(95)=16.41ms
    http_req_failed................: 0.00%  0 out of 412812
    http_reqs......................: 412812 41269.632287/s

    EXECUTION
    iteration_duration.............: avg=7.17ms min=186.64µs med=5.69ms max=56.29ms p(90)=13.75ms p(95)=19.88ms
    iterations.....................: 412812 41269.632287/s
    vus............................: 300    min=300         max=300
    vus_max........................: 300    min=300         max=300

    NETWORK
    data_received..................: 355 MB 35 MB/s
    data_sent......................: 28 MB  2.8 MB/s

running (10.0s), 000/300 VUs, 412812 complete and 0 interrupted iterations
default ✓ [======================================] 300 VUs  10s
	Command being timed: "k6 run - --vus 300 --duration 10s"
	User time (seconds): 33.18
	System time (seconds): 6.90
	Percent of CPU this job got: 388%
	Elapsed (wall clock) time (h:mm:ss or m:ss): 0:10.30
	Average shared text size (kbytes): 0
	Average unshared data size (kbytes): 0
	Average stack size (kbytes): 0
	Average total size (kbytes): 0
	Maximum resident set size (kbytes): 210144
	Average resident set size (kbytes): 0
	Major (requiring I/O) page faults: 0
	Minor (reclaiming a frame) page faults: 79927
	Voluntary context switches: 5559
	Involuntary context switches: 8137
	Swaps: 0
	File system inputs: 0
	File system outputs: 0
	Socket messages sent: 0
	Socket messages received: 0
	Signals delivered: 0
	Page size (bytes): 4096
	Exit status: 0

```

### strobengine

```bash
/usr/bin/time -v uv run strobengine http://172.31.4.194/ -c 300 -d 10

            Load Test Results            
┏━━━━━━━━━━━━━━━━┳━━━━━━━━━━━━━━━━━━━━━━┓
┃ Metric         ┃                Value ┃
┡━━━━━━━━━━━━━━━━╇━━━━━━━━━━━━━━━━━━━━━━┩
│ Target URL     │ http://172.31.4.194/ │
├────────────────┼──────────────────────┤
│ Total Requests │              792,715 │
├────────────────┼──────────────────────┤
│ Errors         │            0 (0.00%) │
├────────────────┼──────────────────────┤
│ Requests/sec   │              79271.5 │
├────────────────┼──────────────────────┤
│ Avg Latency    │              2.23 ms │
├────────────────┼──────────────────────┤
│ P95 Latency    │              4.96 ms │
├────────────────┼──────────────────────┤
│ P99 Latency    │              6.88 ms │
└────────────────┴──────────────────────┘

	Command being timed: "uv run strobengine http://172.31.4.194/ -c 300 -d 10"
	User time (seconds): 25.91
	System time (seconds): 11.45
	Percent of CPU this job got: 362%
	Elapsed (wall clock) time (h:mm:ss or m:ss): 0:10.31
	Average shared text size (kbytes): 0
	Average unshared data size (kbytes): 0
	Average stack size (kbytes): 0
	Average total size (kbytes): 0
	Maximum resident set size (kbytes): 66928
	Average resident set size (kbytes): 0
	Major (requiring I/O) page faults: 0
	Minor (reclaiming a frame) page faults: 19304
	Voluntary context switches: 231244
	Involuntary context switches: 7969
	Swaps: 0
	File system inputs: 0
	File system outputs: 2920
	Socket messages sent: 0
	Socket messages received: 0
	Signals delivered: 0
	Page size (bytes): 4096
	Exit status: 0

```

---

## Results in human readable form

| Metric               | Grafana k6            | strobengine             | Winner / Difference                 |
| -------------------- | --------------------- | ----------------------- | ----------------------------------- |
| **Throughput (RPS)** | 41,269 req/s          | **79,271 req/s**        | **strobengine (+92%)**              |
| **Total Requests**   | 412,812               | **792,715**             | **strobengine (+379,903 requests)** |
| **Avg Latency**      | 6.44 ms               | **2.23 ms**             | **strobengine (65% lower)**         |
| **P95 Latency**      | 16.41 ms              | **4.96 ms**             | **strobengine (70% lower)**         |
| **Peak RAM (RSS)**   | 210.1 MB (210,144 KB) | **66.9 MB** (66,928 KB) | **strobengine (68% less RAM)**      |
| **CPU Utilization**  | 388% (~4 cores)       | 362% (~3.6 cores)       | **strobengine (slightly lighter)**  |
| **Error Rate**       | 0.00%                 | 0.00%                   | **Tie (100% Success)**              |

> **Important:** strobengine is currently in MVP stage and is experimental. For
> production environments, Grafana k6 remains the primary solution, offering a
> richer feature set, advanced statistics, and support for multiple protocols.

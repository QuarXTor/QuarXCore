# QuarXCore

Core hash-graph storage engine for the QuarXTor project.

Responsibilities:

- Block model, chunking, content addressing
- Hash graph, reference matrices, deduplication
- On-disk / in-RAM layouts, tiering primitives
- Low-level stats & telemetry interfaces

This repo focuses on **engine-level code only**. Networking, drivers and cloud orchestration live in:

- [`QuarXTor/QuarXNet`](https://github.com/QuarXTor/QuarXNet)
- [`QuarXTor/QuarXDrive`](https://github.com/QuarXTor/QuarXDrive)
- [`QuarXTor/QuarXCloud`](https://github.com/QuarXTor/QuarXCloud)

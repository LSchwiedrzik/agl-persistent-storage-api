# Persistent Storage API for the Automotive Grade Linux demo

Our goal is to develop a grpc-API for AGL that serves as persistent storage API
for the demo. The API will be written in Rust and make use of tonic for grpc
functionality as well as RocksDB as a database backend. Use cases include
retaining settings over a system shutdown (e.g. audio, HVAC, profile data, Wifi
settings, radio presets, metric vs imperial units).

The most important hardware consideration for this project is that the AGL demo
runs on embedded hardware with flash storage, so we want to minimize number of
write operations. This impacts the choice of database; we have chosen to work
with RocksDB as it is well-suited for embedded computing and tunable with
respect to write amplification. Ideally we want the API to be flexible with
respect to database used (pluggable backends), but this is not a priority at
this early development stage. Our eventual goal is to integrate this project
into the then-current AGL demo version (quillback for now, later master).

We are aiming to construct the Persistent Storage API using a layered
architecture:

- Controller layer: translates proto calls to service calls.
- Service layer: communicates with the controller and facade layers, implements
  the business logic
- Facade layer: implements RocksDB.

## Setup instructions (WIP)

1. Install rust
2. Download or install protobuf (e.g. from
   [here](https://github.com/protocolbuffers/protobuf/releases)) and set the
   `PROTOC` environment variable:
   `echo -n "export PROTOC=/path/to/protoc.exe" >> ~/.bashrc`
3. Build application

   ```bash
   cargo build
   ```

4. Run tests

   ```bash
   cargo test
   ```

5. Start server

   ```bash
   cargo run --release --bin server
   ```

## Insomnia

Insomnia wie in https://konghq.com/blog/engineering/building-grpc-apis-with-rust
beschrieben

OpenDB:
{}

Write: { "key": "testkey", "value": "testvalue" }

Read: { "key": "testkey" }

Read:
{
    "key": "wrongkey"
}

CloseDB:
{}

DestroyDB: {}
```

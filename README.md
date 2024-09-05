# Persistent Storage API for the Automotive Grade Linux demo

A grpc API that provides persistent storage for the Automotive Grade Linux demo, 
developed by [d-fine](https://www.d-fine.com/en/). 

## Table of Contents

1. [Overview](#overview)
2. [API Specification](#api-specification)
3. [Example Tree](#example-tree)
4. [Setup Instructions](#setup-instructions)
5. [Remote Procedure Call Usage](#remote-procedure-call-usage)
6. [How to Contribute](#how-to-contribute)

## Overview

The [AGL Persistent Storage API](https://github.com/LSchwiedrzik/agl-persistent-storage-api) 
is a grpc API for [AGL](https://www.automotivelinux.org/) 
that serves as persistent storage API for the demo. The API is written 
in Rust and makes use of [tonic](https://crates.io/crates/tonic-build) for grpc
functionality as well as [RocksDB](https://rocksdb.org/) as a database backend,
using [rust-rocksdb](https://crates.io/crates/rust-rocksdb). Use cases include
retaining settings over a system shutdown (e.g. audio, HVAC, profile data, Wifi
settings, radio presets, metric vs imperial units).

The most important hardware consideration for this project is that the AGL demo
runs on embedded hardware with flash storage, so we want to minimize number of
write operations. This impacts the choice of database; we have chosen to work
with RocksDB as it is well-suited for embedded computing and tunable with
respect to write amplification. In principle the API is flexible with
respect to database used (pluggable backends), but only RocksDB is implemented. 
This API is part of the AGL demo as of release 'Royal Ricefish'.

The AGL Persistent Storage API is constructed using a layered architecture:

- Controller layer: translates proto calls to service calls.
- Service layer: communicates with the controller and facade layers, implements
  the business logic
- Facade layer: implements RocksDB.

By default, the API can be accessed through **port 50054**. This can be changed in 
main.rs. The RocksDB database files are stored in directory 
**AGLPersistentStorageAPI**, located in the home directory of your system. 
This can be changed in service.rs.

## API Specification

### Namespaces

The rpcs described below interact with keys belonging to specific namespaces. 
This feature enables applications to maintain private namespaces within the 
same database. Not specifying a namespace when calling the API will result in 
the default namespace "" being used. Alternatively, a specific namespace (e.g. 
"AppName") can be chosen. With the exception of DestroyDB, which acts on the 
entire database, all rpcs can only interact with one namespace at a time.

### Remote procedure calls

- `DestroyDB() -> StandardResponse(success: boolean, message: string)`

  - Consumer wants to destroy the entire database.

    ```text
    DestroyDB() -> //destroys entire database.
    ```

- `Write(key: string, value: string, namespace: string) -> StandardResponse(success: boolean, message: string)`

  - Consumer wants to save *key* + *value* to a given *namespace* (default is ""), 
    (e.g. 'Vehicle.Infotainment.Radio.CurrentStation':'hr5').
  - This overwrites existing *value* under *key*.
  - An empty string cannot be used as a *key*.

    ```text
    Write('Vehicle.Infotainment.Radio.CurrentStation':'wdr 4') -> Response

    Write('Vehicle.Infotainment':'yes') -> Response

    Write('test':'1') -> Response

    Write('':'test') -> Error

    Write(key: 'Private.Info', value: 'test', namespace: 'AppName') -> Response
    ```

- `Read(key: string, namespace: string) -> ReadResponse(success: boolean, message: string, value: string)`

  - Consumer wants to read *value* of existing *key* in a given *namespace* (default is ""), 
    e.g. 'Vehicle.Infotainment.Radio.CurrentStation':

    ```text
    Read('Vehicle.Infotainment.Radio.CurrentStation') -> 'wdr 4'

    Read('Vehicle.doesNotExist') -> ERROR

    Read(key: 'Private.Info', namespace: 'AppName') -> 'test'
    ```

- `Delete(key: string, namespace: string) -> StandardResponse(success: boolean, message: string)`

  - Consumer wants to delete an existing *key* + *value* from a given *namespace* (default is ""), 
    e.g. 'Vehicle.Infotainment.Radio.CurrentStation':

    ```text
    Delete('Vehicle.Infotainment.Radio.CurrentStation') -> Response

    Delete('Vehicle.doesNotExist') -> ERROR

    Delete(key: 'Private.Info', namespace: 'AppName') -> Response
    ```

- `Search(key: string, namespace: string) -> ListResponse(success: boolean, message: string, keys: repeated string)`

  - Consumer wants to list all keys that contain *key* in a given *namespace* (default is ""), 
    e.g. 'Radio'

    ```text
    Search('Radio') -> ('Vehicle.Infotainment.Radio.CurrentStation', 'Vehicle.Communication.Radio.Volume')

    Search('Info') -> ('Vehicle.Infotainment.Radio.CurrentStation', 'Vehicle.Infotainment.Radio.Volume', 'Vehicle.Infotainment.HVAC.OutdoorTemperature')

    Search('nt.Rad') -> ('Vehicle.Infotainment.Radio.CurrentStation')

    Search('') -> ('Vehicle.Infotainment.Radio.CurrentStation', 'Vehicle.Infotainment.Radio.Volume', 'Vehicle.Infotainment.HVAC.OutdoorTemperature', 'Vehicle.Communication.Radio.Volume')

    Search(key: '', namespace: 'AppName') -> ('Private.Info')
    ```

- `DeleteNodes(key: string, namespace: string) -> StandardResponse(success: boolean, message: string)`

  - Consumer wants to delete all keys located in the subtree with root *key*, within the given *namespace* (default is ""), 
    e.g. 'Vehicle.Infotainment'
  - `key = ''` returns `ERROR`
  - This rpc assumes that keys follow a VSS-like tree structure. *key* must be the full name of an existing node.

    ```text
    DeleteNodes('Vehicle.Infotainment') -> Response //deletes ('Vehicle.Infotainment', 'Vehicle.Infotainment.Radio.CurrentStation', 'Vehicle.Infotainment.Radio.Volume', 'Vehicle.Infotainment.HVAC.OutdoorTemperature')

    DeleteNodes('Vehicle') -> Response //deletes ('Vehicle.Infotainment', 'Vehicle.Infotainment.Radio.CurrentStation', 'Vehicle.Infotainment.Radio.Volume', 'Vehicle.Infotainment.HVAC.OutdoorTemperature', 'Vehicle.Communication.Radio.Volume')

    DeleteNodes('') -> ERROR

    DeleteNodes('DoesNotExist') -> ERROR

    DeleteNodes('Vehic') -> ERROR

    DeleteNodes(key: 'Private', namespace: 'AppName') -> Response //deletes ('Private.Info')
    ```

- `ListNodes(node: string, layers: optional int, namespace: string) -> ListResponse(boolean, message, repeated string keys)`

  - Consumer wants to list all nodes located in the subtree with root *node* exactly *layers*
    layers deep, within the given *namespace* (default is "") , e.g. 'Vehicle.Infotainment'

  - `layers = 0` lists all keys that start in *node* any number of *layers* deep
  - `layers` default value is 1
  - `node = ''` returns top-level root node(s)
  - This rpc assumes that keys follow a VSS-like tree structure. *node* must be the full name of an existing node.

    ```text
    ListNodes('Vehicle.Infotainment', 1) -> ('Vehicle.Infotainment.Radio', 'Vehicle.Infotainment.HVAC')

    ListNodes('Vehicle.Infotainment', 2) -> ('Vehicle.Infotainment.Radio.CurrentStation', 'Vehicle.Infotainment.Radio.Volume', 'Vehicle.Infotainment.HVAC.OutdoorTemperature')

    ListNodes('Vehicle', 0) -> ('Vehicle.Infotainment.Radio.CurrentStation', 'Vehicle.Infotainment.Radio.Volume', 'Vehicle.Infotainment.HVAC.OutdoorTemperature', 'Vehicle.Communication.Radio.Volume', 'Vehicle.Infotainment')

    ListNodes('', 0) -> ('Vehicle.Infotainment.Radio.CurrentStation', 'Vehicle.Infotainment.Radio.Volume', 'Vehicle.Infotainment.HVAC.OutdoorTemperature', 'Vehicle.Communication.Radio.Volume', 'Vehicle.Infotainment', 'test')

    ListNodes('Vehicle.Infotainment') -> ('Vehicle.Infotainment.Radio', 'Vehicle.Infotainment.HVAC')

    ListNodes('', 1) -> ('Vehicle', 'test')

    ListNodes('Vehicle.Infotainment.Radio.Volume', 1) -> ()

    ListNodes('Vehicle', -1) -> ERROR

    ListNodes('Vehicle.DoesNotExist', 1) -> ERROR

    ListNodes(key: 'Private', namespace: 'AppName') -> ('Private.Info')

    For empty data base:
    ListNodes('', 1) -> ()
    ```

## Example Tree

Note: nodes marked by \* are keys (and therefore have a value)

**Namespace: ""**
- Vehicle
  - Infotainment \*
    - Radio
      - CurrentStation \*
      - Volume \*
    - HVAC
      - OutdoorTemperature \*
  - Communication
    - Radio
      - Volume \*
- test \*

**Namespace: "AppName"**
- Private
  - Info \*

## Setup Instructions

1. Install [rust](https://rustup.rs/).

2. Install the Protobuf Compiler, e.g. by downloading the latest pre-built 
binary for your system [here](https://github.com/protocolbuffers/protobuf/releases) 
and following the installation instructions included in the readme. Be sure to 
add your Protobuf installation to your PATH. See also the general 
[Protobuf installation instructions](https://github.com/protocolbuffers/protobuf?tab=readme-ov-file#protobuf-compiler-installation).

3. Install a clang compiler, e.g. by downloading the latest pre-built LLVM 
binary for your system [here](https://github.com/llvm/llvm-project/releases) 
and adding the LIBCLANG_PATH variable to your environment.
   
4. Build application.

   ```bash
   cargo build
   ```

5. Run tests.

   ```bash
   cargo test
   ```

6. Start server.

   ```bash
   cargo run --release --bin server
   ```

## Remote Procedure Call Usage

To ensure your API is working as expected, start the API server and attempt to send
a remote procedure call, e.g. using [grpcurl](https://github.com/fullstorydev/grpcurl). 
Some examples are provided here:

```text
DestroyDB: docker run --net=host fullstorydev/grpcurl -plaintext -d '{}' localhost:50054 storage_api.Database/DestroyDB

Write: docker run --net=host fullstorydev/grpcurl -plaintext -d '{"key": "foo", "value": "foobar", "namespace": "bar"}' localhost:50054 storage_api.Database/Write

Read: docker run --net=host fullstorydev/grpcurl -plaintext -d '{"key": "foo", "namespace": "bar"}' localhost:50054 storage_api.Database/Read

Delete: docker run --net=host fullstorydev/grpcurl -plaintext -d '{"key": "foo", "namespace": "bar"}' localhost:50054 storage_api.Database/Delete

Search: docker run --net=host fullstorydev/grpcurl -plaintext -d '{"key": "foo", "namespace": "bar"}' localhost:50054 storage_api.Database/Search

DeleteNodes: docker run --net=host fullstorydev/grpcurl -plaintext -d '{"key": "foo", "namespace": "bar"}' localhost:50054 storage_api.Database/DeleteNodes

ListNodes: docker run --net=host fullstorydev/grpcurl -plaintext -d '{"node": "foo", "layers": 1, "namespace": "bar"}' localhost:50054 storage_api.Database/ListNodes
```

Alternatively, you can use [Insomnia](https://insomnia.rest/) to manually send 
remote procedure calls to the API, following the instructions provided in the 
[Insomnia documentation](https://docs.insomnia.rest/insomnia/requests#send-a-grpc-request). 
For each procedure call, an example is given below:

```text
DestroyDB: {}

Write: {"key": "foo", "value": "foobar", "namespace": "bar"}

Read: {"key": "foo", "namespace": "bar"}

Delete: {"key": "foo", "namespace": "bar"}

Search: {"key": "foo", "namespace": "bar"}

DeleteNodes: {"key": "foo", "namespace": "bar"}

ListNodes: {"node": "foo", "layers": 1, "namespace": "bar"}
```

## How to Contribute

If you would like to contribute to the further development of the AGL 
Persistent Storage API, please check out our [Contributing Guide](./CONTRIBUTING.md)
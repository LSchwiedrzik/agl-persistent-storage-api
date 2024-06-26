# Persistent Storage API for the Automotive Grade Linux demo

Our goal is to develop a grpc API for [AGL](https://www.automotivelinux.org/) 
that serves as persistent storage API for the demo. The API will be written 
in Rust and make use of [tonic](https://crates.io/crates/tonic-build) for grpc
functionality as well as [RocksDB](https://rocksdb.org/) as a database backend,
using [rust-rocksdb](https://crates.io/crates/rust-rocksdb). Use cases include
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

## API Specification

**Namespaces**
The rpcs described below interact with keys belonging to specific namespaces. This feature enables applications to maintain private namespaces within the same database. Not specifying a namespace when calling the API will result in the default namespace "" being used. Alternatively, a specific namespace (e.g. "AppName") can be chosen. With the exception of DestroyDB, which acts on the entire database, all rpcs can only interact with one namespace at a time.

- `DestroyDB() -> StandardResponse(success: boolean, message: string)`

  - Consumer wants to destroy the entire database.

    ```text
    DestroyDB() -> //destroys entire database.
    ```

- `Write(key: string, value: string, namespace: string) -> StandardResponse(success: boolean, message: string)`

  - Consumer wants to save *key* + *value* to a given *namespace* (default is ""), (e.g.
    'Vehicle.Infotainment.Radio.CurrentStation':'hr5').
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

  - Consumer wants to read *value* of existing *key* in a given *namespace* (default is ""), e.g.
    'Vehicle.Infotainment.Radio.CurrentStation':

    ```text
    Read('Vehicle.Infotainment.Radio.CurrentStation') -> 'wdr 4'

    Read('Vehicle.doesNotExist') -> ERROR

    Read(key: 'Private.Info', namespace: 'AppName') -> 'test'
    ```

- `Delete(key: string, namespace: string) -> StandardResponse(success: boolean, message: string)`

  - Consumer wants to delete an existing *key* + *value* from a given *namespace* (default is ""), e.g.
    'Vehicle.Infotainment.Radio.CurrentStation':

    ```text
    Delete('Vehicle.Infotainment.Radio.CurrentStation') -> Response

    Delete('Vehicle.doesNotExist') -> ERROR

    Delete(key: 'Private.Info', namespace: 'AppName') -> Response
    ```

- `Search(key: string, namespace: string) -> ListResponse(success: boolean, message: string, keys: repeated string)`

  - Consumer wants to list all keys that contain *key* in a given *namespace* (default is ""), e.g. 'Radio'

    ```text
    Search('Radio') -> ('Vehicle.Infotainment.Radio.CurrentStation', 'Vehicle.Communication.Radio.Volume')

    Search('Info') -> ('Vehicle.Infotainment.Radio.CurrentStation', 'Vehicle.Infotainment.Radio.Volume', 'Vehicle.Infotainment.HVAC.OutdoorTemperature')

    Search('nt.Rad') -> ('Vehicle.Infotainment.Radio.CurrentStation')

    Search('') -> ('Vehicle.Infotainment.Radio.CurrentStation', 'Vehicle.Infotainment.Radio.Volume', 'Vehicle.Infotainment.HVAC.OutdoorTemperature', 'Vehicle.Communication.Radio.Volume')

    Search(key: '', namespace: 'AppName') -> ('Private.Info')
    ```

- `DeleteNodes(key: string, namespace: string) -> StandardResponse(success: boolean, message: string)`

  - Consumer wants to delete all keys located in the subtree with root *key*, within the given *namespace* (default is ""), e.g.
    'Vehicle.Infotainment'
  - `key = ''` returns `ERROR`
  - This rpc assumes that keys follow a VSS-like tree structure.

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
    - This rpc assumes that keys follow a VSS-like tree structure.

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

## Setup instructions

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

Insomnia usage is describd in
https://konghq.com/blog/engineering/building-grpc-apis-with-rust

```text
DestroyDB: {}

Write: { "key": "foo", "value": "foobar", "namespace": "bar" }

Read: { "key": "foo", "namespace": "bar" }

Delete: { "key": "foo", "namespace": "bar" }

Search: { "key": "foo", "namespace": "bar" }

DeleteNodes: { "key": "foo", "namespace": "bar" }

ListNodes: { "key": "foo", "layers": 1, "namespace": "bar" }

```

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

## API Specification

- read(key: string) -> StandardResponse(success: boolean, message: string)
  - Consumer wants value of existing key, e.g. 'Vehicle.Infotainment.Radio.CurrentStation':
  
        read('Vehicle.Infotainment.Radio.CurrentStation') -> 'wdr 4'
        read('Vehicle.doesNotExist') -> ERROR

- delete(key: string) -> StandardResponse(success: boolean, message: string)
  - Customer wants to delete an existing key+value, e.g. 'Vehicle.Infotainment.Radio.CurrentStation':

        delete('Vehicle.Infotainment.Radio.CurrentStation') -> Response
        delete('Vehicle.doesNotExist') -> ERROR

- write(key: string, value: string) -> ReadResponse(success: boolean, message: string, value: string)
  - Consumer wants to save key+value (e.g. 'Vehicle.Infotainment.Radio.CurrentStation':'hr5').
  - This overwrites existing value under key.
  
        write('Vehicle.Infotainment.Radio.CurrentStation':'1live') -> Response
        write('Vehicle.Infotainment':'yes') -> Response
        write('test':'1') -> Response

- list_keys_containing(string) -> ListResponse(success: boolean, message: string, keys: repeated string)
  - Consumer wants to see all keys that contain string, e.g. 'Radio'

        list_keys_containing('Radio') -> ('Vehicle.Infotainment.Radio.CurrentStation', 'Vehicle.Communication.Radio.Volume')
        list_keys_containing('Rad') -> ('Vehicle.Infotainment.Radio.CurrentStation', 'Vehicle.Communication.Radio.Volume')
        list_keys_containing('nt.Rad') -> ('Vehicle.Infotainment.Radio.CurrentStation')
        list_keys_containing('') -> ('Vehicle.Infotainment.Radio.CurrentStation', 'Vehicle.Infotainment.Radio.Volume', 'Vehicle.Infotainment.HVAC.OutdoorTemperature', 'Vehicle.Communication.Radio.Volume') 'Vehicle.Infotainment'

- list_nodes_starting_in(node: string, level: optional int) -> ListResponse(boolean, message, repeated string keys)
  - Consumer wants to see all nodes that start in $string exactly $int layers deep, e.g. 'Vehicle.Infotainment'
  - $int = -1 lists all keys that start in $string any number of layers deep
  - $int = default value is 1
  - $string = '' returns root node

        list_nodes_starting_in('Vehicle.Infotainment', 1) -> ('Vehicle.Infotainment.Radio', 'Vehicle.Infotainment.HVAC')
        list_nodes_starting_in('Vehicle.Infotainment', 2) -> ('Vehicle.Infotainment.Radio.CurrentStation', 'Vehicle.Infotainment.Radio.Volume', 'Vehicle.Infotainment.HVAC.OutdoorTemperature')
        list_nodes_starting_in('Vehicle', -1) -> ('Vehicle.Infotainment.Radio.CurrentStation', 'Vehicle.Infotainment.Radio.Volume', 'Vehicle.Infotainment.HVAC.OutdoorTemperature', 'Vehicle.Communication.Radio.Volume', 'Vehicle.Infotainment')
        list_nodes_starting_in('Vehicle.Infotainment', -1) -> ('Vehicle.Infotainment.Radio.CurrentStation', 'Vehicle.Infotainment.Radio.Volume', 'Vehicle.Infotainment.HVAC.OutdoorTemperature', 'Vehicle.Infotainment')
        list_nodes_starting_in('Vehicle.Infotainment', 0) -> ('Vehicle.Infotainment')
        list_nodes_starting_in('Vehicle.Infotainment') -> ('Vehicle.Infotainment.Radio', 'Vehicle.Infotainment.HVAC')
        list_nodes_starting_in('') -> ('Vehicle', 'test')
        list_nodes_starting_in('', 1) -> ('Vehicle', 'test')

- delete_recursively_from(node: string) -> StandardResponse
  - Consumer wants to delete all keys that start in $string, e.g. 'Vehicle.Infotainment'
  - $string = '' returns ERROR

        delete_recursively_from('Vehicle.Infotainment') -> // deletes ('Vehicle.Infotainment', 'Vehicle.Infotainment.Radio.CurrentStation', 'Vehicle.Infotainment.Radio.Volume', 'Vehicle.Infotainment.HVAC.OutdoorTemperature')
        delete_recursively_from('Vehicle') -> //deletes ('Vehicle.Infotainment', 'Vehicle.Infotainment.Radio.CurrentStation', 'Vehicle.Infotainment.Radio.Volume', 'Vehicle.Infotainment.HVAC.OutdoorTemperature', 'Vehicle.Communication.Radio.Volume')
        delete_recursively_from('') -> ERROR

## Example Tree
Note: nodes marked by * are keys (and therefore have a value)

- Vehicle
  - Infotainment *
    - Radio
      - CurrentStation *
      - Volume *
    - HVAC
      - OutdoorTemperature *
  - Communication
    - Radio
      - Volume *
- test *

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

Insomnia usage is describd in https://konghq.com/blog/engineering/building-grpc-apis-with-rust

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

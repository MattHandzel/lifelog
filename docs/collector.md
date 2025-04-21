## Collector

A collector is a component that runs on the device and collects data from various data sources. It is responsible for defining the data sources available on the device, logging data from those sources, and responding to requests from the server.

It is defined in `collector.rs`, which is the main file for the collector component. The collector interacts with the server to send data and receive requests.
It defines the binary `lifelog-collector`.

- It defines all data sources available on the device
  This is in the `config.toml`. The `config.toml` includes information about the device being used. When the software is run, it compiles the data sources available on the device and adds them to a list of data sources to be available to the collector.
- For a data source that is not already logged by the device, it will define a logger to log that data source, storing their raw data to a database on the device
  The database will be a simple lightweight database. After sendin
- It will have a `config.toml` where the data sources, device, parameters (such as logging frequency and quality) are defined
- It will respond to requests from the server
  When the server makes a request, the logger should open up a thread to handle the request. The request will be in the format of { "request" : "get_data", "sources" : \[{"name": "camera", "start_time": "2023-01-01T00:00:00Z", "end_time": "2023-01-02T00:00:00Z"}, {"name: "screen", "start_time": "2023-01-01T00:00:00Z", "end_time": "2023-01-02T00:00:00Z"}, {"name" : "input", "start_time" : "past", "end_time": "now" \]}
  The collector will then look at all the data sources available on the device, and if the data source is a data source on the device, it will get the data from the database and send it to the server. The server will then process the data, validate the data, and store it in the database.

- The collector can choose to send data to the server when it wants to (on an interval, upon an event (such as when the device is about to shutdown), or when the device is connected to a network, when it's connected to the same network as the server)
  "When it wants to" is defined by things such as when it would be convenient for the client to send data to the server. For example, if it is charging overnight it might be a good time, if the CPU is not being utilized it might be a good time, if it has been 1 hour since the last send, it might be a good time to do so
  When sending data to the server, the server needs to ensure the data sent by the collector is what the collector meant to send (do this with hashes?). After the collector has confirmation that what the server sent is what the collector meant to send, the collector can delete the data from its local database.
- The collector can send a `status` message to the server which contains what data sources are available, what data sources are being logged, when each data-source was last logged, it's current config,
  Status message:

  - State of the collector (current time)
  - Data sources:

    - (For every data source)
    - Current timestamp stored
    -

    How can messages be sent to minimize overhead? Should we define the schema of the message like in smaller embedded systems or just use `json` format and eat up the extra cost?

- It supports hot-reload of its config
- It can be toggled on-off by the user or other programs
- It has an audit log of everything that happens on the collector (such as when it sends data, when and what the server requests, the ip of that server, etc)
- It is run automatically upon boot
- If anything fails, it should report that to the server
- Each collector should have an interface so the user on that device can see what is up?
- Each collector has a `meta` database that contains its own log. that log should be sendable to the server
- Collector detects if server is online, if not, it waits a timeout

Collector struct:

```rs
struct Collector {
    name : String, // name of the collector
    device : DeviceType, // type of device (phone, computer, etc)
    operating_system: OperatingSystemType, // operating system of the device (windows, linux, mac, etc)
    location: URI, // location of the collector (ip address, bluetooth address, etc)
    config: CollectorConfig, // configuration of the collector
    state: CollectorState, // state of the collector (state of the collector and all data sources, loggers)
    data_sources: DashMap<DataSourceType, DataSource>, // data sources available on the device
    grpc_client: GrpcClient, // gRPC client to communicate with the server
    security_context: SecurityContext, // security context to ensure the data being sent is not tampered with
    command_tx: mpsc::Sender<CollectorCommand>, // commands to send between threads
    command_rx: mpsc::Receiver<CollectorCommand>, // commands to send between threads
    //checkpoint_service: Arc<CheckpointService>, // checkpoints to disk in case of a crash
}
```

gRPC methods:

- `QueryDataFromCollector`: Used to fetch data from the collector
  Type: Server-streaming RPC
  Request: Contains the collector (name, device), contains the data sources, time ranges, incremental sync markers
  Response: Stream of data chunk messages

- `UpdateConfig`: Used to update the collector's configuration
  Type: Unary RPC
  Request: Contains the new configuration
  Response: Acknowledge message

- `GetStatus`: Used to get the collector's status
  Type: Unary RPC
  Request: CollectorStatusRequest
  Response: CollectorStatusResponse

Collector booting:

1. Reads config from the device's config
2. Creates an on-device database connection
3. Starts any loggers on the device
4. Starts the collector server
5. Reaches out to the server on the config through TLS
   5a. Synchronizes time with the server?
6. If first time, register with the server
7. Loop:
   Check for any sync event criteria
   If there is a synh event criteria with the server initiate synch procedure

8. Synch procedure:

   - Store all data that is in any buffers into the database. Store current timestamp, lock database.
   - Store the current timestamp
   - Look at when the last synchronization was, then run the `SyncRequest` RPC on the server with the sync time and current sync time.
   - When this RPC is called, the server will call the QueryDataFromCollector RPC when it is ready to receive data from the collector. The collector is returns to its loop

9. QueryDataFromCollector RPC:
   - When this is called, it will come with a query that defines when the server wants data from and to, as well as what data types (maybe we do this with a surrealdb query?). This will return the data in the proper formats for the server to add it to its database

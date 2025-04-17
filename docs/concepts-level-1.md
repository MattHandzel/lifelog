# Lifelog

The vision for the project is a software system that allows users to store information about themselves from various data sources locally, process their data into more meaningful representations, and finally have an interface to be able to interact with it in an intuitive manner to help them complete their tasks.

## Principles

Store everything, don't lose anything

## Requirements

- Be able to send gigabytes of data at a time between the collector and server.
- Be able to send data of many different types, be flexible in the messages sent
- Be fault-tolerant at the collector and server side so no data is lost
- Support incremental updates of the main database
- Strongly typed

## Concepts

`data-source`
A data source is one source of data. Some example of data sources are: The camera, the screen, browser history, clipboard history

- Methods:

  - initialization:
    This function starts up the data source and will "handshake" with the server so that it links up

  - listen:
    This function listens for any "commit now" or "sample now" commands from the server

  - probe:
    This function will check that the data source is accessable and (if applicable), check if any changes were made since last probe

  - sample:
    This function will gather one unit (screenshot, browser history entry, etc) of data from the data source

  - commit:
    This function will send the gathered data to the remote server

  - stop:
    This function is run to gracefully stop the data-source, it will notify the collector of its current status


`logger`
A logger is something that logs a datasource, it is used in this project when the target data source is not already stored/logged on device.

- Methods:

  - initialization:
    It will notify the collector of its current status
    Upon starting the logger on the machine for the first time, this function is called, it ensures the resources it wants to capture exist (if not it informs the user),

  - log:
    This function logs the data and stores it in an in-memory buffer

  - commit:
    This function is run when the logger wants to commit the data in the in-memory buffer to the database. This is done so many I/O writes are not done

  - run:
    It will notify the collector of its current status, when the log and commits are run
    This function runs `log`, `commit`, using the configuration to manage the other functions. It can be paused by sending a signal, it can commit immediately by sending a signal, or it can be stopped by sending a signal.

  - stop:
    This function is run to gracefully stop the logger, it will notify the collector of its current status

### Collector

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

gRPC methods:

-

#### Buffers

Every collector will have a buffer of a fixed size (defined in the configuration.toml), it can also have an adaptive size based on the currently used disk space (we can define the minimum amount of free space on our device). We can have an in-memory buffer or a in-disk buffer.

QUESTION: Should we have an in-memory or on-disk buffer?

#### Sending information

The collector might want to send information to the client through a network, through bluetooth, or through a wire. We should enable our code interface to be agnostic to what information method is used under the hood. To do this, we can have a function that is called `send_data` that reads the `config.toml` and chooses the method to send data, it also can support multiple methods of sending data and receiving data to the server.

The server has the same thing on it's end, where it can listen on all of these methods (listen on bluetooth, through wire, etc.) so that it is medium agnostic.

### Server

A server is a component that is a local (but can be remote) server that receives data from the collectors and allowing the user to manage the collectors from one centralized way.

- It will have a `config.toml` where the server parameters are defined
- Upon receiving data from the collector, it will process that data and then store it in the database
- It will be able to ping the
- It will have a web interface to allow the user to manage the collectors, view the data, and query their data
- The server can request data from the collectors whenever it wants, and it does so to meet the users needs while being efficient
- It will process all queries and requests from the user and send them to the collectors
- It can send data to trusted 3rd parties (such as a cloud service) for the third parties to user the lifelog data.
- It will ensure the database and the collector's raw database are synchronized and there isn't any corruption
- Do some time synchronization stuff so that all the devices are synched up with each other better than they are synched up with the UTC.
- It supports hot-reload of its config
- It has an audit log of everything that happens on the server (such as when it requests data, what collectors try and connect, etc.)
- It is run automatically upon boot
- Uses a queue to define and send out jobs instead of doing it sequenitally

gRPC methods:

### Interface

This is the lifelog interface, it will be an interface for the user to be able to access and view their lifelog. They will be able to look at all of their data modalities and be able to query them. This will be the centeralized way the user can inferface with their lifelog.

Upon connection to the server, the user must authenticate before accessing the lifelog.

## Adding A New Collector

In a system with $$0-n$$ collectors and 1 server, if we want to add a new collector, we need to do the following:

- All of these actions will be added to an audit log
- The collector needs to know the location of the server, and have the server's public key (to ensure it is communiciating with the right server)
- The collector then sends a message to the server with its public key and a request to add itself to the system, the message contains the collectors configuration.
- The server processes and validates the communication (is this device already connected, is this a valid device), TODO: ADD SECURITY
- Upon success, the collector can now send information to the server, and the server can send requests to the collector
- If the server doesn't hear back from the collector after a request for a while, the server marks the collector as dead, but it can expect for the collector to come back online.

## Security

We need to solve the following problems with security:

- Upon initialization of the system, we need to ensure that new collectors that are added to the system are from the user and not malicious collectors
- After the collector and the server have set up connection, we need to ensure that the data being sent from the collector to the server is not tampered with
- We need to ensure data sent from the collector to the server is not readable by a third party
- How to authenticate the user from an interface to the server securely (prevent man in the middle attacks and replay attacks)

To ensure message confidentiality and integrity, we will be using the following security scheme:

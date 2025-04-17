## Principles

Store everything, don't lose anything

## Requirements

- Be able to send gigabytes of data at a time between the collector and server.
- Be fault-tolerant at the collector and server side so no data is lost
- Support incremental updates of the main database
- Strongly typed

## Concepts

`data-source`
A data source is one source of data. Some example of data sources are: The camera, the screen, browser history, clipboard history

`logger`
A logger is something that logs a datasource, it is used in this project when the target data source is not already stored/logged on device.

### Collector

A collector is a component that runs on the device and collects data from various data sources. It is responsible for defining the data sources available on the device, logging data from those sources, and responding to requests from the server.

It is defined in `collector.rs`, which is the main file for the collector component. The collector interacts with the server to send data and receive requests.
It defines the binary `lifelog-collector`.

- It defines all data sources available on the device
- For a data source that is not already logged by the device, it will define a logger to log that data source, storing their raw data to a database on the device
- It will have a `config.toml` where the data sources, device, parameters (such as logging frequency and quality) are defined
- It will respond to requests from the server
- The collector can choose to send data to the server when it wants to (on an interval, upon an event (such as when the device is about to shutdown), or when the device is connected to a network)
- The collector can send a `status` message to the server which contains what data sources are available, what data sources are being logged, when each data-source was last logged, it's current config,
- It supports hot-reload of its config
- It can be toggled on-off by the user or other programs
- It has an audit log of everything that happens on the collector (such as when it sends data, when and what the server requests, the ip of that server, etc)
- It is run automatically upon boot
- Each collector should have an interface so the user on that device can see what is up?
- If anything fails, it should report that to the server
- Each collector has a `meta` database that contains its own log. that log should be sendable to the server
- Collectors automatically detect and connect to servers

#### Buffers

Every collector will have a buffer of a fixed size (defined in the configuration.toml), it can also have an adaptive size based on the currently used disk space (we can define the minimum amount of free space on our device). We can have an in-memory buffer or a in-disk buffer.

QUESTION: Should we have an in-memory or on-disk buffer?

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
- It supports hot-reload of its config
- It creates backups on the device and can interface with different types of data storage (SSD, HDD, and cloud).
- It has an audit log of everything that happens on the server
- It is run automatically on boot
- Server can compress the data for storage depending on the data types

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

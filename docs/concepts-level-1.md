# Lifelog

The vision for the project is a software system that allows users to store information about themselves from various data sources locally, process their data into more meaningful representations, and finally have an interface to be able to interact with it in an intuitive manner to help them complete their tasks.

## Principles

Store everything, don't lose anything, don't delete or overwrite anything,
Everything that is configurable should be configured in a single config.toml file.

## Requirements

- Be able to send gigabytes of data at a time between the collector and server.
- Be able to send data of many different types, be flexible in the messages sent
- Be fault-tolerant at the collector and server side so no data is lost
- Support incremental updates of the main database
- Strongly typed

## Concepts

`data-source`
A data source is one source of data. It is something that is very specific and it is a location of where data comes from. A data source has a unique data modality, but a data modality can come from many different sources. Some example of data sources are: computer screen versus laptop screen are both computer screens, but they are different data sources. A data source can be a file on disk, a network connection, a device, etc.

```rs
struct DataSource {
    name: String, // human-definable name
    location: String,
    device: Device, // the device it is on
    modality: DataModality, // the type of data modality
}
```

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

- Logger trait:

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

[[./collector.md]]

#### Buffers

Every collector will have a buffer of a fixed size (defined in the configuration.toml), it can also have an adaptive size based on the currently used disk space (we can define the minimum amount of free space on our device). We can have an in-memory buffer or a in-disk buffer.

QUESTION: Should we have an in-memory or on-disk buffer?

#### Sending information

The collector might want to send information to the client through a network, through bluetooth, or through a wire. We should enable our code interface to be agnostic to what information method is used under the hood. To do this, we can have a function that is called `send_data` that reads the `config.toml` and chooses the method to send data, it also can support multiple methods of sending data and receiving data to the server.

The server has the same thing on it's end, where it can listen on all of these methods (listen on bluetooth, through wire, etc.) so that it is medium agnostic.

### Server

[[./server.md]]

#### Transforms

A transform is something that takes data of one data type A and applies some transformation of it to create something of another data type B, where A could equal B.

A transform is an abstract term. It is any general function. It can be an ML model applied to the data, some filtering/signal processing, it can be an API call to another server to do something with the data. Only thing is that it acts on existing data.

If there is an update to the transform (using a different model) then store the original? or use the original?

Functions:

```rs

// F - from type, T - to type
trait Transform<F: DataType, T:DataType> {
    // Takes in the input data and outpust the new data type
    fn apply(&self, input: F) -> Result<T, TransformError>;

    fn new(config: TransformConfig) -> Self;

    // Returns the name
    fn name(&self) -> &'static str;

    // Returns the priority of the transform
    fn priority(&self) -> u8;
}
```

For every data type, there can be a transformation pipeline defined for it
![TransformationGraph.svg](TransformationGraph.svg)

With this pipeline, when the server gets some input data from the collectors it will apply the approriate transformation pipeline to the collector data and then send it to the database.

A transform has a name, and a function `apply` that takes in the input data and then outputs the new data.

Transforms, like data modalities, should be easily extensible. Other applications should be able to 'register' transforms with the server so it knows they exist and the server can run it. In the future, other applications might want to create their own transforms to extract data from the lifelog.

##### Cool Transforms

Agent that takes notes - Define a trasnform that is an agent that maybe can extract key words

#### Database

One database for this entire project. It could be moved to separate databases in the future to take advantage of them (a vector specific db for vector queries, a elasticsearch for text-queries).

One table per data source.

Data sources are unique and are based on device + data modality.

The database is used for persistent storage of the lifelog and other data as well as quick retrieval of data.

##### Transformation Pipeline

A transformation pipeline is a graph of a bunch of transforms. It is a directed acyclic graph and it is defined for one input type (in the future maybe multiple input types?).

### Interface

This is the lifelog interface, it will be an interface for the user to be able to access and view their lifelog. They will be able to look at all of their data modalities and be able to query them. This will be the centeralized way the user can inferface with their lifelog.

Upon connection to the server, the user must authenticate before accessing the lifelog.

#### Arbitrary interfaces

It would be very cool if for this project we could define an API for other applications to be able to 'query' something abou tthe user. these queries can be things such as 'how prodductive am I at this moment', 'when was I sick' so that other applications or services can use this data. the user can define exactly waht data to shrae and there are strict policies with these applciations. these apps can be only open sourced, or allowed by the user.

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

Use TLS between the collector and the server to ensure that the data being sent is encrypted and not readable by a third party. This will also ensure that the data being sent is not tampered with.

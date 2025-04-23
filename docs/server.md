## Server

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
- It can process the data and do transforms on it.
  There is a transformation pipeline that is defined for every datatype, it is a DAG where the nodes are intermediate data points or 'actions' and the edges are transformations. The intermediate data points can be stored
  What is a transformation?
  A transformation is a function that takes in data of some type and outputs data of another data type where they can be the same data type.

```rs
struct Server {
    dbPool: DatabasePool, // database pool to connect to the database
    name : String, // name of the server
    location: URI, // location of the server (ip address, bluetooth address, etc)
    config: ServerConfig, // configuration of the server
    database: Database, // database of the server
}
```

gRPC methods:

`RegisterCollector` RPC:

- This is called when a collector wants to be registered, it will send its config and request to be added to the registered collectors on the server. (Maybe) do some security here?

Server bootign sequence:

1. Turns on, reads config from the config path `~/.config/lifelog/config.toml`, or maybe we can move it to `~/.lifelog/config.toml` or `~/.local/share/lifelog.toml`
2. Starts database connection (or a pool of connections)
3. Starts the server for collectors to send data to
4. Looks at what jobs are defined for the server and start running them

#### Cool Transforms

Agent that takes notes - Define a trasnform that is an agent that maybe can extract key words

#### Policy

The server has a policy that allows it to make decisions for whatever decisions the server needs to make. These decisions include:

- When should I request data from the collectors?
- How much CPU usage should I use?
- How much network traffic should I use?
- For all data sources:
  - Of the avaliable transforms, what transform should I run at this instance.
- Should I create a backup, if so, of what data?
- Should I compress any data?
- Should I re-transform any data (suppose)
- Should i train any models of data
- Does the current query require these data sources that aren't transformed.

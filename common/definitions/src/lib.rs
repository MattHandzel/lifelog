use target_lexicon::Triple as ComputerTargetTriple;

enum PhoneType {
    Android(AndroidOperatingSystem),
    IPhone(IPhoneOperatingSystem),
}

enum ComputerType {
    Desktop(ComputerTargetTriple),
    Laptop(ComputerTargetTriple),
}

enum DeviceType {
    Phone(PhoneType),
    Computer(ComputerType),
}

struct Collector {
    name: String,                                      // name of the collector
    device: DeviceType,                                // type of device (phone, computer, etc)
    location: URI, // location of the collector (ip address, bluetooth address, etc)
    config: CollectorConfig, // configuration of the collector
    state: CollectorState, // state of the collector (state of the collector and all data sources, loggers)
    data_sources: DashMap<DataSourceType, DataSource>, // data sources available on the device
    grpc_client: GrpcClient, // gRPC client to communicate with the server
    security_context: SecurityContext, // security context to ensure the data being sent is not tampered with
    command_tx: mpsc::Sender<CollectorCommand>, // commands to send between threads
    command_rx: mpsc::Receiver<CollectorCommand>, // commands to send between threads
}

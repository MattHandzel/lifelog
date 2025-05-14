TODOS:

- Change the register collector to give the mac of the device
- Refactor the code in the sync_data function to be "universal"
- Think about the query, get data pipeline for the collectors
- Send queries out to the frontend
- Implement a better system for my rust/proto files. A huge problem I have is I am restricted in the types of the structs I define because I have to create the macro that translates them. A better way would be to define proto file and generate the rust types, but I also lose some flexibility I gain from macros (for example, I automatically implement a DataType trait, and I can do some codegen). Is there a way I can get around these problems?
- Right now, macros are written before build script which causes an error that types aren't defined. Fix this by moving everything to proto files 😭

- Create transformation pipeline
- Store blobs in a file

- Improve the collector configs, get a new configuration pipeline for this project (server, collectors, etc.)
- Implement the get and set config functions for the collectors and server

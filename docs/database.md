A database that the server uses, we are using SurrealDB:

Database schema:

- [ ] Have an audit log of all things the server does (all actions)
- [ ] For each collector:
      Have a table for each data source (example: SOURCE_MAC:<data-modality>). if the location doesn't come from a device then the SOURCE_MAC is omitted
      When a collector is registered: - [ ] Create a new table for each data source the collector could have. Use the structs of the data modalities for the schema
      When a piece of data comes in, match on the data type and add it to the table for that data type.

- [ ] For a transform
  - [ ] When a new transform is registered, it will take in a data modality and then do a transform on that data modality. This looks like the server choosing to run that transform, if there is a table with the name (SOURCE*MAC:<data-modality>*<transform-name). The transform queries the source with its query and finds all data with a uuid that is not currently in its table. With that data, it runs its transform and adds it to it's table.

What if I have an object store database for the "BLOB" like data (images, audio). Maybe the database should only have "high entropy" content. With this way of thinking, lifelog might be able to easily add extend other forms of merdia (such as uploading a file/book/video). TODO: Think about this more.

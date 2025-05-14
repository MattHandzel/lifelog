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

### Feature Ideas

- Under the assumption the server is the `only` entity that interacts with the database, we don't need this feature, but if there are multiple writers then we might want to leverage surreal db's events to call transforms instead of calling transforms at other points in time. I think this is a good idea.
- Leverage time versioned tables so, suppose for a demo i can make my database go back in time, and then resume when not doing a demo
- Add data constraints onto transforms, take them from the rust structs? (example is paths)
- Should i have the database do the search/indexing or the server? prolly database
- I dont know if we want to go thiiss deep, but allow the users to define a "aggregate" view as described here: https://surrealdb.com/features
- We can use surrealdb's live queries for any query the user makes? Does that make sense for us to use that if we know exactly when more data is used?
- Add database export to UI using surreal sql.

### Querying

For each data modality there might be different ways of querying the data, some examples:

"Set"-based search (contains this keyword or not, contains object or not).

- There are real numbers, ordinal sets, and categorical sets. When selecting from categorical sets among features we can do all the boolean operations between them). With ordinal sets we can do some relationship (<= some value), same with real numbers
  vector-based search (semantic search, sparse keyword vector)

#### Text

- -> Dense embedding (semantic searching)
- Sparse (keyword searching)
- Fuzzy (fuzzy searching)
- Regex (For any symbols)
- -> Named entity recognition
- -> Sentiment analysis
- -> Topic modeling?? (maybe no bc we have dense search)

#### Camera

- Semantic search
- Image search (based on image itself)
- Regex on image???
- Verticle/horizontal image
- -> Object detection & location
- ->

- How do I deal with videos being uploaded/GIFs (multiple images together)

#### Screen

- Semantic search
- Image search
- Regex on image???
- Screen size/orientation

#### Microphone Audio

- (loudness, pitch, search)
- Audio-audio search
- audio-audio embedding search (like shazam)
- -> Audio-text search (transcription)
- -> Multi-modal Embedding (audio-image-text) search

#### System Audio Output

- Audio source (application)
- Audio volume
- -> audio type (meeting, music, etc)
- -> audio-text search (transcription).
  - How to search through this? If i represent everything as (timestamp, word) or (timestamp, sentence) what if the query is across sentences/words? What if there is a mistranslation. It seems like there needs to be a specific text-audio transcription engine. Wait because we have our own search-method might there be a problem since surrealdb doesn't have this innate?
- output device
-

#### Health Data

##### Sleep data

- Sleep quality
- Sleep time

##### Steps

- Step count
- Step time

##### Workouts

- Workout time
- Workout type

#### Calendar

- Event time
- Event name
- Event duration
- -> event type (school, work, etc)

#### Person with

- (images, audio, calendar, strava workout, message, etc) -> person id

I'm curious, could you re-explain what the students scores are? How do you know how "visionary" a person is or how good of a collaborator they are based off their essay? it seems like you don't have that much data.

#### Book read

- (screen, system audio output) -> book content/name

#### Location

- Store location, then create "geo-fences" for the user to store? Hmmm this is it's own program but surrealdb could handle these queires.

### Searching

Generating indicies for the database:

- Right now don't have any fancy search. If someone wants to search the database for a specific string, then they need to generate a query that

Searching is just querying but with a better index?

There seem to be 3 approaches

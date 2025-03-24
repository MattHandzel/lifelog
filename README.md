# Lifelog

The vision for the project would allow the user to record their own data from many different data sources, store their data securely in a local database, and have an interface to be able to query it with natural language.

# LifeLog-Logger

This is a logger that you can use to help log your life! It captures data of many different modalities.

#### Data Modalities

- [x] Screen
- [x] Microphone
- [x] Processes
- [x] Hyprland specific information (like the current window title, all clients)
- [ ] Browser history, browser analytics
- [ ] Application-specific information (like the current song playing on Spotify, current file being used in neovim, etc.)
- [ ] Web-app specific information (instagram messages, youtube watch history (and analytics, videos liked, etc.))
- [ ] Who you are interacting with (like who you are messaging on discord, who you are interacting with in real life)
- [ ] Smartwatch data (like heart rate, steps, etc.)
- [ ] Terminal commands
- [ ] Clipboard history
- [ ] Events (such as calendars)

#### Features

- [ ] Intelligently capture data. I.e. instead of based on time, capture data when events are happening (the screen is changing, the user pressed a button to change windows, etc).

### Benchmarking

Here is an estimate of how much data each modality will generate:

`hyprland` 3600 logs = 6 MB \~= 0.0016666 MB/log
`screen` \~256kb/screenshot

### TODO

Should everything be a database? Should I have one database for each modality? Or should I have one database for all modalities?

### IDEA

Be able to go `back` in time to that event would be very cool.
Be able to stop/change the sources of information easily
Automatically not store data based on what it could be used for?

# Lifelog Server

The lifelog server created `a unified, digital record` from the logger that processes, stores the data into meaninful information.

It should be able to work with software services and allow the user to select what data for other services to use. Other companies cannot take any data from your lifelog.

Features:
Memex (Memory extender): Be able to recall what you were doing at a given time, what you have read, query your own database
Retrieve important information you don't know you need
Stores copies of data, when data is manipulated store original
Have a version of this software for vulnerable populations, one that has metadata and not storing the real data?

# Lifelog Interface

Features:

- Manipulate data, fix erronous data
- Remove information
-

### References

This paper helped inform this project:
https://link.springer.com/article/10.1007/s11948-013-9456-1

### Todo:

    Creating an GUI interface for people to more naturally interface with their data
    Taking natural language and converting it into database queries
    Being able to query multimodal data (video, audio, and text)
    Refactoring the database to better handle queries (right now it's a SQL database)
    Improving the interface for search
    Port the software onto Windows and Mac, develop more logging software.

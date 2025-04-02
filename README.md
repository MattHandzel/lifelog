# Lifelog

The vision for the project would allow the user to record their own data from many different data sources, store their data securely in a local database, and have an interface to be able to query it with natural language.

# LifeLog-Logger

This is a logger that you can use to help log your life! It captures data from many different modalities.

#### Data Modalities

- [x] Screen
- [x] Microphone
- [x] Processes
- [x] Hyprland specific information (like the current window title, all clients)
- [ ] Browser history, browser analytics
- [ ] Application-specific information (like the current song playing on Spotify, current file being used in neovim, etc.)
- [ ] Web-app specific information (instagram messages, youtube watch history (and analytics, videos liked, etc.))
  - [ ] [Instagram](https://www.the-sun.com/lifestyle/tech/272081/how-to-download-all-your-instagram-photos-stories-and-videos-quickly/)
  - [ ] [Reddit](https://www.reddit.com/r/DataHoarder/comments/800g94/any_way_to_download_reddit_profile/)
- [ ] Who you are interacting with (like who you are messaging on discord, who you are interacting with in real life through audio logs)
- [ ] Activity watch and other loggering software
- [ ] Smartwatch data (like heart rate, steps, etc.)
- [ ] Terminal commands
- [ ] Clipboard history
- [ ] Events (such as calendars)

#### Features

- [ ] Intelligently capture data. I.e. instead of based on time, capture data when events are happening (the screen is changing, the user pressed a button to change windows, etc).
- [ ] Synchronize data to server

### Benchmarking

Here is an estimate of how much data each modality will generate:

`hyprland` 3600 logs = 6 MB \~= 0.0016666 MB/log
`screen` \~256kb/screenshot

### IDEA

Be able to go `back` in time to that event would be very cool.
Be able to stop/change the sources of information easily
Automatically not store data based on what it could be used for?

# Lifelog Server

The lifelog server created `a unified, digital record` from the logger that processes, stores the data into meaninful information.

It should be able to work with software services and allow the user to select what data for other services to use. Other companies cannot take any data from your lifelog.

Features:

- Memex (Memory extender): Be able to recall what you were doing at a given time, what you have read, query your own database
- Retrieve important information you don't know you need
- Stores copies of data, when data is manipulated store original
- Have a version of this software for vulnerable populations, one that has metadata and not storing the real data?
- Send commands and signals to all loggers

# Lifelog Interface

Features:

- Manipulate data, fix erronous data
- Remove information
- See last time a data source has been added to, it's size, etc
- Add a button to request social media data, or a button to request that a data source is "synced" between all devices
- Be able to see your data in human understandable format, scrub through audio, video, be able to search/fuzzy find through text. Create components for each data modality. Searching through one data modality also searches through the other.

### References

This paper helped inform this project:
https://link.springer.com/article/10.1007/s11948-013-9456-1

https://x.com/vin_acct/status/1876088761664385346
https://github.com/nanovin/gaze
https://github.com/openrecall/openrecall

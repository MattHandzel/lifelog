## Data Modalities

- [x] Device Screen
- [x] Device Microphone
- [ ] Device audio
- [ ] Device camera
- [x] Device input
- [x] Device Processes
- [ ] Deskctop environment information (like the current window title, monitors connected, current workspace)
  - [x] Hyprland
- [ ] Browser history, browser analytics
- [ ] Applications/APIs
  - [ ] Email
    - [ ] Outlook
    - [ ] Gmail
    - [ ] Thunderbird
  - [ ] Messaging platforms
    - [ ] Instagram
    - [ ] Discord
    - [ ] Whatsapp
  - [ ] Youtube (watch history, videos liked)
  - [ ] Calendar
    - [ ] Google calendar
    - [ ] Apple calendar
  - [ ] Apple Health
  - [ ] Samsung Health
  - [ ] Medical records lol
- [ ] Location
- [ ] Smartwatch data (like heart rate, steps, etc.)
- [ ] Terminal history
  - [ ] Bash
  - [ ] Zsh
  - [ ] Fish
- [ ] Home assistant devices
- [ ] Devices networked (using wireshark)

### Inferences

- [ ] Who you are interacting with (like who you are messaging on discord, who you are interacting with in real life through audio logs)
- [ ] Tasks you are doing (reading, writing, doing homework)
- [ ] Objects in the environment (from camera, microphone)
- [ ] Environment (bedroom, outside, etc.)

## Configuration

Everything in this project is configurable through a `config.toml` file located at `~/.config/lifelog/config.toml`. There is also a planned GUI interface for updating the config.

## LifeLog-Collector

This is a program that when run on a device will activate other modules to record data automatically and in the background.

#### Features

- [x] User-configurable modules
- [x] Adapt modules based on device compiling
- [ ] Write to database
- [ ] Capture data at meaningful times. I.e. instead of based on time, capture data when events are happening (the screen is changing, the user pressed a button to change windows, etc).
- [ ] Write over network
- [ ] Fault tolerance
- [ ] Encrypt logger data

## Lifelog Server

The lifelog server handles requests, does data processing, and communicates with loggers.

It should be able to work with software services and allow the user to select what data for other services to use. Other companies cannot take any data from your lifelog.

Features:

- [ ] Connects with a database
- [x] Communicate with collectors over a network
- [ ] Collectors can send data to server
  - [ ] Automatically create into and from's for prost OR refactor code to use prost types and define them all in proto files
- [ ] Securely process queries (TLS)
- [ ] Open up an API for other applications
- [ ] Data management
  - [ ] Create backups of data on other devices (HDDs)
  - [ ] Secure data on device
- [ ] Create transforms for data
- [ ] Query processing
  - [ ] Takes database queries and executes them
  - [ ] Time, location, object parsing
  - [ ] Processes natural language queries and converts them into a series of database queries
- [ ] Data compression
- [ ] Implement differential privacy measures?

##### Data Transformations

Images:

- [ ] Object recognition
- [ ] OCR
- [ ]

Audio:

- [ ] Speech to text
- [ ] VAD

Text:

- [ ] Sentiment analysis
- [ ] Named Entity Recognition
- [ ] Text embedding

Location:

- [ ] Weatherhttps://dl.acm.org/doi/10.1145/3592573.3593106
- [ ] Luminosity
- [ ] Air pollution (environment data on that date)

# Lifelog Interface

This is the lifelog interface, it will be an interface for the user to be able to access and view their lifelog. They will be able to look at _all_ of their data modalities and be able to query them. This will be the centeralized way the user can inferface with their lifelog.

#### Features:

- [ ] Interface for every data modality
  - [ ] Have combined interfaces for looking at multiple data modalities
- [ ] Disable loggers for a short period of time
- [ ] Query
  - [ ] Audio queries
  - [ ] Image queries
  - [ ] Text queries
  - [ ] Multimodal queries
- [ ] Take relevance feedback from user
- [ ] Update config from interface
- [ ] Connect with server
- [ ] Annotate data
- [ ] Update, fix, remove, archive data
- [ ] System panel (loggers, whether they are https://dl.acm.org/doi/10.1145/3592573.3593106active, last time written)
- [ ] Be able to 'take a snapshot' with all loggers
- [ ] Manually activate individual loggers
- [ ] Manually activate jobs (for processing data)

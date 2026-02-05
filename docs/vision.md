## Lifelog System

This system gives a super-power to the user, the ability for them to store arbitrary data about themselves, instantly retrieve anything about themselves, run data transforms on their data, and have any application leverage this data to improve their lives.

There are three entities in this system, collectors, server, and agents.

#### Collectors

Collectors are device-specific programs that collect data about the user. The user installs these collectors per device and configures them to collect the information they want about themselves. [[./collector.md]]

#### Server

Server is a central server that stores the data collected by the collectors and responds to requests by agents. It does data processing, synchronization, data management. [[./server.md]]

#### Agents

Agents are applications that can permissions to read and write into the lifelog system. They can leverage the lifelog system to improve their own function (such as external apps), or they can add information to the lifelog. The permissions they can have are:

- Read
- Append
- Write

The user explicitly and securely defines the permissions of these agents to follow the principle of least privillege. An example of an agent could be your primary medical facility:

They can read specific information about yourself, this can be raw information (such as heart rates), but they can also request to have some transformations done on the data (such as running a `hours-of-exercise` transform and extract just the table `hours-of-exercise`.
They can append information to your lifelog (such as a doctor writing a note about your health, any tests you have done).

They can write information to your lifelog such as rewriting your medical history if you thought you had some disease but it later is revealed you had another disease.

#### System Properties

- This software sets up a system that has the following properties:
  - Easy to setup per device, even non-technically savvy users can set it up
  - Security as a priority
    - Secure communication
    - Explicit permissions of users of the system
  - Information extraction & combining for better, more salient information
  - Intelligent, east search
  - Easy to use, high skill ceiling (add shortcuts, ways to interface that are easy)
  - Extensible
    - Naive users can define more data modalities without explicitly writing code (pipeing output of a file into the system)
    - More advanced users can pipe information from their lifelog (such as less sensivity information such as `driving-car` into other applications)
  - System works in real time (i.e. if configured the user can immediately see what information they have just captured)
  - Adaptors for any external API
  - Standard data formats
  - Easy to upgrade
    - Easy to update data transforms
    - Easy to update software
  - Allows the opt-in ability for third parties to leverage their data (scientific studies)

# Lifelog

The vision for the project is a software system that allows users to store information about themselves from various data sources locally, process their data into more meaningful representations, and finally have an interface to be able to interact with it in an intuitive manner to help them complete their tasks.

## Installation

#### Linux

##### NixOS

Use the `flake.nix` 😀

#### MacOS

## System Diagram

![System Diagram](./docs/Lifelog.drawio.svg)

### References

Some references used for this project:

```
https://link.springer.com/article/10.1007/s11948-013-9456-1
This talked about challenges and feasibility of lifelog software

https://x.com/vin_acct/status/1876088761664385346

https://github.com/nanovin/gaze
https://github.com/openrecall/openrecall
These two are some other software that try to do the same thing. Copied some code from nanovin.

[ImageBind: One Embedding Space to Bind Them All](https://arxiv.org/pdf/2305.05665)
This paper talks about and shows some very cool examples of the benefits of having one embedding space for many different data modalities.

[LifeInsight: An interactive lifelog retrieval system with comprehensive spatial insight and query assistance](https://dl.acm.org/doi/10.1145/3592573.3593106)
This paper gave some ideas for how to do the relevance feedback. Some cool ideas are to use LLMs to refine queries and to allow the user to select data modalities to add to queries
```

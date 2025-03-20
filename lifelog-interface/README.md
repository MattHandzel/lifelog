# Lifelog Interface

Goal of this interface is to be able to query my lifelog

Ideas of use cases:

- I remember watching a youtube video that was a talk about lifelogging. I want to find the parts of the video where the author talked about lifelogging research because I want to more intelligently design my lifelogging system. I remember sending it on the CS510 campuswire. I want to find it even if I am not longer in the class. I remember watching the whole youtube video without having to go through all the messages.
  Data modalities: - clipboard history (can regex for clipboard) - Current website being used - filter dates - idea of Youtube title - (Potentially) messages on campuswire - Videos being watched/idle time - Youtube watch history (and amount)
- I remember writing this function that does X Y Z for this project, i remember working on it with neovim. Pull it up.
  Data modalities: - all files on my computer - files I wrote using neovim - github commit history

#### Interface Methods

Each query the user does can be turned into something like an SQL statement. There are a lot of different possible modalities. Here is how I am thinking queries are turned into code representations:

`I remember watching a youtube video that was a talk about lifelogging. I remember sending it on the CS510 campuswire.`

For this, it seems like the system would need to be able to extract objects and understand that the same YouTube video watched about lifelogging was also sent on campuswire.
It would also need to extract the modalities to search through, the return modality (which would be some bit of text, or a screenshot of the video), and any helpful constraints.
It should be able to break this query down into:

- Look through Youtube search history for a video that has the title "lifelogging"
- Return it
  OR
- Look through browser history
- Find times when user is on campuswire and in the CS510 channel
- Do OCR for youtube links, or lifelogging
- Return all images found with youtube links

This is a **hard** problem, maybe have the user specify the prompt.

For example, text="Lifelogging talk on youtube", modalities="youtube, screen, browser, clipboard", return="screenshot OR text". It is not bad for the user to refine their search results.

#### Interface

The user will be able to select the data modalities they want to see from a drop-down menu. Then the data modalities will pop up in a table. On the bottom there will be an element that selects the current time, and it will pull up the associated data modaility.

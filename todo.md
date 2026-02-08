Create and identify a workflow for doing integration test for multiple, separate devices. Mock these integration tests and also include an expensive integration test where, using VM's, phone emulators, etc. We are able to test the multi-device nature of this software. The goal is to find and catch any bugs when it comes with syncing.

Plan out how to do these tests, different tiers of tests depending on what features were added or changed, how to get extremely accurate tests that simulate multiple devices. What are frameworks to do this testing so we can test real-world problems (such as connection dropping, etc). Brainstorm how to do in-depth testing of this project, you have full autonomy and freedom.

---

A general problem: compilation time takes quite a long time. how can we speed it up? surrealdb-core is the main culperate, but we should keep that. Remember, speed is quality, there are huge gains in doubling the iteration speed. Please reflect on what is taking the most time.

---

Please read the SPEC. We are going to be doing a large refactor of the front-end. I completely removed the frontend because I did not like it Please redesign the frontend from scratch. Think deeply about how to do this, and how to best serve the user.

---

I want to set up this software so that my laptop has a collector and my main server (this device) is running the server. What else needs to be done for this goal to be realized? What are some other collectors that should be added. What do I need to do on my laptop and server to verify this works. What setup needs to be done on both devices? How can you, after I set this up, validate that this works in a real/live environment.

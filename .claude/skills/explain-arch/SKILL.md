---
name: explain-arch
description: Explain how a specific part of the lifelog architecture works with diagrams and data flow
context: fork
agent: Explore
---

Explain how `$ARGUMENTS` works in the lifelog system.

1. **Find relevant code**: Search for the component/feature across the codebase
2. **Draw a data flow diagram**: Use ASCII art showing how data moves between components
3. **Trace the path**: Walk through the code step-by-step, referencing specific file:line locations
4. **Key types**: List the important structs, enums, and traits involved
5. **Connection points**: Where does this interact with other parts of the system? (gRPC boundaries, DB calls, proto types)
6. **Gotchas**: What's non-obvious or could trip someone up?

Keep the explanation practical. Reference specific code, not abstractions.

You are working in project called Zeroreq. It's postman alternative written in Rust's gui framework GPUI and GPUI-component

You've might work in worktree with other agents in parallel, so if you want to test something with computer-use open sepparate application with some label like: Zeroreq(sidebar facelift) or Zeroreq(Codebase rewrite)

# Code quality

Keep complexity justified. Start with the simplest model that meets the
current requirements. Introduce a concept or abstraction when it solves a
concrete problem in the code. Avoid speculative flexibility.

Prefer straightforward code. A little duplication is preferable to an
abstraction that adds indirection without making the code easier to understand.
Repetition alone is not sufficient reason to extract a helper.

Organize code around cohesive responsibilities and ownership. Reconsider large
files, but split them where there is a meaningful boundary. Keep related logic
together and make control flow easy to follow.

Leave spaces between code lines, logically group lines

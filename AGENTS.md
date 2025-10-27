This project is a timeline visualization of video game consoles, showing when
each console was released, discontinued, and various kinds of other data about
each console.

`src/data.rs` contains all of the data. It should be possible to discern
structure from that file, but if need be, the structs are defined in
`src/model.rs`.

When adding data, AI agents are NEVER allowed to specify a source and must
ALWAYS write `source: None`. Fact-checking is exclusively done by humans. The
only exception is when a human directly tells an AI that a specific point of
information is true, and gives a source. Only then can an AI add that source to
the specified data point.

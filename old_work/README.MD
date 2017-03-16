Here you'll find some tests/implementations that didn't work out so well

## python_fuse_system
python_fuse_system is a really basic FUSE implementation that just proves that
I can unify a bunch of package dirs into one directory. It isn't really fast,
but it isn't meant to be.

## Weave
weave was actually a prototype of this system that I wrote in Go. The dependency
solver is heavily multithreaded using goroutines, but I'm really not sure that
those give it a significant speed advantage. I do know for a fact that it makes
everything messy to maintain and debug. Once I finish my Rust implementation, I
might do some testing to evaluate if it's worth threading it, in which case I'll
use some of the architecture designed here to do so.

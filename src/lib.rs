// We need to create an abstraction for an index, use parallel collections to search
// quickly across the files. We need to keep in-memory index of a file structure.
// We should add a benchmark for each request to make sure we return an answer within a
// second.
//
// We could create a structure where we keep some sort of bitmap for each directory, so
// we can quickly discard directories where we know search query would return nothing.

extern crate grep;
extern crate ignore;

// We need:
// - list of supported extensions
// - configuration struct

const SUPPORTED_EXTENSIONS: [&str; 2] = [".scala", ".java"];

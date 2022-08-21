# About

Alfred workflow to search for repos and open them.

## Installation

### Building from source

1. install Rust and Cargo using [rustup](https://rustup.rs/).
2. clone
3. install in alfred: `make install`
4. set environment variables according to your setup:
    1. `DIRECTORY_PATH`:
       - Full pathes to directories, which you want to search in.
       - Separator is a comma. e.g. `/home/user/repos,/home/user/private-repos`.
       - Omit the separator, if you only want to scan one directory.
    2. `BINARY_TO_EXECUTE`: Full path to binary you want to execute with the selected path.

## Debugging issues

To see all output from the workflow you can run the following open the workflwo in debug mode.

## Credits

* This repo used https://github.com/rossmacarthur/crates.alfredworkflow/ as a template.
* To generate the binary: https://github.com/rossmacarthur/powerpack
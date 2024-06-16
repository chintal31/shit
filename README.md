# Shit: A CLI app resembling a Git client

## How to Run the App

### Compiling, Building, and Running the Executable

To compile, build the executable, and immediately run it:

`cargo run`

### Building the App without Running

To build the app without executing it:

`cargo build`

### Adding Shit to $PATH

To make the `shit` command available globally in your terminal (assuming you use Zsh):

1. Open your `~/.zshrc` file in a text editor:

2. Add the path to your Shit executable. Cargo run puts the executable in `/path/to/shit/target/debug`, add this to `~/.zshrc`.

3. Restart your terminal or run `source ~/.zshrc` to apply the changes.

## Usage

### Displaying Help

To get help for the available commands:

`shit --help`

### Initializing a Git Repository

To initialize a Git repository (similar to `git init`):

`shit init --name <name>`

Replace `<name>` with the desired repository name.

### Adding a File to the Index

To add a file to the index (similar to `git add`):

`shit add --file-name <file-name>`

Replace `<file-name>` with the name of the file you want to add.

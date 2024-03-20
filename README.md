# locatex

## Objective
The **locate** (Unix) command is particularly useful for quickly finding files and directories without having to search through the entire filesystem, when 
doing such tasks in **Windows** the File Explorer gives us all a hard time; for that reason I am making **locatex** as a locate alternative available for Windows users.

## Functionality


## Installation


## Usage


## How it Works
The tool internally works in the same way of the locate command, it scans the file system and creates a local snapshot as a *SQLite* database that is leveraged when 
searching for a file or a directory.

The tool is split up into two packages: 

- `locate` provides a command line interface written in **Python** to the file system snapshot.
- `scanner` is responsible for indexing the file system and creating the file system snapshot, that's written in **Rust** for better performance. 


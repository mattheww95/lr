# lr - ls but in Rust (and not as good)

## What is this?

When learning a new language I like to start by trying to re-write `ls` as it I feel like it is a good starter program. I don't think this is very good as it currently stands however as it offers alot less functionality then ls, nor does it behave the same. 

I do want to try and make this useful later on however, perhaps by adding the ability to use this functionality on URLs to list files in FTP sites.


## Building

Simply running `cargo build --release` should take care of everything.


## Usage


```
Usage: lr [OPTIONS] [FILES]...

Arguments:
  [FILES]...  List of directories

Options:
  -c, --colourize  Colour files/folders by type
  -h, --human      Show file sizes in a human readable format
  -l, --long       Print long form output
  -a, --all        Print all items in directory
      --help       Print help message
  -V, --version    Print version
```

# **Riv** the **R**ust **I**mage **V**iewer

Why riv? This project was born out of a frustration with image viewers on Mac. 
Generally the options are:-

* iPhoto - Way too heavy for just viewing images
* Preview - Clunky and really only good for viewing one image at a time
* Others that require a GUI folder browser

Riv on the other hand runs from the command line, and accepts a path with globs in quotes (with globstar `**` for recursive search). For example:-

```$ riv "**/*.jpg"```

## Manual

Start riv with 

```$ riv```. 

As an optional second parameter you can add a path with globs.

```$ riv "**/*.png"```

Without any second parameter, riv will look for all images in the current directory.

Set a destination folder for moving files with the `f` flag. The folder will be created if it doesn't exist.

```$ riv -f ~/saved_images```

Set a sorting order with the `s` or `--sort` flag, case insensitive.

```$ riv -s alphabetical "**/*.png"```

### Normal Mode Controls


| Key 1      | Key 2                      | Action                                              |
|------------|----------------------------|-----------------------------------------------------|
| q          | Esc                        | Quit                                                |
| k/j        | Left/Right                 | Previous/Next Image                                 |
| i/o        | Up/Down                    | Zoom in/out                                         |
| H, J, K, L | Shift + Up/Down/Left/Right | Pan left/down/up/right                              |
| b/w        | PageDown/PageUp            | Backward/Forward 10% of images                      |
| g/G        | Home/End                   | First/Last Image                                    |
| m          |                            | Move image to destination folder (default ./keep)   |
| c          |                            | Copy image to destination folder (default ./keep)   |
| d          | Delete                     | Delete image from it's location                     |
| t          |                            | Toggle information bar                              |
| f          | F11                        | Toggle fullscreen mode                              |
| ?          |                            | Toggle help box                                     |
| z          | Left Click                 | Toggle actual size vs scaled image                  |
| Z          |                            | Center image                                        |
| . (period) |                            | Repeat last action                                  |


### Command Mode Controls


| Short | Long       | Argument | Action                              |
|-------|------------|----------|-------------------------------------|
| ng    | newglob    | Required | The new glob/directory/file         |
| ?     | help       | None     | Toggle help box                     |
| q     | quit       | None     | Quit                                |
|       | sort       | Optional | The method to sort by               |
| df    | destfolder | Required | New folder to move/copy images to   |
| m     | max        | Required | New maximum number of files to view |

### Sorting Options

| Option           | Description                                                                              |
|------------------|------------------------------------------------------------------------------------------|
| Alphabetical     | Alphabetically by filename only                                                          |
| Date             | By date last modified, most recent first                                                 |
| Size             | By size, largest first                                                                   |
| DepthFirst       | [Default] Ordered by farthest depth from current directory first                         |
| BreadthFirst     | Ordered by farthest depth from current directory last                                    |

Reverse the sorting order with `r` or `--reverse` flag

```$ riv -sr date **/*.png```

Set the maximum number of images to be displayed `m` or `--max` flag. 0 means infinitely many images.

```$ riv -m 0 **/*.png```


## Getting Started

These instructions will get you a copy of the project up and running on your local machine for development and testing purposes.

### Prerequisites

You will need to install Rust and the SDL2 libraries to work with this project.

### Installing

Go [here](https://www.rust-lang.org/) for instructions on installing rust.
Go [here](https://github.com/Rust-SDL2/rust-sdl2) for instructions on installing SDL2.

You will also need sdl2_image and sdl2_ttf

#### Mac

`brew install sdl sdl2_image sdl2_ttf`

#### Arch

`sudo pacman -S sdl2 sdl2_image sdl2_ttf`

#### Ubuntu

`sudo apt-get install libsdl2-dev libsdl2-image-dev libsdl2-ttf-dev`

#### Other distros

Hopefully you can figure it out from the above instructions. If you do, please make a PR for this README with the specific instructions.

After that you can build with:

```cargo build```

## Contributing

I aim for this project to be a great place for people just starting with Rust and just starting with Open Source to get involved. I'm pretty green with Rust myself, so any code review, refactorings to idiomatic style, bug fixes and feature PRs are very much appreciated. I have purposely left some features unimplemented before open sourcing with the idea that someone can pick them up as a good first contribution. So please, join in. No developer is too green for this project.

Never made a pull request before? Check out this [5 minute video](https://www.youtube.com/watch?v=rgbCcBNZcdQ) which explains a simple process. Remember to make pull requests against the development branch.

Not sure what to work on? Check out our issues.

## Versioning

We use [SemVer](http://semver.org/) for versioning. For the versions available, see the [tags on this repository](https://github.com/davejkane/riv/tags).

## Authors

* **[Dave Kane](https://github.com/Davejkane)** - *Initial Implementation*
* **[Alex Gurganus](https://github.com/gurgalex)** - *Implementing core features since 0.1.0*
* **[Nick Hackman](https://github.com/nickhackman)** - *Implementing core features since 0.2.0*

See also the list of [contributors](https://github.com/davejkane/riv/contributors) who participated in this project.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details

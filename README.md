# **Riv** the **R**ust **I**mage **V**iewer

Why riv? This project was born out of a frustration with image viewers on Mac. 
Generally the options are:-

* iPhoto - Way too heavy for just viewing images
* Preview - Clunky and really only good for viewing one image at a time
* Others that require a GUI folder browser

Riv on the other hand runs from the command line, and accepts a glob. For example:-

```$ riv **/*.jpg```

## Manual

Start riv with 

```$ riv```. 

As an optional second parameter you can add a glob, space separated filepaths or even a single filepath.

```$ riv **/*.png```

Without any second parameter, riv will look for all images in the current directory.

Set a destination folder for moving files with the `f` flag. The folder will be created if it doesn't exist.

```$ riv -f ~/saved_images```

### Controls


| Key              | Action                                                 |
|------------------|--------------------------------------------------------|
| Esc OR q         | Quit                                                   |
| Left Arrow OR k  | Previous Image                                         |
| Right Arrow OR j | Next Image                                             |
| PageUp OR w      | Forward 10% of images                                  |
| PageDown OR b    | Backward 10% of images                                 |
| Home OR g        | First Image                                            |
| End OR G         | Last Image                                             |
| m                | Move image to destination folder (default is ./keep)   |
| c                | Copy image to destination folder (default is ./keep)   |
| Delete OR d      | Delete image from it's location                        |
| t                | Toggle information bar                                 |
| f OR F11         | Toggle fullscreen mode                                 |
| h                | Toggle help box                                        |

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

See also the list of [contributors](https://github.com/davejkane/riv/contributors) who participated in this project.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details

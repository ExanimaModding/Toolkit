# Toolkit Mods

This folder contains example mods for this project.


## Dependencies

Make sure you have Rust installed with [Rustup](https://rustup.rs/)

Make sure you have Rust Nightly installed by running: `rustup toolchain install nightly`.

## Building

You need to get the `emf.dll.lib` file from the Toolkit. To do this:

1. In this project, open a terminal and run `cargo build -p emf`.

2. Next, run `cargo build -p godhand` to build the mod for godhand. You can replace the word godhand with a different mod of your choice

## Loading into the client

In each of the mods folders, there should be a `config.toml`, make sure this is filled out with the correct information.

In your Exanima game directory (`steamapps/common/Exanima`), make a folder called `mods`.

In there, make a folder for each mod you want to add, and place the corresponding `config.toml` inside it.

You can then copy the `.dll` for your mod (e.g. `godhand.dll`) into the corresponding mod folder.

An example layout should look like:

```
steamapps/common/Exanima/
	mods/
		GodHand/
			config.toml
			dev.megu.godhand.dll
		GodMode/
			config.toml
			dev.megu.godmode.dll
```

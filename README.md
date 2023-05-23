# bevy_snake

This app was made using Bevy and the Rust programming language. It is my first project using Tust and I learned a lot through the process.

The ultimate goal is for me to put it on the web by compiling the binary down to WebAssembly, which would be incredible !

The state of the game is pretty basic for now, but I may update the project one day to add a UI and some assets to make it look better :)

## Installation

### On desktop

Follow the installation steps for Rust and Cargo [here](https://www.rust-lang.org/tools/install).

Once it is setup, clone this repository, and run `cargo run`.
The executable should be launched.

### On the web

Relies on [TrunkRS](https://trunkrs.dev/).

Run the following commands to install `trunk` and the WASM binder:

```
cargo install trunk
cargo install --locked wasm-bindgen-cli
```

Run `trunk build` to build the HTML and SCSS files from the rust file.
Then run `trunk serve` to launch a live server.
You can now play in your browser !


## Clear the project

```
# Clears the target/ folder
cargo clean

# Clears the dist/ folder
trunk clean
```

## How to play ?

Just hit the arrow keys to move the snake and grow until you hit a wall or your own body !

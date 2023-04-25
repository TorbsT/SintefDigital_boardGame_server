# Sintef Digital Board Game

## Installation and running
1. Clone this repository at the desired location.
2. Install the Rust programming language by following [these](https://www.rust-lang.org/learn/get-started) steps.
3. Start the server in release mode (the fastest) using `cargo run --release`.
4. Alternatively, you can build the server using `cargo build --release` and then run the binary in <path_to_server>/target/release/. If you are using Ubuntu, you might need run `chmod u+x <path_to_server_binary>`.
5. On Ubuntu you can also start the server-binary as a service by following the steps [here](https://askubuntu.com/a/1314957).

## Installing and running the client server
1. Install apache2 on your computer and configure it. On Ubuntu it's recommended to follow [this](https://ubuntu.com/tutorials/install-and-configure-apache#1-overview) tutorial.
2. Clone the repository at a desired location.
4. Build the project for WebGL.
5. Copy the WebGL build files into the directory you made in step 1.

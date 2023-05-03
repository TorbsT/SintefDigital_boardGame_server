# Sintef Digital Board Game

## Introduction
This is the code repository for the back end server for Sintef Digital Board game. This is a collaboration project with 6 students taking the IT2901 - Informatics Project II course on NTNU.

## Technology stack
The code is written in the [Rust](https://www.rust-lang.org) language, and is using the [Actix Web](https://actix.rs) framework for communication between the client and server. Actix provides a HTTP server which is running on the HTTP/1.1 protocol, although the technology is capable of running HTTP/2.0.

Rust is blazingly fast and memory-efficient: with no runtime or garbage collector, it can power performance-critical services, run on embedded devices, and easily integrate with other languages. 
Rust’s rich type system and ownership model guarantee memory-safety and thread-safety — enabling you to eliminate many classes of bugs at compile-time.
Rust has great documentation, a friendly compiler with useful error messages, and top-notch tooling — an integrated package manager and build tool, smart multi-editor support with auto-completion and type inspections, an auto-formatter, and more (src: [https://www.rust-lang.org](https://www.rust-lang.org)).

[Actix Web](https://actix.rs) is a powerful, pragmatic, and extremely fast web framework for Rust. What makes Actix so powerful is that values do not have to be passed around using strings. Instead, everything has a type. Actix also provides features for HTTP/2, logging, etc (src: [https://actix.rs](https://actix.rs)).

## Installing and running the server locally
1. Clone this repository at the desired location.
2. Install the Rust programming language by following [these](https://www.rust-lang.org/learn/get-started) steps.
3. Start the server in release mode (the fastest) using `cargo run --release`.
4. Alternatively, you can build the server using `cargo build --release` and then run the binary in <path_to_server>/target/release/. If you are using Ubuntu, you might need run `chmod u+x <path_to_server_binary>`.
5. On Ubuntu you can also start the server-binary as a service by following the steps [here](https://askubuntu.com/a/1314957).

## Installing and running the server on a separate domain
### TODO: Add steps on getting the server up and running on the domain

## Installing and running the client server
1. Install apache2 on your computer and configure it. On Ubuntu it's recommended to follow [this](https://ubuntu.com/tutorials/install-and-configure-apache#1-overview) tutorial.
2. Clone the repository at a desired location.
4. Build the project for WebGL.
5. Copy the WebGL build files into the directory you made in step 1.

## ⚠ Disclaimer about caching
As the client server runs on WebGL, caching can become an issue as it runs an older snapshot of the game which does not have the latest features. In order to circumvent this problem, a reload requesting all files will be necessary after an update (typically achieved with the CTRL+R hotkey in your web browser).
# Rust Flow Based Programming

# Introduction

Most _system_ code has been written in C or C++ for many years. These tools have served quite well in the development of large complex systems. On the other hand, C and C++ can have significant issues with memory problems and dangling pointers. For this reason, the good folks at **Mozilla Research** design and built the Rust programming language. Rust is intended to be a language for highly concurrent and highly safe systems development. It is also designed to support &quot;_programming in the large_&quot; which is creating and maintaining boundaries that preserve large-system integrity. These design goals led to the development of Rust to have an emphasis on safety, control of memory layout, and concurrency.

## Rust Memory Safety

Rust is designed to be memory safe. That means that it does not permit null pointers, dangling pointers or data races in safe code. Rust does not use an automated garbage collection system. This means that the performance of Rust code is on par with C and C++.

## Ownership of Data in Rust

Rust has an ownership system where all values have a unique owner, and the scope of the value is the same as the scope of the owner. Values can be passed by immutable reference (&amp;T), by mutable reference (&amp;mut T) or b y value (T). There can either be multiple immutable references or **one** mutable reference which provides an implicit readers-writer lock. The Rust compiler using what is called the &quot;_Borrow Checker_&quot; enforces these rules at compile time.

## Rust Types and Polymorphism

Rust does **not** support inheritance like C++. It uses a system called traits which was inspired by the Haskell language. This trait system allows for ad hoc polymorphism. This means that a struct which is a datatype in Rust can choose to adhere to a trait which allows that struct to _be_ a subclass of that trait. This means that a Rust type can _inherit_ behavior but **not** fields. This is *no* &quot; _multiple inheritance_ in Rust, however a struct can choose to adhere to multiple traits.

# Flow Based Programming

Flow Based Programming (FBP) was invented by J. Paul Morrison in the early 1970s. It views applications not as a single, sequential process, but as a network of asynchronous processes communicating by means of streams of structured data. This means that FBP focuses on the application data and the transformations applied to it to produce the desired outputs. Typically the network is defined externally to the processes as a list of connections. One can read more about FBP here: [https://en.wikipedia.org/wiki/Flow-based\_programming](https://en.wikipedia.org/wiki/Flow-based_programming).


## Things that are Currently Missing

Only the bare bones of an FBP system are currently in place. There are many pieces that still need to be written. 

TODO: Flesh out this area


# Housekeeping

The following is a brief abstract that should help in getting started with Rust and this codebase.  This code has been built and tested on Ubuntu 20.04, Window 10 in PowerShell, Windows 10 in Ubuntu 20.04 WSL and on a Mac.  The first stem will be to install Rust on your platform.

## Install Rust on Ubuntu 20.04 or Windows 10 Ubuntu 20.04 WSL2

The following directions work for Ubuntu 20.04:

[https://www.osradar.com/install-rust-programming-language-ubuntu-debian/](https://www.osradar.com/install-rust-programming-language-ubuntu-debian/)

## Install Rust On Windows 10 for PowerShell

The following directions work for Windows 10.  I use PowerShell and that configuration has been tested.  It _should_ work for CMD as well but you mileage might vary.

[https://www.shadercat.com/setting-up-a-rust-development-environment-on-windows-10/#:~:text=%20Install%20Rust%20for%20Windows%20%201%20First%2C,or%20stable%2C%20and%20building%20against%20the...%20More%20](https://www.shadercat.com/setting-up-a-rust-development-environment-on-windows-10/#:~:text=%20Install%20Rust%20for%20Windows%20%201%20First%2C,or%20stable%2C%20and%20building%20against%20the...%20More%20)

## Install Rust on Mac OSX

For Mac users, you will need to have Homebrew installed.  Homebrew can be installed on the Mac using these directions:

[https://brew.sh/](https://brew.sh/)

Once Homebrew is installed follow these directions to install Rust:

[https://sourabhbajaj.com/mac-setup/Rust/](https://sourabhbajaj.com/mac-setup/Rust/)

# Building the Source

Rust has a package manager called cargo.  That is how the source has been built on all of the OSes ourlined.  Once Rust is installed on your OS, open a terminal window and change directory to this git enlistment.  Once there the following commands can be used:

*cargo clean*

This will do a _clean_ of the code so that a full build will be done.

*cargo build*

This will do a build of the source and create the library (crate in Rust terms).

*cargo test*

This will run the unit tests on the crate.  *NOTE* When running cargo test any println! that you may have put into the source will _*not*_ output to the terminal.  That is because cargo test captures the output.  To debug or _see_ any println! statements, cargo test must be run as follows *cargo test -- --nocapture*

*cargo doc*

This will build standard Rust documentation for this crate.

## Rust Resources

There are many different resouces for Rust on the net.  I personally found the _"Learning Rust With Entirely Too Many Linked Lists"_ [http://cglab.ca/~abeinges/blah/too-many-lists/book/README.html?ref=hackr.io] (http://cglab.ca/~abeinges/blah/too-many-lists/book/README.html?ref=hackr.io) to be quite helpful.  There is also the Rust Tutorial [https://aml3.github.io/RustTutorial/html/toc.html?ref=hackr.io](https://aml3.github.io/RustTutorial/html/toc.html?ref=hackr.io).  For a grouping of resources there is the following: [https://analyticsindiamag.com/top-10-free-resources-to-learn-rust-programming-language/](https://analyticsindiamag.com/top-10-free-resources-to-learn-rust-programming-language/)


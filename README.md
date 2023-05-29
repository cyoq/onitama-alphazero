# onitama-alphazero
A bachelor project for implementing AlphaZero algorithm for the game of Onitama

## Prerequisites
Please, take a look at [tch-rs](https://github.com/LaurentMazare/tch-rs) page for the instruction on how to install LibTorch on other platform. Instruction is specific for the Windows.

1. Download LibTorch 1.13.1 version for CPU or GPU. [Link to the CPU version](https://download.pytorch.org/libtorch/cpu/libtorch-cxx11-abi-shared-with-deps-1.13.1%2Bcpu.zip)
2. Unzip the archive
3. Set environment variables. Go to search -> Edit the system environments -> Environment variables. Set variable `LIBTORCH` with the path to the unzipped libtorch folder, for example: `D:\path\to\libtorch`

## GUI compilation
For the GUI compilation run the command:
```
cargo run --release --package onitama-gui --bin onitama-gui
```
**Note** that the compilation should be started from the root folder `onitama-alphazero`. Otherwise `assets` might not be found.

## Training 
To start the training, check [train.rs](alphazero-training/src/bin/train.rs) for training.
```
cd alphazero-training; cargo run --release --bin train
```

## TODO:
More information on a project will be pushed later
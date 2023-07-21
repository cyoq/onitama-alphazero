# onitama-alphazero
A bachelor project for implementing AlphaZero algorithm for the game of Onitama

## Prerequisites
Please, take a look at [tch-rs](https://github.com/LaurentMazare/tch-rs) page for the instruction on how to install LibTorch on other platform. Instruction is specific for the Windows.

1. Download LibTorch 1.13.1 version for GPU or CPU. Pre-built executables work only with the GPU version [Link to the GPU version](https://download.pytorch.org/libtorch/cu117/libtorch-win-shared-with-deps-1.13.1%2Bcu117.zip)
2. Unzip the archive
3. Set environment variables. Go to search -> Edit the system environments -> Environment variables. Set variable `LIBTORCH` with the path to the unzipped libtorch folder, for example: `D:\path\to\libtorch`
4. Append `D:\path\to\libtorch\lib` to the `PATH` variable

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

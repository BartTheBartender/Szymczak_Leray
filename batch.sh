#!/bin/bash 
#SBATCH --time=12:00:00
#SBATCH --cpus-per-task=144
#SBATCH --mem=1G 
#SBATCH --output=system_out
#SBATCH --error=system_err
#SBATCH --partition=kmo

cargo run --release

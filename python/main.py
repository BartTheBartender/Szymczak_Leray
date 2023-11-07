from launcher import launch
from plotter import plot

max_dim = 2
for base in range(2, 63):
    launch(base, max_dim)
    plot(base, max_dim, "szymczak-wide", full=True)

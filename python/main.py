from launcher import launch
from plotter import plot

for base in range(2, 63):
    launch(base, 1)
    plot(base, 1, "szymczak", full=True)

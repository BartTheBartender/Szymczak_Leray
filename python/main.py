import os
script_dir = os.path.dirname(os.path.abspath(__file__))
os.chdir(script_dir)
#-------------------------------------------------------------------
from launcher import launch
from plotter import plot
from hypotheses import check


for base in range(2, 31):
    launch(base = base, max_dim = 1)
    plot(base = base, max_dim = 1)


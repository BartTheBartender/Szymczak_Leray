import os
script_dir = os.path.dirname(os.path.abspath(__file__))
os.chdir(script_dir)
#-------------------------------------------------------------------
from launcher import launch
from plotter import plot
from hypotheses import find_special_representant

launch(base = 2, max_dim = 1)


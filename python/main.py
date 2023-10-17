import os
script_dir = os.path.dirname(os.path.abspath(__file__))
os.chdir(script_dir)
#-------------------------------------------------------------------
from launcher import launch, launch_all_isos
from plotter import plot, plot_all_isos
from hypotheses import find_special_representant


plot_all_isos(base = 2, max_dim = 1)



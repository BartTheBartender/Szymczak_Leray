import os

script_dir = os.path.dirname(os.path.abspath(__file__))
os.chdir(script_dir)
# -------------------------------------------------------------------
from launcher import launch, launch_all_isos
from plotter import plot, plot_all_isos
from hypotheses import find_special_representant


for base in range(2, 63):
    launch(base=base, max_dim=1)
    plot_all_isos(base=base, max_dim=1)

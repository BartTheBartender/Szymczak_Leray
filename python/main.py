from launcher import launch
from plotter import plot
#import benchmarker
#import math_utils
#import num_of_classes


for base in range(2, 31):
    launch(base = base, max_dim = 1)
    plot(base = base, max_dim = 1)


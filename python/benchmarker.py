import launcher
import re
import matplotlib.pyplot as plt

def benchmark(base, max_dim):
    time = {}
    recursion_parameter = 2
    while True:
        raw_output = launcher.run(base, max_dim, recursion_parameter, 'string').split('\n===\n')

        raw_preamble = raw_output[0]
        raw_timeinfo = raw_output[2]

        category_time = int(re.search(r'Category generated after: (\d+)',raw_timeinfo).group(1))
        iso_classes_time = int(re.search(r'Isomorphisms classes generated after: (\d+)', raw_timeinfo).group(1))

        time[recursion_parameter] = category_time + iso_classes_time

        
        num_of_endos = int(re.search(r'Number of endomorphisms: (\d+)', raw_preamble).group(1))

        if recursion_parameter == num_of_endos:
            break
        else:
            recursion_parameter += 1
    x = list(time.keys())
    y = list(time.values())

    
    plt.title(raw_preamble)
    plt.savefig(f'../results/benchmark/dim{max_dim}/Z{base}-dim-{max_dim}.jpg')

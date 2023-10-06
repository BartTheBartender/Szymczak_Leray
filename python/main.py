import parser
import launcher
import benchmarker

for base in range(2, 64):
    benchmarker.benchmark(base,1)

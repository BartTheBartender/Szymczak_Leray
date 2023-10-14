import re
import os
#-------------------------------------------------------------------

def find_special_representant(functor_name):

    if functor_name != 'Szymczak':
        raise ValueError('Wrong functor name!')

    output = []

    results = f'../results/{functor_name}/txt'
    for dim in os.listdir(results):

        dimpath = os.path.join(results, dim)
        if os.path.isdir(dimpath):
            
            for filename in os.listdir(dimpath):
                filepath = os.path.join(dimpath, filename)

                if re.search(r'Z\d+\-dim\-\d+', filepath) is not None:
                    file = open(filepath, 'r')
                    raw_preamble = file.read().split("===\n")[0]
                    file.close()
                    
                    map_hypoth = (re.search(r'Every class has a map: (.+)', raw_preamble).group(1) == 'true')
                    bij_hypoth = (re.search(r'Every class has a map: (.+)', raw_preamble).group(1) == 'true')
                    strong_bij_hypoth = (re.search(r'Every class has a map: (.+)', raw_preamble).group(1) == 'true')

                    base = int(re.search(r'Z(\d+)\-dim\-\d+', filepath).group(1))
                    dim = int(re.search(r'Z\d+\-dim\-(\d+)', filepath).group(1))
                    #print(map_hypoth, bij_hypoth, strong_bij_hypoth, base, dim)
                    output.append((base, dim, map_hypoth, bij_hypoth, strong_bij_hypoth))
                    
    output = sorted(output, key= lambda x: x[1])
    output = sorted(output, key= lambda x: x[0])

    string = ""
    for quintuple in output:
        base = quintuple[0]
        dim = quintuple[1]
        map_hypoth = quintuple[2]
        bij_hypoth = quintuple[3]
        strong_bij_hypoth = quintuple[4]
        
        string += f'Z{base}-Modules to dimension {dim}: {map_hypoth}, {bij_hypoth}, {strong_bij_hypoth}\n'

    print(string)

import launcher
import re

def num_of_classes(max_base):

    for base in range(2, max_base + 1):

        output = launcher.launch(base, 1, output_type = 'string')[0]
        print(output)

        #num_of_classes = int(re.search(r'Number of classes: (\d+)', output).group(1))
        #print(num_of_classes)

num_of_classes(4)


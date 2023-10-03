import numpy as np
import math
import matplotlib.pyplot as plt
from PIL import Image
import io
#-------------------------------------------------------------------
#-------------------------------------------------------------------

def parse(filepath):
    file = open(filepath, 'r')
    raw_data = file.read()
    (raw_preamble, raw_classes) = tuple(raw_data.split('===\n'))

    preamble = [parameter for parameter in raw_preamble.split('\n') if parameter.strip()]

    if len(preamble) != 3:
       raise ValueError('incorrect preamble!')

    functor_type = preamble[0]
    obj_type = preamble[1]
    endo_type = preamble[2]


    return [parse_class(raw_class, obj_type, endo_type) for raw_class in raw_classes.split('---\n') if raw_class.strip()]


def parse_class(raw_class, obj_type, endo_type):

    return [parse_class_fixed_obj(raw_class_fixed_obj, obj_type, endo_type) for raw_class_fixed_obj in raw_class.split('-\n') if raw_class_fixed_obj.strip()]

def parse_class_fixed_obj(raw_class_fixed_obj, obj_type, endo_type):
    (raw_obj, raw_endos) = tuple([string.strip() for string in raw_class_fixed_obj.split(':\n') if string.strip()])
    
    obj = parse_obj(raw_obj, obj_type)

    endos = [parse_endo(raw_endo, endo_type) for raw_endo in raw_endos.split('\n') if raw_endo.strip()]

    return (obj, endos)

def parse_obj(raw_obj, obj_type):
    if obj_type == 'Zn Module':
        return parse_Zn_module(raw_obj)

    raise ValueError('wrong object_type!')


def parse_endo(raw_endo, endo_type):
    if endo_type == 'RELATION':
        return parse_relation(raw_endo)
    
    raise ValueError('wrong endo_type!')

#-------------------------------------------------------------------
def parse_Zn_module(raw_Zn_module):
    if raw_Zn_module == '0':
        return( (['0'], [0]))
    
    torsion_coeffs = [int(string[1:]) for string in raw_Zn_module.split('x') if len(string) > 1]

    def generate_elements(torsion_coeffs, index, element):
        if index == len(torsion_coeffs):
            elements.append(''.join([str(x) for x in element]))

        else:
            for x in range(torsion_coeffs[index]):
                element[index] = x
                generate_elements(torsion_coeffs, index + 1, element)

    elements =[]
    generate_elements(torsion_coeffs, 0, [0]*len(torsion_coeffs))

    return (torsion_coeffs, elements)

def parse_relation(raw_relation):

    dim = int(math.sqrt(len(raw_relation)))
    if dim**2 != len(raw_relation):
        raise ValueError('raw_endo was not a nxn matrix!')

    return np.array([[int(raw_relation[j*dim+i]) for i in range(dim)] for j in range(dim)])

#-------------------------------------------------------------------
#-------------------------------------------------------------------
def plot_endo(endo, elements, color):
    fig, ax = plt.subplots(figsize=(8, 8))
    n = int(math.sqrt(endo.size))

    for i in range(n):
        for j in range(n):
            color_ = color if endo[i][j] == 1 else 'white'
            rect = plt.Rectangle((j, i), 1, 1, facecolor=color_, edgecolor='black', linewidth=2)
            ax.add_patch(rect)

    ax.set_xticks(np.arange(0.5, n, 1))
    ax.set_yticks(np.arange(0.5, n, 1))
    ax.set_xticklabels(elements, fontsize=40)
    ax.set_yticklabels(elements, fontsize=40)
    ax.tick_params(axis='both', which='both', length=0)  # Remove ticks

    
    # Set axis limits to ensure all rectangles are fully visible
    ax.set_xlim(0, n)
    ax.set_ylim(0, n)

    return plt.gcf()

def plot_class_fixed_obj(class_fixed_obj, color):

    obj = class_fixed_obj[0]
    endos = class_fixed_obj[1]
    torsion_coefficients = obj[0]
    elements = obj[1]
    
    row = 7

    plotted_endos_1d = [plot_endo(endo, elements, color) for endo in endos]
    plotted_endos = [plotted_endos_1d[i:i+row] for i in range(0, len(plotted_endos_1d), row)]

    col = 1

    fig, axs = plt.subplots(row, col, figsize = (10, 10))

    for i in range(5):
        for j in range(5):

            plotted_endo = plotted_endos[i][j]
            renderer = plotted_endo.canvas.get_renderer()
            plotted_endo.canvas.draw()
            plotted_endo = np.frombuffer(renderer.buffer_rgba(), dtype=np.uint8).reshape(int(renderer.get_canvas_width_height()[0]), int(renderer.get_canvas_width_height()[1]), -1)

            axs[i*col+j].imshow(plotted_endo)

    plt.tight_layout()
    plt.show()



#-------------------------------------------------------------------
#-------------------------------------------------------------------
result = parse('out')
test = result[0][0]
print(test)

plot_class_fixed_obj(test, 'blue')









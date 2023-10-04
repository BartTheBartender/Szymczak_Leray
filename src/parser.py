import numpy as np
import math
import matplotlib.pyplot as plt
from PIL import Image, ImageDraw, ImageFont 
import io
import cv
import PyPDF2
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


def plot_endo(endo, elements, color, endo_type):
    if endo_type == 'RELATION':
        return plot_adj_matrix(endo, elements, color)
    raise ValueError('wrong endo_type!')

def plot_obj(obj, obj_type):
    if obj_type == 'Zn Module':
        return plot_Zn_module(obj)
    raise ValueError('wrong object_type!')

def plot_class_fixed_obj(class_fixed_obj, color, obj_type, endo_type):
    obj = class_fixed_obj[0]
    endos = class_fixed_obj[1]
    torsion_coefficients = obj[0]
    elements = obj[1]

    row = 7
    plotted_endos_1d = [plot_endo(endo, elements, color, endo_type) for endo in endos]
    plotted_endos = [plotted_endos_1d[i:i+row] for i in range(0, len(plotted_endos_1d), row)]
    col = len(plotted_endos)

    width, height = plotted_endos_1d[0].size
    image_class_fixed_obj = Image.new('RGB', (row*width, (col+1)*height), 'white')
    
    for i in range(len(plotted_endos)):
        for j in range(len(plotted_endos[i])):
            image_class_fixed_obj.paste(plotted_endos[i][j], (j*width, (i+1)*height))

    image_obj = plot_obj(obj, obj_type)
    image_class_fixed_obj.paste(image_obj, (row*width//2, height // 3))
    image_class_fixed_obj.show()
    

def plot_class(class_, color, obj_type, endo_type):

    plotted_class = [plot_class_fixed_obj(class_fixed_obj, color, obj_type, endo_type) for class_fixed_obj in class_]

    for x in plotted_class:
        plt.show()

    #pdf_class = [PyPDF2.PdfReader(open(plotted_class_fixed_obj), 'rb') for plotted_class_fixed_object in plotted_class]


    #for pdf_class_fixed_obj in pdf_class:
        result.add(pdf_class)

    #return result

def plot(classes):

    pdf_classes = [PyPDF2.PdfReader(open(plot_class(class_), 'rb')) for class_ in classes]
    result = PyPDF2.PdfWriter()

    for pdf_class in pdf_classes:
        result.add(pdf_class)

    with open("result.pdf", "wb") as out_pdf:
        result.write(out_pdf)

#-------------------------------------------------------------------
def plot_Zn_module(obj):
    
    if obj[0] == 0:
        latex = '0'

    else:
        latex =''
        latex += '\mathbb{Z} \slash ' + str(obj[0][0])

        for i in range(1, len(obj[0])):
            latex += ' \oplus \mathbb{Z} \slash ' + str(obj[0][i])

    latex = '$' + latex + '$'

    fig, ax = plt.subplots(figsize=(10, 5))
    fig.text(0.5, 0.5, latex, size=75, ha='center', va='center')
    ax.set_axis_off()
    plt.tight_layout()

    buf = io.BytesIO()
    plt.gcf().savefig(buf, format='png')
    buf.seek(0)
    img = Image.open(buf)
    return img


def plot_adj_matrix(adj_matrix, elements, color):
    fig, ax = plt.subplots(figsize=(8, 8))
    n = int(math.sqrt(adj_matrix.size))

    for i in range(n):
        for j in range(n):
            color_ = color if adj_matrix[i][j] == 1 else 'white'
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

    buf = io.BytesIO()
    plt.gcf().savefig(buf, format='png')
    buf.seek(0)
    img = Image.open(buf)
    return img
#-------------------------------------------------------------------
#-------------------------------------------------------------------
parsed = parse('out')
test = parsed[0][0]
print(test)

plot_class_fixed_obj(test, 'red', 'Zn Module', 'RELATION')











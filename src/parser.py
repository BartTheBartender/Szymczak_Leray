import numpy as np
import math
import matplotlib.pyplot as plt
from PIL import Image, ImageDraw, ImageFont, ImageOps
from matplotlib.backends.backend_agg import FigureCanvasAgg
import io
import cv
from PyPDF2 import PdfReader, PdfWriter, PdfMerger, PageObject
from reportlab.pdfgen import canvas
from reportlab.lib.pagesizes import letter
from reportlab.lib.utils import ImageReader
import img2pdf
import random
import re
#-------------------------------------------------------------------
#-------------------------------------------------------------------

def parse(filepath):
    obj_type = 'Zn Module'
    endo_type = 'RELATION'
    file = open(filepath, 'r')

    (raw_preamble, raw_classes) = file.read().split('\n===\n')

    preamble = parse_preamble(raw_preamble)

    classes = [parse_class(raw_class, obj_type, endo_type) for raw_class in raw_classes.split('---\n') if raw_class.strip()]

    return (classes, preamble)


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

def parse_preamble(raw_preamble):
    raw_preamble = raw_preamble.split('\n')
    return raw_preamble



#-------------------------------------------------------------------
def parse_Zn_module(raw_Zn_module):
    if raw_Zn_module == '0':
        return( (['0'], [0]))
    
    torsion_coeffs = [int(string[1:]) for string in raw_Zn_module.split('x') if len(string) > 1]

    def generate_elements(torsion_coeffs, index, element):
        if index == len(torsion_coeffs):
            elements.append(' '.join([str(x) for x in element]))

        else:
            for x in range(torsion_coeffs[index]):
                element[index] = x
                generate_elements(torsion_coeffs, index + 1, element)

    elements =[]
    generate_elements(torsion_coeffs, 0, [0]*len(torsion_coeffs))

    return (torsion_coeffs, elements)

def parse_relation(raw_relation):

    def is_a_map(relation,dim):
        for j in range(dim):
            if sum([relation[i][j] for i in range(dim)]) != 1:
                return False
        return True

    dim = int(math.sqrt(len(raw_relation)))
    if dim**2 != len(raw_relation):
        raise ValueError('raw_endo was not a nxn matrix!')

    relation = np.array([[int(raw_relation[j*dim+i]) for i in range(dim)] for j in range(dim)])

    return (relation, is_a_map(relation,dim))

#-------------------------------------------------------------------
#-------------------------------------------------------------------


def plot_endo(endo, elements, color, endo_type):
    if endo_type == 'RELATION':
        return plot_relation(endo, elements, color)
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
    obj_width, obj_height = image_obj.size

    image_class_fixed_obj.paste(image_obj, (0, 0))
    image_class_fixed_obj = ImageOps.expand(image_class_fixed_obj, 10, fill ='black')
    image_class_fixed_obj = ImageOps.expand(image_class_fixed_obj, 70, fill ='white')

    buf = io.BytesIO()
    image_class_fixed_obj.save(buf, format='pdf')
    buf.seek(0)
    pdf_reader = PdfReader(buf)
    pdf_writer = PdfWriter()
    pdf_writer.add_page(pdf_reader.pages[0])
    return pdf_writer

def plot_class(class_, color, obj_type, endo_type):

    pdfs_class = [plot_class_fixed_obj(class_fixed_obj, color, obj_type, endo_type).pages[0] for class_fixed_obj in class_]

    
    pdf_writer = PdfWriter()

    for pdf in pdfs_class:
        pdf_writer.add_page(pdf)

    return pdf_writer

def plot_preamble(preamble, no_of_classes):

    base = preamble[0]
    dim = preamble[1]
    functor_name = preamble[2]
    obj_type = preamble[3]
    endo_type = preamble[4]
    map_found = preamble[5]

    output = f'Isomorphism classes of the Szymczak Functor\nCategory: LinRel(Z{base[-1]}-mod), max dimension is {dim[-1]}.\nThere are {no_of_classes} classes.'

    if map_found == 'EVERY CLASS HAS A MAP':
        output += 'Every class has a map.'
    else:
        output += 'A class without a map was found.'

    
    fig = plt.figure()
    plt.rc('text')
    plt.rc('font', family='serif')
    plt.text(0.5, 0.5, output, va='center', ha='center')
    plt.axis('off')
    plt.tight_layout()

    buf = io.BytesIO()
    fig.savefig(buf, format='pdf')
    return PdfReader(buf)

    


def plot(parsed_input, colors, obj_type, endo_type):
    
    classes = parsed_input[0]
    preamble = parsed_input[1]

    base = preamble[0][-1]
    max_dim = preamble[1][-1]

    if len(classes) > len(colors):
        raise ValueError('not enought colors!')

    pdf_writer = PdfWriter()
    pdf_writer.add_page(plot_preamble(preamble, len(classes)).pages[0])
    
    for i in range(len(classes)):
        pdf_writer_class = plot_class(classes[i], colors[i], obj_type, endo_type)

        for page in pdf_writer_class.pages:
            pdf_writer.add_page(page)


    with open(f"results/pdf/Z{base}-dim-{max_dim}.pdf", "wb") as output_pdf:
        pdf_writer.write(output_pdf)



#-------------------------------------------------------------------
def plot_Zn_module(obj):
    if obj[0][0] == 0:
        latex = '0'
    else:
        latex = '\mathbb{Z} \slash ' + str(obj[0][0])
        for i in range(1, len(obj[0])):
            latex += ' \oplus \mathbb{Z} \slash ' + str(obj[0][i])
    latex = '$' + latex + ':$'
    
    # Set a custom figure size (in inches)
    fig = plt.figure(figsize = (20,8))
    plt.rc('text', usetex=True)
    plt.rc('text.latex', preamble=r'\usepackage{amsfonts}')
    plt.rc('font', family='serif')
    plt.text(0.5, 0.5, latex, fontsize=160, va='center', ha='center')
    plt.axis('off')
    plt.tight_layout()

    canvas = FigureCanvasAgg(plt.gcf())
    canvas.draw()
    plt.close()
    img = Image.frombytes('RGB', canvas.get_width_height(), canvas.tostring_rgb())
    return img

def plot_relation(relation, elements, color):

    adj_matrix = relation[0]
    is_a_map = relation[1]

    fig, ax = plt.subplots(figsize=(8, 8))
    n = int(math.sqrt(adj_matrix.size))

    for i in range(n):
        for j in range(n):
            color_ = color[int(is_a_map)] if adj_matrix[i][j] == 1 else 'white'
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

    canvas = FigureCanvasAgg(fig)
    canvas.draw()
    plt.close()
    return Image.frombytes('RGB', canvas.get_width_height(), canvas.tostring_rgb())
#-------------------------------------------------------------------
#-------------------------------------------------------------------

colors = [
        ('lightgray', 'dimgray'),
        ('gold', 'darkgoldenrod'),
        ('lime', 'forestgreen'),
        ('salmon', 'red'),
        ('darkblue', 'blue'),
        ('darkolivegreen','yellowgreen'),
        ('lightseagreen', 'turquoise'),
        ('darkorange','bisque'),
        ('darkmagenta', 'orchid'),
        ('maroon','salmon'),
        ('lightgray', 'dimgray'),
        ('gold', 'darkgoldenrod'),
        ('lime', 'forestgreen'),
        ('salmon', 'red'),
        ('darkblue', 'blue'),
        ('darkolivegreen','yellowgreen'),
        ('lightseagreen', 'turquoise'),
        ('darkorange','bisque'),
        ('darkmagenta', 'orchid'),
        ('maroon','salmon'),
        ('lightgray', 'dimgray'),
        ('gold', 'darkgoldenrod'),
        ('lime', 'forestgreen'),
        ('salmon', 'red'),
        ('darkblue', 'blue'),
        ('darkolivegreen','yellowgreen'),
        ('lightseagreen', 'turquoise'),
        ('darkorange','bisque'),
        ('darkmagenta', 'orchid'),
        ('maroon','salmon'),
        ('lightgray', 'dimgray'),
        ('gold', 'darkgoldenrod'),
        ('lime', 'forestgreen'),
        ('salmon', 'red'),
        ('darkblue', 'blue'),
        ('darkolivegreen','yellowgreen'),
        ('lightseagreen', 'turquoise'),
        ('darkorange','bisque'),
        ('darkmagenta', 'orchid'),
        ('maroon','salmon'),
        ('lightgray', 'dimgray'),
        ('gold', 'darkgoldenrod'),
        ('lime', 'forestgreen'),
        ('salmon', 'red'),
        ('darkblue', 'blue'),
        ('darkolivegreen','yellowgreen'),
        ('lightseagreen', 'turquoise'),
        ('darkorange','bisque'),
        ('darkmagenta', 'orchid'),
        ('maroon','salmon'),
        ('lightgray', 'dimgray'),
        ('gold', 'darkgoldenrod'),
        ('lime', 'forestgreen'),
        ('salmon', 'red'),
        ('darkblue', 'blue'),
        ('darkolivegreen','yellowgreen'),
        ('lightseagreen', 'turquoise'),
        ('darkorange','bisque'),
        ('darkmagenta', 'orchid'),
        ('maroon','salmon'),
        ]
#-------------------------------------------------------------------
#-------------------------------------------------------------------
res = parse('out')[0][0][0]
print(res[0][1], res[1][0])

output = plot_endo(res[0][1], res[1][0], colors[3], 'RELATION')










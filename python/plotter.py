import re
import numpy as np

import matplotlib.pyplot as plt
from io import BytesIO
from PIL import Image, ImageOps
from pypdf import PdfReader, PdfWriter, PageObject
#-------------------------------------------------------------------
#-------------------------------------------------------------------
num_of_colors = 60
raw_colors = [
        ((102,255,178),(0,204,102),(0,102,51)),
        ((255,153,204),(255,0,127),(102,0,51)),
        ((153,255,255),(0,204,204),(0,102,102)),
        ((255,204,153),(255,128,0),(153,76,0)),
        ((102,102,255),(0,0,255),(0,0,102)),
        ((255,255,153),(255,255,51),(204,204,0)),
        ((229,204,255),(178,102,255),(76,0,153))
        ]
length = len(raw_colors)
raw_colors = [((rel[0]/255, rel[1]/255, rel[2]/255), (map_[0]/255, map_[1]/255, map_[2]/255), (bij[0]/255, bij[1]/255, bij[2]/255)) for (rel, map_, bij) in raw_colors]
colors = []
for i in range(num_of_colors):
    colors.append(raw_colors[i%length])
#-------------------------------------------------------------------
#-------------------------------------------------------------------
def parse(raw_text):

    raw_output = raw_text.split('\n===\n')
    raw_preamble = raw_output[0]
    raw_classes = raw_output[1]

    preamble = parse_preamble(raw_preamble)
    obj_type = preamble[1]
    endo_type = preamble[2]

    classes = [parse_class(raw_class, obj_type, endo_type) for raw_class in raw_classes.split('---\n') if raw_class.strip()]

    return (preamble, classes)


def parse_class(raw_class, obj_type, endo_type):

    return [parse_class_fix_obj(raw_class_fix_obj, obj_type, endo_type) for raw_class_fix_obj in raw_class.split('-\n') if raw_class_fix_obj.strip()]

def parse_class_fix_obj(raw_class_fix_obj, obj_type, endo_type):
    (raw_obj, raw_endos) = tuple([string.strip() for string in raw_class_fix_obj.split(':\n') if string.strip()])
    
    obj = parse_obj(raw_obj, obj_type)

    endos = [parse_endo(raw_endo, endo_type) for raw_endo in raw_endos.split('\n') if raw_endo.strip()]

    return (obj, endos)

def parse_obj(raw_obj, obj_type):
    if obj_type == 'Zn-Module':
        return parse_Zn_module(raw_obj)

    raise ValueError('wrong object_type!')


def parse_endo(raw_endo, endo_type):
    if endo_type == 'Relation':
        return parse_relation(raw_endo)
    
    raise ValueError('wrong endo_type!')

def parse_preamble(raw_preamble):
    
    functor_name = re.search(r'Functor name: (.+)', raw_preamble).group(1)
    obj_type = re.search(r'Object: (.+)', raw_preamble).group(1)
    endo_type = re.search(r'Endomorphism: (.+)', raw_preamble).group(1)
    num_of_endos = int(re.search(r'Number of endomorphisms: (.+)', raw_preamble).group(1))
    num_of_classes = int(re.search(r'Number of classes: (.+)', raw_preamble).group(1))
    map_hypoth = (re.search(r'Every class has a map: (.+)', raw_preamble).group(1) == 'true')
    bij_hypoth = (re.search(r'Every class has a bijection: (.+)', raw_preamble).group(1) == 'true')

    return [functor_name, obj_type, endo_type, num_of_endos, num_of_classes, map_hypoth, bij_hypoth]



#-------------------------------------------------------------------
def parse_Zn_module(raw_Zn_module):
    if raw_Zn_module == '0':
        return( ([0], ['0']))
    
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

    def is_a_map(relation):
        dim = relation.shape[0]
        for j in range(dim):
            if sum([relation[i][j] for i in range(dim)]) != 1:
                return False
        return True

    def is_a_bij(relation):
        dim = relation.shape[0]
        for i in range(dim):
            if sum([relation[i][j] for j in range(dim)]) != 1:
                return False
        for j in range(dim):
            if sum([relation[i][j] for i in range(dim)]) != 1:
                return False
        return True

    dim = int(len(raw_relation)**0.5)
    if dim**2 != len(raw_relation):
        raise ValueError('raw_endo was not a nxn matrix!')

    relation = np.array([int(entry) for entry in raw_relation]).reshape(dim, dim)
    return (relation, is_a_map(relation), is_a_bij(relation))

#-------------------------------------------------------------------
#-------------------------------------------------------------------
#-------------------------------------------------------------------
#-------------------------------------------------------------------
#-------------------------------------------------------------------


def plot_endo(endo, elements, color, endo_type):
    if endo_type == 'Relation':
        return plot_relation(endo, elements, color)
    raise ValueError('wrong endo_type!')

def plot_obj(obj, obj_type):
    if obj_type == 'Zn-Module':
        return plot_Zn_module(obj)
    raise ValueError('wrong object_type!')

def plot_class_fix_obj(class_fix_obj, color, obj_type, endo_type):
    obj = class_fix_obj[0]
    endos = class_fix_obj[1]
    torsion_coefficients = obj[0]
    elements = obj[1]

    row = 7
    plotted_endos = [plot_endo(endo, elements, color, endo_type) for endo in endos]

    plotted_endos = [plotted_endos[i:i+row] for i in range(0, len(plotted_endos), row)]

    col = len(plotted_endos)
    im = Image.open(plotted_endos[0][0])
    width, height = im.size

    buffer_obj = plot_obj(obj, obj_type)
    image_obj = Image.open(buffer_obj)
    
    image_endos = Image.new('RGB', (row*width, col*height), 'white')
    for i in range(len(plotted_endos)):
        for j in range(len(plotted_endos[i])):
            image_endos.paste(Image.open(plotted_endos[i][j]), (j*width, i*height))
    
    for i in range(len(plotted_endos)):
        for j in range(len(plotted_endos[i])):
            plotted_endos[i][j].close()

    image = Image.new('RGB', (row*width, (col+1)*height), 'white')
    image.paste(image_obj, (0,0))
    image.paste(image_endos, (0, height))
    image = ImageOps.expand(image, border=5, fill='black')
    image = ImageOps.expand(image, border=25, fill='white')

    buffer = BytesIO()
    image.save(buffer, format = 'JPEG')
    buffer.seek(0)

    im.close()
    buffer_obj.close()
    image_obj.close()
    image_endos.close()
    return buffer
    

def plot_class(class_, color, obj_type, endo_type):

    images_class = []
    for class_fix_obj in class_:
        buffer = plot_class_fix_obj(class_fix_obj, color, obj_type, endo_type)
        images_class.append(Image.open(buffer))

    width = images_class[0].width
    heights = [image.height for image in images_class]
    height = sum(heights)

    image = Image.new('RGB', (width, height), 'white')
    curr_height = 0
    for i in range(len(images_class)):
        image.paste(images_class[i], (0, curr_height))
        curr_height += heights[i]

    buffer = BytesIO()
    image.save(buffer, format = 'pdf')
    image.close()

    for image in images_class:
        image.close()

    return buffer

def plot_preamble(preamble):
    

    functor_name = preamble[0]
    obj_type = preamble[1]
    endo_type = preamble[2]
    num_of_endos = preamble[3]
    num_of_classes = preamble[4]
    map_hypoth = preamble[5]
    bij_hypoth = preamble[6]

    if obj_type == 'Zn-Module':
        pass

    output = f'Functor name: {functor_name}\nObject: {obj_type}\nEndomorphism: {endo_type}\nNumber of endomorphisms: {num_of_endos}\nNumber of classes: {num_of_classes}\nEvery class has a map: {map_hypoth}\nEvery class has a bijection: {bij_hypoth}'
    
    fig = plt.figure()
    plt.rc('text')
    plt.rc('font', family='serif')
    plt.text(0.5, 0.5, output, va='center', ha='center')
    plt.axis('off')
    plt.tight_layout()

    buffer = BytesIO()
    fig.savefig(buffer, format='pdf')
    plt.close()
    return buffer
    


def plot_classes(parsed_input, colors):
    
    preamble = parsed_input[0]
    classes = parsed_input[1]

    obj_type = preamble[1]
    endo_type = preamble[2]

    if len(classes) > len(colors):
        raise ValueError('not enought colors!')

    pdf_writer = PdfWriter()

    pdf_writer.add_page(PdfReader(plot_preamble(preamble)).pages[0])

    for i in range(len(classes)):

        buffer = plot_class(classes[i], colors[i], obj_type, endo_type)
        page = PdfReader(buffer).pages[0]

        pdf_writer.add_page(page)

    return pdf_writer
    

#-------------------------------------------------------------------
def plot_Zn_module(obj):
    latex = None
    if obj[0][0] == 0:
        latex = '0'
    else:
        latex = '\mathbb{Z} \slash ' + str(obj[0][0])
        for i in range(1, len(obj[0])):
            latex += ' \oplus \mathbb{Z} \slash ' + str(obj[0][i])

    latex = '$' + latex + ':$'
    
    fig = plt.figure(figsize = (20,8))
    plt.rc('text', usetex=True)
    plt.rc('text.latex', preamble=r'\usepackage{amsfonts}')
    plt.rc('font', family='serif')
    plt.text(0.5, 0.5, latex, fontsize=160, va='center', ha='center')
    plt.axis('off')
    plt.tight_layout()

    buffer = BytesIO()
    plt.savefig(buffer, format = 'jpg')
    plt.close()
    return buffer

def plot_relation(relation, elements, color):
    adj_matrix = relation[0]
    is_a_map = relation[1]
    is_a_bij = relation[2]

    fig, ax = plt.subplots(figsize=(8, 8))
    n = int((adj_matrix.size)**0.5)
    

    for i in range(n):
        for j in range(n):
            color_ = color[int(is_a_map) + int(is_a_bij)] if adj_matrix[i][j] == 1 else 'white'
            rect = plt.Rectangle((j, i), 1, 1, facecolor=color_, edgecolor='black', linewidth=1)
            ax.add_patch(rect)

    ax.set_xticks(np.arange(0.5, n, 1))
    ax.set_yticks(np.arange(0.5, n, 1))
    ax.set_xticklabels(elements, fontsize=15)
    ax.set_yticklabels(elements, fontsize=15)
    ax.tick_params(axis='both', which='both', length=0)

    ax.set_xlim(0, n)
    ax.set_ylim(0, n)


    buffer = BytesIO()
    plt.savefig(buffer, format = 'jpg')
    plt.close()
    return buffer

#-------------------------------------------------------------------
#-------------------------------------------------------------------
def plot(base, max_dim):
    input_path = f'../results/szymczak/txt/dim{max_dim}/Z{base}-dim-{max_dim}'
    output_path = f'../results/szymczak/pdf/dim{max_dim}/Z{base}-dim-{max_dim}.pdf'
    input_text = open(input_path, 'r').read()
    input_parsed = parse(input_text)
    output_writer = plot_classes(input_parsed, colors = colors)
    with open(output_path, 'wb') as output:
        output_writer.write(output_path)


'''
color = (20/255, 6/255, 182/255)
color = (color, color, color)
raw_text = open('out8', 'r').read()

text_to_be_plotted = parse(raw_text)
'''

'''
endo = text_to_be_plotted[1][3][0][1][0]
elements = text_to_be_plotted[1][3][0][0][1]
out = Image.open(plot_endo(endo, elements, color, 'Relation'))
out.show()
'''

'''
class_fix_obj = text_to_be_plotted[1][0][0]
#print(class_fix_obj)
out = plot_class_fix_obj(class_fix_obj, color, 'Zn-Module', 'Relation')
pdf_reader = PdfReader(out)
pdf_writer = PdfWriter()
pdf_writer.add_page(pdf_reader.pages[0])
out.close()
with open(f"test.pdf", "wb") as output_pdf:
    pdf_writer.write(output_pdf)
'''

'''
class_ = text_to_be_plotted[1][0]
#print(class_)
out = plot_class(class_, color, 'Zn-Module', 'Relation')
pdf_reader = PdfReader(out)
pdf_writer = PdfWriter()
pdf_writer.add_page(pdf_reader.pages[0])
out.close()
with open(f"test.pdf", "wb") as output_pdf:
    pdf_writer.write(output_pdf)
'''

'''
preamble = text_to_be_plotted[0]
#print(preamble)
out = plot_preamble(preamble)
pdf_reader = PdfReader(out)
pdf_writer = PdfWriter()
pdf_writer.add_page(pdf_reader.pages[0])
out.close()
with open(f"test.pdf", "wb") as output_pdf:
    pdf_writer.write(output_pdf)
'''

'''
out = plot_classes(text_to_be_plotted, colors)
with open(f"test.pdf", "wb") as output_pdf:
    out.write(output_pdf)
'''

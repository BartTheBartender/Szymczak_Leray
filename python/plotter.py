import regex as re
import numpy as np

import matplotlib.pyplot as plt
from io import BytesIO
from PIL import Image, ImageOps
from pypdf import PdfReader, PdfWriter, PageObject

# -------------------------------------------------------------------
# -------------------------------------------------------------------
num_of_colors = 60
raw_colors = [
    ((102, 255, 178), (0, 204, 102), (0, 102, 51)),
    ((255, 153, 204), (255, 0, 127), (102, 0, 51)),
    ((153, 255, 255), (0, 204, 204), (0, 102, 102)),
    ((255, 204, 153), (255, 128, 0), (153, 76, 0)),
    ((102, 102, 255), (0, 0, 255), (0, 0, 102)),
    ((255, 255, 153), (255, 255, 51), (204, 204, 0)),
    ((229, 204, 255), (178, 102, 255), (76, 0, 153)),
]
length = len(raw_colors)
raw_colors = [
    (
        (rel[0] / 255, rel[1] / 255, rel[2] / 255),
        (map_[0] / 255, map_[1] / 255, map_[2] / 255),
        (bij[0] / 255, bij[1] / 255, bij[2] / 255),
    )
    for (rel, map_, bij) in raw_colors
]
colors = []
for i in range(num_of_colors):
    colors.append(raw_colors[i % length])


# -------------------------------------------------------------------
# -------------------------------------------------------------------
def parse(raw_text):
    raw_output = raw_text.split("\n===\n")
    raw_preamble = raw_output[0]
    raw_classes = raw_output[1]

    preamble = parse_preamble(raw_preamble)
    obj_type = preamble[1]
    endo_type = preamble[2]

    classes = [
        parse_class(raw_class, obj_type, endo_type)
        for raw_class in raw_classes.split("---\n")
        if raw_class.strip()
    ]

    return (preamble, classes)


def parse_preamble(raw_preamble):
    functor_name = re.search(r"Functor name: (.+)", raw_preamble).group(1)
    obj_type = re.search(r"Object: (.+)", raw_preamble).group(1)
    endo_type = re.search(r"Morphism: (.+)", raw_preamble).group(1)
    num_of_endos = int(
        re.search(r"Number of endomorphisms: (.+)", raw_preamble).group(1)
    )
    num_of_classes = int(re.search(r"Number of classes: (.+)", raw_preamble).group(1))
    map_hypoth = (
        re.search(r"Every class has a map: (.+)", raw_preamble).group(1) == "true"
    )
    bij_hypoth = (
        re.search(r"Every class has a bijection: (.+)", raw_preamble).group(1) == "true"
    )
    strong_bij_hypoth = (
        re.search(r"Every class has exactly one bijection: (.+)", raw_preamble).group(1)
        == "true"
    )

    return [
        functor_name,
        obj_type,
        endo_type,
        num_of_endos,
        num_of_classes,
        map_hypoth,
        bij_hypoth,
        strong_bij_hypoth,
    ]


def parse_class(raw_class, obj_type, endo_type):
    return [
        parse_class_fix_obj(raw_class_fix_obj, obj_type, endo_type)
        for raw_class_fix_obj in raw_class.split("-\n")
        if raw_class_fix_obj.strip()
    ]


def parse_class_fix_obj(raw_class_fix_obj, obj_type, endo_type):
    (raw_obj, raw_endos) = tuple(
        [string.strip() for string in raw_class_fix_obj.split(":\n") if string.strip()]
    )

    obj = parse_obj(raw_obj, obj_type)

    endos = [
        parse_endo(raw_endo, endo_type)
        for raw_endo in raw_endos.split("\n")
        if raw_endo.strip()
    ]

    return (obj, endos)


def parse_obj(raw_obj, obj_type):
    if re.search(r"Z\d+\-Module", obj_type) is not None:
        return parse_Zn_module(raw_obj)

    raise ValueError("wrong object_type!")


def parse_endo(raw_endo, endo_type):
    if endo_type == "Relation":
        return parse_relation(raw_endo)

    raise ValueError("wrong endo_type!")


# -------------------------------------------------------------------
def parse_Zn_module(raw_Zn_module):
    if raw_Zn_module == "0":
        return ([0], ["0"])

    torsion_coeffs = [
        int(string[1:]) for string in raw_Zn_module.split("x") if len(string) > 1
    ]

    def generate_elements(torsion_coeffs, index, element):
        if index == len(torsion_coeffs):
            elements.append(" ".join([str(x) for x in element]))

        else:
            for x in range(torsion_coeffs[index]):
                element[index] = x
                generate_elements(torsion_coeffs, index + 1, element)

    elements = []
    generate_elements(torsion_coeffs, 0, [0] * len(torsion_coeffs))

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

    dim = int(len(raw_relation) ** 0.5)
    if dim**2 != len(raw_relation):
        raise ValueError("raw_endo was not a nxn matrix!")

    relation = np.array([int(entry) for entry in raw_relation]).reshape(dim, dim)
    return (relation, is_a_map(relation), is_a_bij(relation))


# -------------------------------------------------------------------
# -------------------------------------------------------------------
# -------------------------------------------------------------------
# -------------------------------------------------------------------
# -------------------------------------------------------------------


def plot_endo(endo, elements, color, endo_type):
    if endo_type == "Relation":
        return plot_relation(endo, elements, color)
    raise ValueError("wrong endo_type!")


def plot_obj(obj, obj_type):
    if re.search(r"Z\d+\-Module", obj_type) is not None:
        return plot_Zn_module(obj)
    raise ValueError("wrong object_type!")


def plot_class_fix_obj(class_fix_obj, color, obj_type, endo_type):
    obj = class_fix_obj[0]
    endos = class_fix_obj[1]
    torsion_coefficients = obj[0]
    elements = obj[1]

    row = 7
    plotted_endos = [plot_endo(endo, elements, color, endo_type) for endo in endos]

    plotted_endos = [
        plotted_endos[i : i + row] for i in range(0, len(plotted_endos), row)
    ]

    col = len(plotted_endos)
    im = Image.open(plotted_endos[0][0])
    width, height = im.size

    buffer_obj = plot_obj(obj, obj_type)
    image_obj = Image.open(buffer_obj)

    image_endos = Image.new("RGB", (row * width, col * height), "white")
    for i in range(len(plotted_endos)):
        for j in range(len(plotted_endos[i])):
            image_endos.paste(Image.open(plotted_endos[i][j]), (j * width, i * height))

    for i in range(len(plotted_endos)):
        for j in range(len(plotted_endos[i])):
            plotted_endos[i][j].close()

    image = Image.new("RGB", (row * width, (col + 1) * height), "white")
    image.paste(image_obj, (0, 0))
    image.paste(image_endos, (0, height))
    image = ImageOps.expand(image, border=5, fill="black")
    image = ImageOps.expand(image, border=25, fill="white")

    buffer = BytesIO()
    image.save(buffer, format="JPEG")
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

    image = Image.new("RGB", (width, height), "white")
    curr_height = 0
    for i in range(len(images_class)):
        image.paste(images_class[i], (0, curr_height))
        curr_height += heights[i]

    buffer = BytesIO()
    image.save(buffer, format="pdf")
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
    strong_bij_hypoth = preamble[7]

    output = f"Functor name: {functor_name}\nObject: {obj_type}\nEndomorphism: {endo_type}\nNumber of endomorphisms: {num_of_endos}\nNumber of classes: {num_of_classes}\nEvery class has a map: {map_hypoth}\nEvery class has a bijection: {bij_hypoth}\nEvery class has exactly one bijection: {strong_bij_hypoth}"

    fig = plt.figure()
    plt.rc("text")
    plt.rc("font", family="serif")
    plt.text(0.5, 0.5, output, va="center", ha="center")
    plt.axis("off")
    plt.tight_layout()

    buffer = BytesIO()
    fig.savefig(buffer, format="pdf")
    plt.close()
    return buffer


def plot_classes(parsed_input, colors):
    preamble = parsed_input[0]
    classes = parsed_input[1]

    obj_type = preamble[1]
    endo_type = preamble[2]

    if len(classes) > len(colors):
        raise ValueError("not enought colors!")

    pdf_writer = PdfWriter()

    pdf_writer.add_page(PdfReader(plot_preamble(preamble)).pages[0])

    for i in range(len(classes)):
        buffer = plot_class(classes[i], colors[i], obj_type, endo_type)
        page = PdfReader(buffer).pages[0]

        pdf_writer.add_page(page)

    return pdf_writer


# -------------------------------------------------------------------
def plot_Zn_module(obj):
    latex = None
    if obj[0][0] == 0:
        latex = "0"
    else:
        latex = "\mathbb{Z} \slash " + str(obj[0][0])
        for i in range(1, len(obj[0])):
            latex += " \oplus \mathbb{Z} \slash " + str(obj[0][i])

    latex = "$" + latex + ":$"

    fig = plt.figure(figsize=(20, 8))
    plt.rc("text", usetex=True)
    plt.rc("text.latex", preamble=r"\usepackage{amsfonts}")
    plt.rc("font", family="serif")
    plt.text(0.5, 0.5, latex, fontsize=160, va="center", ha="center")
    plt.axis("off")
    plt.tight_layout()

    buffer = BytesIO()
    plt.savefig(buffer, format="jpg")
    plt.close()
    return buffer


def plot_relation(relation, elements, color):
    adj_matrix = relation[0]
    is_a_map = relation[1]
    is_a_bij = relation[2]

    fig, ax = plt.subplots(figsize=(8, 8))
    n = int((adj_matrix.size) ** 0.5)

    for i in range(n):
        for j in range(n):
            color_ = (
                color[int(is_a_map) + int(is_a_bij)]
                if adj_matrix[i][j] == 1
                else "white"
            )
            rect = plt.Rectangle(
                (j, i), 1, 1, facecolor=color_, edgecolor="black", linewidth=1
            )
            ax.add_patch(rect)

    ax.set_xticks(np.arange(0.5, n, 1))
    ax.set_yticks(np.arange(0.5, n, 1))
    ax.set_xticklabels(elements, fontsize=15)
    ax.set_yticklabels(elements, fontsize=15)
    ax.tick_params(axis="both", which="both", length=0)

    ax.set_xlim(0, n)
    ax.set_ylim(0, n)

    buffer = BytesIO()
    plt.savefig(buffer, format="jpg")
    plt.close()
    return buffer


# -------------------------------------------------------------------
# -------------------------------------------------------------------
def plot(base, max_dim):
    input_path = f"../results/szymczak/txt/dim{max_dim}/Z{base}-dim-{max_dim}"
    output_path = f"../results/szymczak/pdf/dim{max_dim}/Z{base}-dim-{max_dim}.pdf"
    input_text = open(input_path, "r").read()
    input_parsed = parse(input_text)
    output_writer = plot_classes(input_parsed, colors=colors)
    with open(output_path, "wb") as output:
        output_writer.write(output_path)


# -------------------------------------------------------------------
# -------------------------------------------------------------------
# -------------------------------------------------------------------
# -------------------------------------------------------------------
# -------------------------------------------------------------------
# -------------------------------------------------------------------
# -------------------------------------------------------------------
# -------------------------------------------------------------------
# -------------------------------------------------------------------
# -------------------------------------------------------------------
# -------------------------------------------------------------------
# -------------------------------------------------------------------
# -------------------------------------------------------------------
# -------------------------------------------------------------------
# -------------------------------------------------------------------
# -------------------------------------------------------------------
# -------------------------------------------------------------------
# -------------------------------------------------------------------
def parse_all_isos(raw_text):
    raw_output = raw_text.split("\n===\n")
    raw_preamble = raw_output[0]
    raw_classes = raw_output[1]

    preamble = parse_preamble_all_isos(raw_preamble)
    obj_type = preamble[1]
    mor_type = preamble[2]

    classes = [
        parse_class_all_isos(raw_class, obj_type, mor_type)
        for raw_class in raw_classes.split("---\n")
        if raw_class.strip()
    ]

    return (preamble, classes)


def parse_preamble_all_isos(raw_preamble):
    functor_name = re.search(r"Functor name: (.+)", raw_preamble).group(1)
    obj_type = re.search(r"Object: (.+)", raw_preamble).group(1)
    mor_type = re.search(r"Morphism: (.+)", raw_preamble).group(1)
    num_of_classes = int(re.search(r"Number of classes: (.+)", raw_preamble).group(1))

    return [functor_name, obj_type, mor_type, num_of_classes]


def parse_class_all_isos(raw_class, obj_type, mor_type):
    return [
        parse_class_fix_endo(raw_class_fix_endo, obj_type, mor_type)
        for raw_class_fix_endo in raw_class.split("--\n")
        if raw_class_fix_endo.strip()
    ]


def parse_class_fix_endo(raw_class_fix_endo, obj_type, mor_type):
    return [
        parse_quadruple(raw_quadruple, obj_type, mor_type)
        for raw_quadruple in raw_class_fix_endo.split("\n")
        if raw_quadruple.strip()
    ]


def parse_quadruple(raw_quadruple, obj_type, mor_type):
    (raw_endo_bij, raw_maps) = raw_quadruple.split("--")

    (raw_endo_obj, raw_bij) = raw_endo_bij.split("-")
    (raw_endo_to_bij, raw_bij_to_endo) = raw_maps.split("-")

    (raw_endo_obj, raw_endo) = raw_endo_obj.split(":")
    (raw_bij_obj, raw_bij) = raw_bij.split(":")

    endo = parse_endo(raw_endo, mor_type)[0]
    bij = parse_endo(raw_bij, mor_type)[0]

    endo_obj = parse_obj(raw_endo_obj, obj_type)
    bij_obj = parse_obj(raw_bij_obj, obj_type)

    endo_to_bij = parse_mor(raw_endo_to_bij, endo_obj, bij_obj, mor_type)
    bij_to_endo = parse_mor(raw_bij_to_endo, bij_obj, endo_obj, mor_type)

    return (endo_obj, endo, bij_obj, bij, endo_to_bij, bij_to_endo)


def parse_mor(raw_mor, source, target, mor_type):
    if mor_type == "Relation":
        return parse_relation_mor(raw_mor, source, target)

    raise ValueError("wrong mor_type!")


# -------------------------------------------------------------------


def parse_relation_mor(raw_relation, source, target):
    return np.array([int(entry) for entry in raw_relation]).reshape(
        len(source[1]), len(target[1])
    )


# -------------------------------------------------------------------
# -------------------------------------------------------------------


def plot_classes_all_isos(parsed_input, colors):
    preamble = parsed_input[0]
    classes_all_isos = parsed_input[1]

    if len(colors) < len(classes_all_isos):
        raise ValueError("Not enough colors!")

    pdf_writer = PdfWriter()
    pdf_writer.add_page(plot_preamble_all_isos(preamble))

    mor_type = preamble[2]
    pages_classes = [
        plot_class_all_isos(classes_all_isos[i], colors[i], mor_type)
        for i in range(len(classes_all_isos))
    ]

    for pages_class in pages_classes:
        for page in pages_class:
            pdf_writer.add_page(page)

    return pdf_writer


def plot_preamble_all_isos(preamble):
    functor_name = preamble[0]
    obj_type = preamble[1]
    endo_type = preamble[2]
    num_of_classes = preamble[3]

    output = f"Functor name: {functor_name}\nObject: {obj_type}\nEndomorphism: {endo_type}\nNumber of classes: {num_of_classes}"

    fig = plt.figure()
    plt.rc("text")
    plt.rc("font", family="serif")
    plt.text(0.5, 0.5, output, va="center", ha="center")
    plt.axis("off")
    # plt.tight_layout()

    buffer = BytesIO()
    fig.savefig(
        buffer,
        format="pdf",
    )
    plt.close()

    return PdfReader(buffer).pages[0]


def plot_class_all_isos(class_all_isos, color, mor_type):
    pages_class = [
        plot_class_fix_endo(class_fix_endo, color, mor_type)
        for class_fix_endo in class_all_isos
    ]

    pdf_writer = PdfWriter()

    for page in pages_class:
        pdf_writer.add_page(page)

    return pdf_writer.pages


def plot_class_fix_endo(class_fix_endo, color, mor_type):
    images_quadruples = [
        plot_quadruple(quadruple, color, mor_type) for quadruple in class_fix_endo
    ]

    width, height = images_quadruples[0].size

    image = Image.new("RGB", (width, height * len(images_quadruples)), "white")

    for i in range(len(images_quadruples)):
        image.paste(images_quadruples[i], (0, i * height))
        images_quadruples[i].close()

    image = ImageOps.expand(image, border=2, fill="black")
    image = ImageOps.expand(image, border=10, fill="white")

    buffer = BytesIO()
    image.save(buffer, format="pdf", quality=50)
    image.close()
    return PdfReader(buffer).pages[0]


def plot_quadruple(quadruple, color, mor_type):
    endo_obj = quadruple[0]
    endo = quadruple[1]

    bij_obj = quadruple[2]
    bij = quadruple[3]

    endo_to_bij = quadruple[4]
    bij_to_endo = quadruple[5]

    width = len(endo_obj[1]) * 10

    buffer_endo = plot_mor(endo, endo_obj, endo_obj, width, color[1], mor_type)
    buffer_bij = plot_mor(bij, bij_obj, bij_obj, width, color[2], mor_type)
    buffer_endo_to_bij = plot_mor(
        endo_to_bij, endo_obj, bij_obj, width, color[0], mor_type
    )
    buffer_bij_to_endo = plot_mor(
        bij_to_endo, bij_obj, endo_obj, width, color[0], mor_type
    )

    image_endo = Image.open(buffer_endo[0])
    image_bij = Image.open(buffer_bij[0])
    image_endo_to_bij = Image.open(buffer_endo_to_bij[0])
    image_bij_to_endo = Image.open(buffer_bij_to_endo[0])

    width, height = image_endo.size

    sig_width, sig_height = image_endo.size

    image_endo_sig = Image.open(buffer_endo[1]).resize(
        (width, (sig_height * width) // sig_width)
    )
    buffer_endo[1].close()

    image_bij_sig = Image.open(buffer_bij[1]).resize(
        (width, (sig_height * width) // sig_width)
    )
    buffer_bij[1].close()

    image_endo_to_bij_sig = Image.open(buffer_endo_to_bij[1]).resize(
        (width, (sig_height * width) // sig_width)
    )
    buffer_endo_to_bij[1].close()

    image_bij_to_endo_sig = Image.open(buffer_bij_to_endo[1]).resize(
        (width, (sig_height * width) // sig_width)
    )
    buffer_bij_to_endo[1].close()

    sig_width, sig_height = image_endo_sig.size

    image_endo_sq = image_endo

    ###
    curr_width, curr_height = image_bij.size
    image_bij.resize((width, (curr_height * width) // curr_width))
    curr_width, curr_height = image_bij.size

    image_bij_sq = Image.new("RGB", (width, height), "white")
    image_bij_sq.paste(image_bij, ((width - curr_width) // 2, height - curr_height))
    image_bij.close()

    ###
    curr_width, curr_height = image_endo_to_bij.size
    image_endo_to_bij.resize((width, (curr_height * width) // curr_width))
    curr_width, curr_height = image_endo_to_bij.size

    image_endo_to_bij_sq = Image.new("RGB", (width, height), "white")
    image_endo_to_bij_sq.paste(
        image_endo_to_bij, ((width - curr_width) // 2, height - curr_height)
    )
    image_endo_to_bij.close()

    ###
    curr_width, curr_height = image_bij_to_endo.size
    image_bij_to_endo.resize((width, (curr_height * width) // curr_width))
    curr_width, curr_height = image_bij_to_endo.size

    image_bij_to_endo_sq = Image.new("RGB", (width, height), "white")
    image_bij_to_endo_sq.paste(
        image_bij_to_endo, ((width - curr_width) // 2, height - curr_height)
    )
    image_bij_to_endo.close()

    image = Image.new("RGB", (4 * width, height + sig_height), "white")

    image.paste(image_endo_sq, (0 * width, 0))
    image_endo_sq.close()

    image.paste(image_bij_sq, (1 * width, 0))
    image_bij_sq.close()

    image.paste(image_endo_to_bij_sq, (2 * width, 0))
    image_endo_to_bij_sq.close()

    image.paste(image_bij_to_endo_sq, (3 * width, 0))
    image_bij_to_endo_sq.close()

    image.paste(image_endo_sig, (0 * width, height))
    image_endo_sig.close()

    image.paste(image_bij_sig, (1 * width, height))
    image_bij_sig.close()

    image.paste(image_endo_to_bij_sig, (2 * width, height))
    image_endo_to_bij_sig.close()

    image.paste(image_bij_to_endo_sig, (3 * width, height))
    image_bij_to_endo_sig.close()

    return image


def plot_mor(mor, source, target, width, color, mor_type):
    if mor_type == "Relation":
        return plot_relation_mor(mor, source, target, width, color)

    raise ValueError("wrong mor_type!")


# -------------------------------------------------------------------
def plot_relation_mor(relation, source, target, width, color):
    height = 5

    source_len = len(source[1])
    target_len = len(target[1])

    fig, ax = plt.subplots(figsize=(source_len * 1.8, target_len * 1.8))

    for i in range(source_len):
        for j in range(target_len):
            color_ = color if relation[i][j] == 1 else "white"
            rect = plt.Rectangle(
                (i, j), 1, 1, facecolor=color_, edgecolor="black", linewidth=1
            )
            ax.add_patch(rect)

    ax.set_xticks(np.arange(0.5, source_len, 1))
    ax.set_yticks(np.arange(0.5, target_len, 1))
    ax.set_xticklabels(source[1], fontsize=13)
    ax.set_yticklabels(target[1], fontsize=13)
    ax.tick_params(axis="both", which="both", length=0)

    ax.set_xlim(0, source_len)
    ax.set_ylim(0, target_len)

    buffer_matrix = BytesIO()
    plt.savefig(buffer_matrix, format="jpg")
    plt.close()

    source_latex = None
    if source[0][0] == 0:
        source_latex = "0"
    else:
        source_latex = "\mathbb{Z} \slash " + str(source[0][0])
        for i in range(1, len(source[0])):
            source_latex += " \oplus \mathbb{Z} \slash " + str(source[0][i])

    target_latex = None
    if target[0][0] == 0:
        target_latex = "0"
    else:
        target_latex = "\mathbb{Z} \slash " + str(target[0][0])
        for i in range(1, len(target[0])):
            target_latex += " \oplus \mathbb{Z} \slash " + str(target[0][i])

    latex = "$" + source_latex + r" \rightarrow " + target_latex + "$"

    fig = plt.figure(figsize=(width, height))
    plt.rc("text", usetex=True)
    plt.rc("text.latex", preamble=r"\usepackage{amsfonts}")
    plt.rc("font", family="serif")
    plt.text(0.5, 0.5, latex, fontsize=140, va="center", ha="center")
    plt.axis("off")
    plt.tight_layout()

    buffer_sig = BytesIO()
    plt.savefig(buffer_sig, format="jpg")
    plt.close()

    return (buffer_matrix, buffer_sig)


# -------------------------------------------------------------------
# -------------------------------------------------------------------
def plot_all_isos(base, max_dim):
    print("Plotting started...")
    input_path = f"../results/szymczak_all_isos/txt/dim{max_dim}/Z{base}-dim-{max_dim}"
    output_path = (
        f"../results/szymczak_all_isos/pdf/dim{max_dim}/Z{base}-dim-{max_dim}.pdf"
    )
    input_text = open(input_path, "r").read()
    input_parsed = parse_all_isos(input_text)
    output_writer = plot_classes_all_isos(input_parsed, colors=colors)
    with open(output_path, "wb") as output:
        output_writer.write(output_path)
    print("Plotting finished succesfully")


# -------------------------------------------------------------------

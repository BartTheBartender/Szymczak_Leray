import regex as re
import numpy as np

import matplotlib.pyplot as plt
from io import BytesIO
from PIL import Image
from pypdf import PdfWriter, PdfReader, PageObject

import gc


# --------------------------------------------------------------------------------
class OutputFull:
    def __init__(self, raw_text):
        raw_text = raw_text.split("===\n")
        self.preamble = Preamble(raw_text[0])
        self.buffer = [
            IsoClassFull(
                raw_iso_class_full, self.preamble.obj_type, self.preamble.mor_type
            )
            for raw_iso_class_full in raw_text[1].split("---\n")
            if raw_iso_class_full.strip()
        ]

    def plot(self, colors):
        if len(colors) < len(self.buffer):
            raise ValueError("Not enough colors!")

        self.pdf_writer = PdfWriter()
        for index, iso_class_full in enumerate(self.buffer):
            iso_class_full.plot(colors[index])
            for page in iso_class_full.pages:
                self.pdf_writer.add_page(page)

            gc.collect()


# --------------------------------------------------------------------------------
class Preamble:
    def __init__(self, raw_preamble):
        self.functor = re.search(r"Functor name: (.+)", raw_preamble).group(1)
        self.obj_type = re.search(r"Object: (.+)", raw_preamble).group(1)
        self.mor_type = re.search(r"Morphism: (.+)", raw_preamble).group(1)
        self.nof_classes = int(
            re.search(r"Number of classes: (.+)", raw_preamble).group(1)
        )


# --------------------------------------------------------------------------------
class IsoClassFull:
    def __init__(self, raw_iso_class_full, obj_type, mor_type):
        self.buffer = [
            IsoClassFixEndo(raw_iso_class_fix_endo, obj_type, mor_type)
            for raw_iso_class_fix_endo in raw_iso_class_full.split("--\n")
            if raw_iso_class_fix_endo.strip()
        ]

        """
        self.buffer = [
            class_fix_endo
            for class_fix_endo in self.buffer
            if class_fix_endo.endo.matrix.shape == class_fix_endo.spec.matrix.shape
        ]

        if len(self.buffer) == 0:
            raise ValueError("The hypothesis is false or there is a mistake")

       """
        self.buffer = [
            class_fix_endo
            for class_fix_endo in self.buffer
            if not class_fix_endo.endo.is_a_bijection()
        ]

    def plot(self, colors):
        self.pages = []
        for iso_class_fix_endo in self.buffer:
            iso_class_fix_endo.plot(colors)
            pdf_reader = PdfReader(iso_class_fix_endo.buffer, strict=True)
            self.pages.append(pdf_reader.pages[0])
            # iso_class_fix_endo.buffer.close()

        gc.collect()


# --------------------------------------------------------------------------------
class IsoClassFixEndo:
    def __init__(self, raw_iso_class_fix_endo, obj_type, mor_type):
        raw_iso_class_fix_endo = raw_iso_class_fix_endo.split("#\n")
        (raw_endo, raw_spec) = raw_iso_class_fix_endo[0].split("--")

        if mor_type == "Relation":
            self.endo = Relation(raw_endo, obj_type)
            self.spec = Relation(raw_spec, obj_type)

            isos = []
            for raw_iso_pair in raw_iso_class_fix_endo[1].splitlines():
                (raw_morphs, raw_pows) = tuple(raw_iso_pair.split("---"))

                (raw_phi, raw_psi) = raw_morphs.split("--")
                phi = Relation(raw_phi, obj_type)
                psi = Relation(raw_psi, obj_type)

                (raw_pows_alpha, raw_pows_beta) = tuple(raw_pows.split("--"))
                pows_alpha = tuple(int(x) for x in raw_pows_alpha.split("-"))
                pows_beta = tuple(int(x) for x in raw_pows_beta.split("-"))

                isos.append(((phi, psi), pows_alpha, pows_beta))

            self.isos = isos

            """
            self.isos = [
                iso_pair
                for iso_pair in self.isos
                if iso_pair[0][0].is_a_partial_map()
                and iso_pair[0][1].inverse_is_a_partial_map()
            ]

            if len(self.isos) == 0:
                raise ValueError("The hypothesis is false or there is a mistake")
            """

        else:
            raise TypeError("Unknown morphism type!")

    def plot(self, colors):
        self.endo.plot(colors[1], r"\alpha")
        self.spec.plot(colors[2], r"\beta")

        alpha = Image.open(self.endo.matrix)
        beta = Image.open(self.spec.matrix)

        alpha_sig = Image.open(self.endo.sig)
        beta_sig = Image.open(self.spec.sig)

        alpha_width, alpha_height = alpha.size
        beta_width, beta_height = beta.size
        alpha_sig_width, alpha_sig_height = alpha_sig.size
        beta_sig_width, beta_sig_height = beta_sig.size

        # assume alpha acts on the biggest space
        width = alpha_width
        height = alpha_height
        sig_width = alpha_sig_width
        sig_height = alpha_sig_height

        nof_pairs = len(self.isos)

        image = Image.new(
            "RGB",
            (int(alpha_width * 5), (alpha_height + alpha_sig_height) * nof_pairs),
            "white",
        )

        for index, iso_pair in enumerate(self.isos):
            phi = iso_pair[0][0]
            psi = iso_pair[0][1]

            descr = IsoClassFixEndo.plot_description(iso_pair[1], iso_pair[2])
            descr_im = Image.open(descr)

            phi.plot(colors[0], r"\varphi")
            psi.plot(colors[0], r"\psi")

            phi_im = Image.open(phi.matrix)
            phi_sig = Image.open(phi.sig)
            phi_width, phi_height = phi_im.size
            phi_sig_width, phi_sig_height = phi_sig.size

            psi_im = Image.open(psi.matrix)
            psi_sig = Image.open(psi.sig)
            psi_width, psi_height = psi_im.size
            psi_sig_width, psi_sig_height = psi_sig.size

            descr_width, descr_height = descr_im.size

            # alpha
            image.paste(
                alpha,
                (
                    0 * width + (width - alpha_width) // 2,
                    index * (height + sig_height) + (height - alpha_height),
                ),
            )

            image.paste(
                alpha_sig,
                (
                    0 * width + (sig_width - alpha_sig_width) // 2,
                    index * (height + sig_height) + height,
                ),
            )

            # beta
            image.paste(
                beta,
                (
                    1 * width + (width - beta_width) // 2,
                    index * (height + sig_height) + (height - beta_height),
                ),
            )

            image.paste(
                beta_sig,
                (
                    1 * width + (sig_width - beta_sig_width) // 2,
                    index * (height + sig_height) + height,
                ),
            )

            # phi
            image.paste(
                phi_im,
                (
                    2 * width + (width - phi_width) // 2,
                    index * (height + sig_height) + (height - phi_height),
                ),
            )

            image.paste(
                phi_sig,
                (
                    2 * width + (sig_width - phi_sig_width) // 2,
                    index * (height + sig_height) + height,
                ),
            )

            # psi
            image.paste(
                psi_im,
                (
                    3 * width + (width - psi_width) // 2,
                    index * (height + sig_height) + (height - psi_height),
                ),
            )

            image.paste(
                psi_sig,
                (
                    3 * width + (sig_width - psi_sig_width) // 2,
                    index * (height + sig_height) + height,
                ),
            )

            # description
            image.paste(
                descr_im,
                (
                    4 * width + (sig_width - descr_width) // 2,
                    index * (height + sig_height) + height,
                ),
            )

            phi_im.close()
            phi_sig.close()
            phi.matrix.close()
            phi.sig.close()

            psi_im.close()
            psi_sig.close()
            psi.matrix.close()
            psi.sig.close()

            descr_im.close()
            descr.close()

        alpha.close()
        alpha_sig.close()
        beta.close()
        beta_sig.close()
        self.endo.matrix.close()
        self.endo.sig.close()
        self.spec.matrix.close()
        self.spec.sig.close()

        self.buffer = BytesIO()
        image.save(self.buffer, format="pdf")
        gc.collect()

    @staticmethod
    def plot_description(pows_alpha, pows_beta):
        fig = plt.figure(figsize=(5, 2))
        plt.rc("text", usetex=True)
        plt.rc("text.latex", preamble=r"\usepackage{amsfonts} \usepackage{amsmath}")
        plt.rc("font", family="serif")

        latex = (
            "$"
            + r"\psi \circ \varphi \circ \alpha^"
            + str(pows_alpha[0])
            + r" = \alpha^"
            + str(pows_alpha[1])
            + " $"
        )
        plt.text(0.5, 0.5, latex, fontsize=40, va="bottom", ha="center")

        latex = (
            "$"
            + r"\varphi \circ \psi \circ \beta^"
            + str(pows_beta[0])
            + r" = \beta^"
            + str(pows_beta[1])
            + " $"
        )
        plt.text(0.5, 0.5, latex, fontsize=40, va="top", ha="center")

        plt.axis("off")
        plt.tight_layout()

        buffer = BytesIO()
        plt.savefig(buffer, format="jpg")
        plt.close()
        return buffer


# --------------------------------------------------------------------------------
class Mor:
    def plot(self, color, name, special_coloring=False):
        pass

    def is_a_partial_map(self):
        pass

    def inverse_is_a_partial_map(self):
        pass

    def is_a_map(self):
        pass

    def is_a_matching(self):
        pass

    def is_a_bijection(self):
        pass


# --------------------------------------------------------------------------------
class Relation(Mor):
    def __init__(self, raw_relation, obj_type):
        raw_relation = raw_relation.split("-")

        if re.search(r"^Z[n\d]+-Module$", obj_type):
            self.source = ZnModule(raw_relation[0])
            self.target = ZnModule(raw_relation[1])

            if len(self.source.elements) * len(self.target.elements) != len(
                raw_relation[2]
            ):
                raise ValueError("The string cannot be parsed into a proper matrix!")
            self.matrix = np.array(
                [int(entry) for entry in list(raw_relation[2])]
            ).reshape(len(self.source.elements), len(self.target.elements))

            self.orbit_len = int(raw_relation[3]) if len(raw_relation) == 4 else None
        else:
            raise TypeError("Unknown object type!")

    def is_a_map(self):
        num_rows, num_cols = self.matrix.shape

        for i in range(num_rows):
            if sum(self.matrix[i][j] for j in range(num_cols)) != 1:
                return False

        return True

    def is_a_partial_map(self):
        num_rows, num_cols = self.matrix.shape

        for i in range(num_rows):
            if sum(self.matrix[i][j] for j in range(num_cols)) > 1:
                return False

        return True

    def inverse_is_a_partial_map(self):
        num_rows, num_cols = self.matrix.shape

        for j in range(num_cols):
            if sum(self.matrix[i][j] for i in range(num_rows)) > 1:
                return False

        return True

    def is_a_matching(self):
        num_rows, num_cols = self.matrix.shape

        for i in range(num_rows):
            if sum(self.matrix[i][j] for j in range(num_cols)) > 1:
                return False

        for j in range(num_cols):
            if sum(self.matrix[i][j] for i in range(num_rows)) > 1:
                return False

        return True

    def is_a_bijection(self):
        num_rows, num_cols = self.matrix.shape

        for i in range(num_rows):
            if sum(self.matrix[i][j] for j in range(num_cols)) != 1:
                return False

        for j in range(num_cols):
            if sum(self.matrix[i][j] for i in range(num_rows)) != 1:
                return False

        return True

    def plot(
        self,
        color,
        name,
        special_coloring=False,
    ):  # this function is overloaded, the "color" argument is a color or a triple of colors depending on usage (indicated by the "special_coloring" variable
        if isinstance(self.source, Obj) and isinstance(self.target, Obj):
            source_len = len(self.source.elements)
            target_len = len(self.target.elements)

            fig, ax = plt.subplots(figsize=(source_len * 1.8, target_len * 1.8))

            if special_coloring:
                pass  # todo
            else:
                for i in range(source_len):
                    for j in range(target_len):
                        color_ = color if self.matrix[i][j] else "white"
                        rect = plt.Rectangle(
                            (i, j),
                            1,
                            1,
                            facecolor=color_,
                            edgecolor="black",
                            linewidth=1,
                        )
                        ax.add_patch(rect)

            ax.set_xticks(np.arange(0.5, source_len, 1))
            ax.set_yticks(np.arange(0.5, target_len, 1))
            ax.set_xticklabels(self.source.elements, fontsize=13)
            ax.set_yticklabels(self.target.elements, fontsize=13)
            ax.tick_params(axis="both", which="both", length=0)

            ax.set_xlim(0, source_len)
            ax.set_ylim(0, target_len)
            # plt.tight_layout()

            height = 2
            width = max(fig.get_figwidth(), fig.get_figheight())

            buffer_matrix = BytesIO()
            plt.savefig(buffer_matrix, format="jpg")
            plt.close()

            source_latex = (
                "0"
                if len(self.source.tc) == 0
                else " \oplus ".join(
                    "\mathbb{Z} \slash" + str(x) for x in self.source.tc
                )
            )

            target_latex = (
                "0"
                if len(self.target.tc) == 0
                else " \oplus ".join(
                    "\mathbb{Z} \slash" + str(x) for x in self.target.tc
                )
            )

            latex = (
                "$"
                + source_latex
                + f" \overset{name} \longrightarrow "
                + target_latex
                + "$"
            )

            fig = plt.figure(figsize=(width, height))
            plt.rc("text", usetex=True)
            plt.rc("text.latex", preamble=r"\usepackage{amsfonts} \usepackage{amsmath}")
            plt.rc("font", family="serif")
            plt.text(0.5, 0.5, latex, fontsize=40, va="bottom", ha="center")

            if self.orbit_len is not None:
                latex = (
                    "$" + " ,".join(f"{name}^{i}" for i in range(self.orbit_len)) + "$"
                )
                plt.text(0.5, 0.5, latex, fontsize=30, va="top", ha="center")

            plt.axis("off")
            # plt.tight_layout()

            buffer_sig = BytesIO()
            plt.savefig(buffer_sig, format="jpg")
            plt.close()

            self.matrix = buffer_matrix
            self.sig = buffer_sig

        else:
            raise TypeError("Unknown object type!")


# --------------------------------------------------------------------------------
class Obj:
    pass


# --------------------------------------------------------------------------------
class ZnModule(Obj):
    @staticmethod
    def elements(tc):
        def elements(tc, buffer, element, index):
            if index == len(tc):
                buffer.append(" ".join(str(i) for i in element))
            else:
                for i in range(tc[index]):
                    element[index] = i
                    elements(tc, buffer, element, index + 1)

        buffer = []
        element = [0] * len(tc)
        elements(tc, buffer, element, 0)
        return buffer

    def __init__(self, raw_zn_module):
        if raw_zn_module == "0":
            self.tc = ()  # i think this representation is better than "(0,)"
            self.elements = ["0"]
        else:
            self.tc = tuple(
                [
                    int(raw_tc[1:])
                    for raw_tc in raw_zn_module.split("x")
                    if raw_tc.strip()
                ]
            )
            self.elements = ZnModule.elements(self.tc)


# --------------------------------------------------------------------------------
def plot(base, max_dim, functor_name, full):
    num_of_colors = 60
    colors = [
        ((102, 255, 178), (0, 204, 102), (0, 102, 51)),
        ((255, 153, 204), (255, 0, 127), (102, 0, 51)),
        ((153, 255, 255), (0, 204, 204), (0, 102, 102)),
        ((255, 204, 153), (255, 128, 0), (153, 76, 0)),
        ((102, 102, 255), (0, 0, 255), (0, 0, 102)),
        ((255, 255, 153), (255, 255, 51), (204, 204, 0)),
        ((229, 204, 255), (178, 102, 255), (76, 0, 153)),
    ]

    raw_colors = [
        tuple(tuple(component / 255 for component in color) for color in row)
        for row in colors
    ]

    length = len(raw_colors)
    colors = []
    for i in range(num_of_colors):
        colors.append(raw_colors[i % length])

    in_filepath = (
        "results/"
        + functor_name
        + ("-full/" if full else "/")
        + f"txt/dim{max_dim}/Z{base}-dim-{max_dim}"
    )
    out_filepath = (
        "results/"
        + functor_name
        + ("-full/" if full else "/")
        + f"pdf/dim{max_dim}/Z{base}-dim-{max_dim}.pdf"
    )

    print("Plotting started...")
    output_full = OutputFull(open(in_filepath).read())
    print("Parsing finished!")
    try:
        output_full.plot(colors)
        print("Plotting finished!\n-----------------------------------")
        output_full.pdf_writer.write(out_filepath)
    except KeyboardInterrupt:
        print("Keyboard interrupt!\n-----------------------------------")
    except:
        print("File was too big!\n-----------------------------------")


# --------------------------------------------------------------------------------

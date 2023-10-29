import regex as re
import numpy as np


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


# --------------------------------------------------------------------------------
class IsoClassFixEndo:
    def __init__(self, raw_iso_class_fix_endo, obj_type, mor_type):
        raw_iso_class_fix_endo = raw_iso_class_fix_endo.split("#\n")
        (raw_endo, raw_spec) = raw_iso_class_fix_endo[0].split("--")

        if mor_type == "Relation":
            self.endo = Relation(raw_endo, obj_type)
            self.spec = Relation(raw_spec, obj_type)
        else:
            raise ValueError("Unknown morphism type!")

        # print(raw_iso_class_fix_endo)


# --------------------------------------------------------------------------------
class Mor:
    def plot(self, color):
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

            if len(raw_relation) == 4:
                self.orbit_len = int(raw_relation[3])

            print(self.matrix, "\n")

        else:
            raise ValueError("Unknown object type!")


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
            self.tc = ()  # i think this representation is better than "[0]"
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
raw_text = open("out", "r").read()
OutputFull(raw_text)

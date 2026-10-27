"""Export the active material of the selected object from Blender to a Kvas node material (.kmat).

Usage in Blender: open the Scripting workspace, load this file, select an object, press Run.
The file is written next to the .blend (or into your home folder), together with the images it uses.
Copy the .kmat and the images into your project's assets folder and drop the .kmat on a model.

Supported: Principled BSDF (base color, roughness, metallic, emission, alpha), Image Texture,
RGB, Value, Mix / MixRGB, Math (add, sub, mul, div, power, min, max, sine, abs), Noise Texture,
Checker Texture, Color Ramp (first/last stop), Hue/Saturation, Bright/Contrast, Invert, Fresnel,
Texture Coordinate. Other nodes become a neutral grey value and are listed in the console.
"""
import bpy
import json
import os
import shutil

OUT_NAMES = ["Base Color", "Roughness", "Metallic", "Emission Color", "Alpha"]


class Exporter:
    def __init__(self, out_dir):
        self.out_dir = out_dir
        self.nodes = []
        self.textures = ["", "", ""]
        self.next_id = 1
        self.cache = {}
        self.warnings = []

    def add(self, kind, params=None, n_inputs=0, pos=(0, 0)):
        node = {"id": self.next_id, "kind": kind, "pos": [float(pos[0]), float(pos[1])],
                "inputs": [{"link": None, "value": [0.0, 0.0, 0.0]} for _ in range(n_inputs)],
                "params": list(params or [])}
        self.next_id += 1
        self.nodes.append(node)
        return node

    def const(self, socket):
        v = getattr(socket, "default_value", 0.0)
        try:
            vals = list(v)
            return [float(vals[0]), float(vals[1]), float(vals[2])]
        except TypeError:
            return [float(v)] * 3

    def connect(self, node, index, socket):
        """Fills input `index` of `node` from a Blender socket (a link or its constant)."""
        if socket.is_linked:
            node["inputs"][index]["link"] = self.convert(socket.links[0].from_node)
        else:
            value = self.const(socket)
            node["inputs"][index]["value"] = [v if v != 0.0 else 1e-6 for v in value]

    def image_slot(self, image, normal=False):
        if image is None:
            return 0
        slot = 1 if normal else (0 if not self.textures[0] else 2)
        os.makedirs(os.path.join(self.out_dir, "textures"), exist_ok=True)
        src = bpy.path.abspath(image.filepath) if image.filepath else ""
        name = os.path.basename(src) or (image.name + ".png")
        dst = os.path.join(self.out_dir, "textures", name)
        if src and os.path.exists(src):
            shutil.copyfile(src, dst)
        else:
            image.save_render(dst)
        self.textures[slot] = "textures/" + name
        return slot

    def convert(self, bnode):
        if bnode.name in self.cache:
            return self.cache[bnode.name]
        t = bnode.type
        pos = (bnode.location.x, -bnode.location.y)
        if t == "TEX_IMAGE":
            node = self.add("Texture", [self.image_slot(bnode.image), 1.0], 1, pos)
        elif t == "RGB":
            c = bnode.outputs[0].default_value
            node = self.add("Color", [c[0], c[1], c[2]], 0, pos)
        elif t == "VALUE":
            node = self.add("Value", [bnode.outputs[0].default_value], 0, pos)
        elif t in ("MIX", "MIX_RGB"):
            node = self.add("Mix", [], 3, pos)
            self.connect(node, 2, bnode.inputs[0])
            self.connect(node, 0, bnode.inputs[1])
            self.connect(node, 1, bnode.inputs[2])
        elif t == "MATH":
            kinds = {"ADD": "Add", "SUBTRACT": "Subtract", "MULTIPLY": "Multiply", "DIVIDE": "Divide",
                     "POWER": "Power", "MINIMUM": "Min", "MAXIMUM": "Max", "SINE": "Sine", "ABSOLUTE": "Abs"}
            kind = kinds.get(bnode.operation)
            if kind is None:
                self.warnings.append("math operation " + bnode.operation)
                node = self.add("Value", [0.5], 0, pos)
            else:
                params = [1.0] if kind == "Sine" else []
                count = 1 if kind in ("Sine", "Abs") else 2
                node = self.add(kind, params, count, pos)
                for i in range(count):
                    self.connect(node, i, bnode.inputs[i])
        elif t == "TEX_NOISE":
            node = self.add("Noise", [float(bnode.inputs["Scale"].default_value)], 1, pos)
        elif t == "TEX_CHECKER":
            node = self.add("Checker", [float(bnode.inputs["Scale"].default_value)], 1, pos)
        elif t == "VALTORGB":
            ramp = bnode.color_ramp.elements
            node = self.add("Gradient", [], 3, pos)
            self.connect(node, 0, bnode.inputs[0])
            node["inputs"][1]["value"] = [float(v) for v in ramp[0].color[:3]]
            node["inputs"][2]["value"] = [float(v) for v in ramp[len(ramp) - 1].color[:3]]
        elif t == "HUE_SAT":
            node = self.add("Saturation", [float(bnode.inputs["Saturation"].default_value)], 1, pos)
            self.connect(node, 0, bnode.inputs["Color"])
        elif t == "BRIGHTCONTRAST":
            node = self.add("Contrast", [1.0 + float(bnode.inputs["Contrast"].default_value)], 1, pos)
            self.connect(node, 0, bnode.inputs["Color"])
        elif t == "INVERT":
            node = self.add("One minus", [], 1, pos)
            self.connect(node, 0, bnode.inputs["Color"])
        elif t == "FRESNEL":
            node = self.add("Fresnel", [], 1, pos)
            node["inputs"][0]["value"] = [3.0, 3.0, 3.0]
        elif t == "TEX_COORD":
            node = self.add("UV", [], 0, pos)
        elif t == "NORMAL_MAP":
            # The normal map is bound to texture slot 1; the material uses it through the object's normal map.
            if bnode.inputs["Color"].is_linked and bnode.inputs["Color"].links[0].from_node.type == "TEX_IMAGE":
                self.image_slot(bnode.inputs["Color"].links[0].from_node.image, normal=True)
            node = self.add("Value", [0.0], 0, pos)
        else:
            self.warnings.append("node type " + t)
            node = self.add("Value", [0.5], 0, pos)
        self.cache[bnode.name] = node["id"]
        return node["id"]


def export(path=None):
    obj = bpy.context.active_object
    if obj is None or not obj.active_material or not obj.active_material.use_nodes:
        raise RuntimeError("Select an object with a node-based material.")
    material = obj.active_material
    tree = material.node_tree
    bsdf = next((n for n in tree.nodes if n.type == "BSDF_PRINCIPLED"), None)
    if bsdf is None:
        raise RuntimeError("The material has no Principled BSDF node.")
    out_dir = os.path.dirname(bpy.data.filepath) or os.path.expanduser("~")
    ex = Exporter(out_dir)
    output = ex.add("Output", [], 6, (900, 100))
    for i, name in enumerate(OUT_NAMES):
        sock = bsdf.inputs.get(name) or bsdf.inputs.get("Emission")
        if sock is not None:
            ex.connect(output, i, sock)
    doc = {"nodes": ex.nodes, "textures": ex.textures, "next_id": ex.next_id}
    path = path or os.path.join(out_dir, material.name.replace(" ", "_") + ".kmat")
    with open(path, "w", encoding="utf-8") as f:
        json.dump(doc, f, indent=2)
    print("Kvas material written:", path)
    for w in sorted(set(ex.warnings)):
        print("  not supported, replaced by a grey value:", w)
    return path


if __name__ == "__main__":
    export()

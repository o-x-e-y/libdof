from libdof import Dof


with open("./example_dofs/minimal_valid.json") as f:
    dofstr = f.read()


print(Dof.parse(dofstr))
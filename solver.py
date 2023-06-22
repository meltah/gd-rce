import os
import sys
import z3

pre = os.path.getsize("../build/bootloader_pre.bin")
post_gmh = os.path.getsize("../build/bootloader_post_gmh.bin")
post_b64 = os.path.getsize("../build/bootloader_post_b64.bin")

gmh_b4 = z3.Int("gmh_b4")
gmh_8 = z3.Int("gmh_8")
gmh_1 = z3.Int("gmh_1")

gmh_value = gmh_b4 * 0xb4 + gmh_8 * 8 + gmh_1 - 8

b64_b4 = z3.Int("b64_b4")
b64_8 = z3.Int("b64_8")
b64_1 = z3.Int("b64_1")

b64_value = b64_b4 * 0xb4 + b64_8 * 8 + b64_1 - 8

nop = z3.Int("nop")

size = gmh_b4 * 4 + gmh_8 * 4 + gmh_1 * 4 + b64_b4 * 4 + b64_8 * 4 + b64_1 * 4 + nop * 4
after_adds = pre + size

gmh_name_target = after_adds + post_gmh
b64_input_target = after_adds + post_b64


solver = z3.Optimize()

solver.minimize(size)

solver.add(gmh_b4 >= 0)
solver.add(gmh_8 >= 0)
solver.add(gmh_1 >= 0)

solver.add(b64_b4 >= 0)
solver.add(b64_8 >= 0)
solver.add(b64_1 >= 0)

solver.add(nop >= 0)

solver.add(gmh_value == gmh_name_target)
solver.add(b64_value == b64_input_target)
solver.check()

m = solver.model()
print(f"GMH: [b4: {m[gmh_b4]}] [8: {m[gmh_8]}] [1: {m[gmh_1]}]", file=sys.stderr)
print(f"B64: [b4: {m[b64_b4]}] [8: {m[b64_8]}] [1: {m[b64_1]}]", file=sys.stderr)
print(f"nop: {m[nop]}", file=sys.stderr)
print(f"{m[gmh_b4]} {m[gmh_8]} {m[gmh_1]} {m[b64_b4]} {m[b64_8]} {m[b64_1]} {m[nop]}")

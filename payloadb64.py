import sys
import base64

# Gets the length of the payload, with no zero bytes

with open("../build/payload.bin", "rb") as f, open("../build/payload_b64.txt", "wb") as o:
	data = base64.urlsafe_b64encode(f.read())
	l = len(data)
	b = 0
	for i in range(0, 4):
		if l >> (i * 8) & 0xFF != 0:
			b = i + 1
	print(f"original length: {l}, occupying {b} bytes", file=sys.stderr)
	for i in range(0, 4):
		if l >> (i * 8) & 0xFF == 0:
			l += 1 << (i * 8)
			if i < b:
				print("warning: a byte inside the actual size has been incremented. this is not tested", file=sys.stderr)
			print(f"byte {i + 1} is zero, incrementing to 1. length is now {l & ((1 << (b*8)) - 1)}, stored {l}", file=sys.stderr)
	
	eff = l & ((1 << (b*8)) - 1)
	
	print(f"final length: {eff}, stored {l} (changed by {eff - len(data)}), occupying {b} bytes", file=sys.stderr)
	
	print(f"{l} {b}")
	
	data += bytes(eff - len(data))
	o.write(data)

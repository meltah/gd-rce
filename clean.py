import gzip
import base64
import re
import gd

db = gd.api.save.load()
levels = db.get_created_levels()
lvl = levels[0]

lvl_string = lvl.get_data()
lvl_string = re.sub('kA14,[^,]+?,', 'kA14,1~1,', lvl_string)
lvl.set_data(lvl_string)

db.dump_created_levels(levels)
db.dump()

import re
import time

start_time = time.time()

with open("nri.zig", "r") as f:
    content = f.read()

print("NRI aliasok tisztítása...")

# 1. @compileError sorok kiszedése (hogy leforduljon a fájl)
content = re.sub(r'.*@compileError\(.*\);\n', '', content)

# 2. Csak a nagybetűs Nri előtagot vágjuk le az aliasokról (pl. NriDevice -> Device)
# A kisbetűs függvénynevek (pl. nriGetInterface) érintetlenül maradnak
content = re.sub(r'\bNri([A-Z][A-Za-z0-9_]*)', r'\1', content)

# 3. Az NRI_ konstans előtagok levágása (pl. NRI_SUCCESS -> SUCCESS)
content = re.sub(r'\bNRI_([A-Za-z0-9_]*)', r'\1', content)

with open("nri.zig", "w") as f:
    f.write(content)

print(f"Kész! Feldolgozási idő: {time.time() - start_time:.4f} másodperc.")


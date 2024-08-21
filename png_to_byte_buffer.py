from PIL import Image
import numpy as np
import os

name = "linear_theta_polarizer"

for filename in os.listdir("./assets/optical_objects_pngs/"):
    print(f'converting {filename[:-4]} to bytes')
    with Image.open(f'./assets/optical_objects_pngs/{filename[:-4]}.png') as im:
        im = im.convert('RGBA')
        out_image = bytearray(np.asarray(im))

        newFile = open(f'./assets/optical_objects_bytes/{filename[:-4]}.bytes', "wb")
        newFile.write(out_image)

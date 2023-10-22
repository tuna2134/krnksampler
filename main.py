from krnksampler import resample


with open("sample-3s.mp3", "rb") as f:
    data = resample(f.read())

with open("music.mp3", "wb") as f:
    f.write(data)
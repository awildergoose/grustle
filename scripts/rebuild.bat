@echo off
python gen.py && python deps.py && python copies.py && python minify.py && python perversionlibs.py

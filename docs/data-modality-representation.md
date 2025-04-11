For each data modality the following needs to be defined:

- What device it works on, what operating system, requirements from the device
  Example:
  Screen logger:
  | availiable | device | operating system | requires |
  |-|-|-|-|
  | ✅ | computer | NixOS 24.11 | to screen |
  | ✅ | computer | Windows 11 | access to screen |
  | ❌ | computer | Windows 10 | access to screen |
  | ✅ | computer | MacOS 14 | access to screen |
  | ❌ | phone | android 14 | access to screen |

- What it's database schema(s) are and description
  Example:
  Screen logger has creates a table called screen

Screen
| Timestamp | filepath | Size | Monitor | Resolution | ... |
| --------- | -------- | ---- | ------- | ---------- | --- |

Example:
Desktop environment logger creates multiple tables, applications, current workspace, mouse, etc.

- What data type the logger logs (image, song, table, text, etc.)
- What processing transforms it likes to use and their parameters
  Example:
  Screen logger would like OCR transform and image embedding transform on the file in column filepath. For image embedding transform has a priority of 1 and OCR transform has a priority of 2. The OCR transform would like to use the tesseract engine with the language set to eng+deu. The image embedding transform would like to use the CLIP model with the text encoder...
  Audio logger would like text to speech logger. Overnight it would like to process the audio file with whisper-large but if the user wants to request unprocessed audio then it will use whisper-small

-

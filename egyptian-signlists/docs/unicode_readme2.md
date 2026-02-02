
## Get the latest NamesList

The file is stable, plain text, and intended for tooling. Unicode guarantees 
backward compatibility of codepoints. 

This is exactly the right source for:
- reproducible regeneration
- version pinning (by recording the Unicode version separately)
- long-term maintenance

```
curl -O https://www.unicode.org/Public/UCD/latest/ucd/NamesList.txt
```
If later you want to pin to a specific Unicode version (e.g., 15.1.0), the path becomes:

```
https://www.unicode.org/Public/15.1.0/ucd/NamesList.txt
```


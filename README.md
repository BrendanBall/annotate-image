# Annotate Image
This tool currently does one very specific thing. It annotates an image with the timestamp found in the metadata. If there is no metadata, you can manually specify text to annotate. 
I first attempted to use GraphicsMagick, with options something like this:

```
gm convert IMG_20200109_200413.jpg -gravity NorthWest -font /usr/share/fonts/Type1/c0648bt_.pfb -pointsize 200 -draw 'text 200,200 "%[EXIF:DateTimeOriginal]"'  IMG.jpg
```

However I was trying to convert a folder of photos of different sizes and orientations, which doesn't play nice with having to specify the pointsize etc.
Instead of spending more time trying to hack together a script that would actually work, I took the opportunity to learn some Rust and just implement exactly what I needed. So this was more a learning exercise than anything else. 

Command Line Usage
==================
    $ img-dup --help
    Duplicate Image Finder

    Options:
	-t --threads [1+]   How many threads the program should use to process
			    images. Defaults to the number of cores reported by
			    the OS.
	-d --dir [directory]
			    The directory the program should search in. Default is
			    the current working directory.
	-r --recurse        If present, the program will search subdirectories.
	-h --hash-size [1+] Helps the program decide the number of bits to use for
			    the hash. A higher number means more detail, but
			    greater memory usage. Default is 8
	-s --threshold [0.01 - 99.99]
			    The amount in percentage that an image must be
			    different from another to qualify as unique. Default
			    is 3
	-f --fast           Use a faster, less accurate algorithm. Really only
			    useful for finding duplicates. Using a low threshold
			    and/or a larger hash is recommended.
	-e --ext [extension]
			    Search for filenames with the given extension.
			    Defaults are jpeg, jpg, png, and gif.
	-o --outfile [file] Output to the given file. If omitted, will print to
			    stdout. If not absolute, it will be relative to the
			    search directory.
	--help              Display this help.
	-u --dup-only       Only output images with similars or duplicates.
	-l --limit [1+]     Only process the given number of images.
	-j --json [[1+] (optional)]
			    Output the results in JSON format. If outputting to
			    stdout, normal output is suppressed. An integer may
			    optionally be passed with this flag, indicating the
			    number of spaces to indent per level. Otherwise, the
			    JSON will be in compact format. See the README for
			    details.



Given no arguments, `img-dup` will search the current working directory with a configuration that should be optimal
for most use cases, as discovered via brief experimentation. It will output its results to stdout, which may not be a good idea for large galleries as it can easily overflow the terminal window buffer.

`img-dup --outfile=results.txt` will put the results of the search to `results.txt` in the search directory, specified by `--dir=[directory]` or otherwise the current working directory. If it already exists, the file will be overwritten.

`img-dup` can take quite a long time to process all the images it finds, depending on the average size and the number of images in a directory tree. It took about an hour to process ~2300 images (~2.3GB) on the following machine:

* Core i7 3770k (stock clocks) (8 logical cores as reported by the OS)
* 16 GB DDR3 RAM
* Windows 7 64-bit
* 1 TB HDD 7200RPM SATA3

You might see performance improvements using a higher number of threads than the default (the number of cores in your CPU as reported by your OS), since many of them will be blocked on I/O at any given point. An SSD or RAMDisk might further improve search speeds, as will a properly defragmented hard drive (if using NTFS).

However, in my experiments, all 8 cores were at 100% capacity most of the time, so the bottleneck might actually be in decoding the images and not loading them from disk. Further experimentation with the help of a profiler might be needed.

A `--threshold` of greater than 3(%) difference often produces misleading results, as the perceptual hash will find images that are "similar" in structure or composition but aren't subjectively similar to the human eye. Exact duplicates are always 0% different, and resizes and minor edits are usually within 2%.

If detail is a concern, a larger threshold should be used with a larger `--hash-size` setting, though memory usage increases on the order of `O([number of images] * hash-size^2)`. The actual image data isn't kept in memory after being hashed, so memory usage shouldn't be much of a concern. In the above test, `img-dup` kept below 500MB for the duration of the test.

GIF files are currently not searched for by default due to an elusive bug in `rust-image` that may or may not have to do with animations. You can add `--ext=gif` to search for them. Errors produced during decoding or hashing are now safely caught and logged so the task can continue. Errored images are reported in the processing results.

For JSON structure, see `JSON.md`.


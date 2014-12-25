###img_dup GUI Guide

To start `img_dup` in GUI mode, pass the `-g` flag. Any other config flags will set the starting values in the setup window.

Cargo:
```shell
cargo run -- -g
```

Standalone binary:
```shell
./img_dup -g
```

###Setup Window
This window is where you set the search configuration. Set the options and click "Go!" to begin the search. The options are as follows:

#####Search Directory
The directory to search for images. Click "Browse" to browse for a folder, or enter a path into the text field. 

Also set by the `--dir` command-line flag.

#####Threads
The number of threads to run image loading and hashing in parallel. By default, this is set to the number of CPUs as reported by your operating system. Hyperthreaded Intel CPUs will report double the number of physical cores.

It is recommended to leave this at default for best performance. More threads will usually only introduce additional overhead, while fewer threads will throttle the loading and hashing. 

Reduce the number of threads if the default makes your computer slow or unstable in the next step.

Also set by the `--threads` command-line flag.

#####Hash Size
The square of this number is the number of bits to use in the image hash. Generally, a higher number will create a more detailed hash at the cost of memory usage and performance. Reduce this to reduce memory usage and possibly improve performance at the cost of accuracy.

#####Recurse
If set (black square instead of white)
